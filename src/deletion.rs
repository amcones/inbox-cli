//! Recoverable deletion of one or many notes.
use crate::{
    Result,
    i18n::text,
    markdown, query,
    storage::{Store, open_rw, sync_dir},
    trash, views,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashSet},
    fs,
    io::{Cursor, Write},
    path::{Path, PathBuf},
};

const PENDING: &str = ".inbox/delete-pending";

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    #[serde(default)]
    dates: Vec<String>,
    #[serde(default)]
    ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    date: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    #[serde(default)]
    trash: bool,
}

#[derive(Clone, Debug)]
pub struct Plan {
    pub ids: Vec<String>,
}

pub fn pending(root: &Path) -> Result<bool> {
    Ok(root.join(PENDING).join("manifest.json").try_exists()?)
}

pub fn plan_id(store: &Store, prefix: &str) -> Result<Plan> {
    Ok(Plan {
        ids: vec![query::find(store, prefix)?.meta.id],
    })
}

pub fn plan_date(store: &Store, date: &str) -> Result<Plan> {
    let mut ids = Vec::new();
    for path in store.day_files()? {
        if path.file_stem().and_then(|s| s.to_str()) == Some(date) {
            store.scan_file(&path, |note| {
                ids.push(note.meta.id);
                Ok(())
            })?;
            break;
        }
    }
    Ok(Plan { ids })
}

pub fn plan_range(store: &Store, date: &str, start_hour: u8, end_hour: u8) -> Result<Plan> {
    let mut ids = Vec::new();
    for path in store.day_files()? {
        if path.file_stem().and_then(|s| s.to_str()) != Some(date) {
            continue;
        }
        store.scan_file(&path, |note| {
            let hour = note.meta.created[11..13].parse::<u8>()?;
            if hour >= start_hour && hour < end_hour {
                ids.push(note.meta.id);
            }
            Ok(())
        })?;
        break;
    }
    Ok(Plan { ids })
}

pub fn plan_all(store: &Store) -> Result<Plan> {
    let mut ids = Vec::new();
    for path in store.day_files()? {
        store.scan_file(&path, |note| {
            ids.push(note.meta.id);
            Ok(())
        })?;
    }
    Ok(Plan { ids })
}

/// Deletes exactly the confirmed snapshot. Notes added after planning survive.
pub fn execute(store: &Store, plan: &Plan) -> Result<usize> {
    store.require_exclusive()?;
    if plan.ids.is_empty() {
        return Ok(0);
    }
    let wanted: HashSet<_> = plan.ids.iter().cloned().collect();
    if wanted.len() != plan.ids.len() {
        return Err(text(
            "删除清单包含重复 ID",
            "The deletion plan contains duplicate IDs",
        )
        .into());
    }
    let mut found = HashSet::new();
    let mut removed = BTreeMap::new();
    let mut replacements = BTreeMap::<String, String>::new();
    for path in store.day_files()? {
        let mut notes = Vec::new();
        store.scan_file(&path, |note| {
            notes.push(note);
            Ok(())
        })?;
        if !notes.iter().any(|note| wanted.contains(&note.meta.id)) {
            continue;
        }
        let date = path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| text("无效的日期文件名", "Invalid day filename"))?;
        let mut next = format!("# {date}\n\n");
        for note in notes {
            if wanted.contains(&note.meta.id) {
                found.insert(note.meta.id.clone());
                removed.insert(note.meta.id.clone(), note);
            } else {
                next.push_str(&markdown::encode(&note)?);
            }
        }
        replacements.insert(date.to_owned(), next);
    }
    if found != wanted {
        return Err(text(
            "确认后部分灵感已不存在，未执行删除",
            "Some confirmed notes no longer exist; nothing was deleted",
        )
        .into());
    }
    views::counts(store)?;
    let view_path = store.root.join(".inbox/views.log");
    let view_text = match fs::read_to_string(&view_path) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(e.into()),
    };
    let log = view_text
        .split_inclusive('\n')
        .filter(|line| {
            line.split_once('\t')
                .is_none_or(|(id, _)| !wanted.contains(id))
        })
        .collect::<String>();

    let stage = store.root.join(PENDING);
    cleanup_uncommitted(&stage)?;
    crate::storage::ensure_dir(&stage)?;
    trash::ensure_trash(&store.root)?;
    for id in &plan.ids {
        if trash::path(&store.root, id).try_exists()? {
            return Err(text(
                "回收站中已存在相同 ID，未执行删除",
                "Trash already contains the same ID; nothing was deleted",
            )
            .into());
        }
    }
    let deleted_at = jiff::Timestamp::now().to_string();
    for (id, note) in &removed {
        write_stage(
            &stage.join(trash_stage_name(id)),
            &trash::encode(note, &deleted_at)?,
        )?;
    }
    for (date, content) in &replacements {
        write_stage(&stage.join(stage_name(date)), content.as_bytes())?;
    }
    write_stage(&stage.join("views.next"), log.as_bytes())?;
    let manifest = serde_json::to_vec(&Manifest {
        dates: replacements.keys().cloned().collect(),
        ids: plan.ids.clone(),
        date: None,
        id: None,
        trash: true,
    })?;
    write_stage(&stage.join("manifest.tmp"), &manifest)?;
    fs::rename(stage.join("manifest.tmp"), stage.join("manifest.json"))?;
    sync_dir(&stage)?;
    recover(&store.root).map_err(|e| {
        crate::message!(
            "删除已提交但尚未完成；下次运行 inbox 将继续恢复：{e}",
            "Deletion committed but not fully applied; the next inbox command will recover it: {e}"
        )
    })?;
    Ok(plan.ids.len())
}

pub fn recover(root: &Path) -> Result<()> {
    let stage = root.join(PENDING);
    let manifest_path = stage.join("manifest.json");
    if !manifest_path.try_exists()? {
        return Ok(());
    }
    ensure_regular_dir(&stage)?;
    let mut manifest: Manifest = serde_json::from_slice(&fs::read(&manifest_path)?)?;
    let legacy = manifest.dates.is_empty() && manifest.date.is_some();
    if manifest.dates.is_empty()
        && let Some(date) = manifest.date.take()
    {
        manifest.dates.push(date);
    }
    if manifest.ids.is_empty()
        && let Some(id) = manifest.id.take()
    {
        manifest.ids.push(id);
    }
    validate_manifest(&manifest)?;
    if manifest.trash {
        let trash_dir = trash::ensure_trash(root)?;
        for id in &manifest.ids {
            let staged = stage.join(trash_stage_name(id));
            let destination = trash::path(root, id);
            if staged.try_exists()? {
                if destination.try_exists()? {
                    if fs::read(&staged)? != fs::read(&destination)? {
                        return Err(text(
                            "回收站中存在冲突的灵感",
                            "Trash contains a conflicting note",
                        )
                        .into());
                    }
                    fs::remove_file(staged)?;
                } else {
                    fs::rename(staged, destination)?;
                }
            }
        }
        sync_dir(&trash_dir)?;
    }
    for date in &manifest.dates {
        let destination = day_path(root, date)?;
        let staged = stage.join(stage_name(date));
        let source = if legacy && !staged.try_exists()? {
            stage.join("day.next")
        } else {
            staged
        };
        replace_if_staged(&source, &destination)?;
        sync_dir(destination.parent().unwrap())?;
    }
    let log_path = root.join(".inbox/views.log");
    replace_if_staged(&stage.join("views.next"), &log_path)?;
    sync_dir(log_path.parent().unwrap())?;

    let ids: HashSet<_> = manifest.ids.iter().map(String::as_str).collect();
    for date in &manifest.dates {
        let path = day_path(root, date)?;
        markdown::scan(Cursor::new(fs::read(&path)?), &path, |note| {
            if ids.contains(note.meta.id.as_str()) {
                return Err(text("删除恢复不完整，请保留恢复目录并检查文件", "Deletion recovery is incomplete; preserve the recovery directory and inspect the files").into());
            }
            Ok(())
        })?;
    }
    let log = fs::read_to_string(&log_path)?;
    if log.lines().any(|line| {
        line.split_once('\t')
            .is_some_and(|(id, _)| ids.contains(id))
    }) {
        return Err(text("删除恢复不完整，请保留恢复目录并检查文件", "Deletion recovery is incomplete; preserve the recovery directory and inspect the files").into());
    }
    if manifest.trash {
        for id in &manifest.ids {
            let (_, note) =
                serde_json::from_slice::<trash::Entry>(&fs::read(trash::path(root, id))?)?
                    .validate()?;
            if note.meta.id != *id {
                return Err(text("删除恢复不完整，请保留恢复目录并检查文件", "Deletion recovery is incomplete; preserve the recovery directory and inspect the files").into());
            }
        }
    }
    cleanup_committed(&stage, &manifest)?;
    sync_dir(&root.join(".inbox"))?;
    Ok(())
}

fn validate_manifest(manifest: &Manifest) -> Result<()> {
    if manifest.dates.is_empty() || manifest.ids.is_empty() {
        return Err(text("删除恢复清单为空", "Deletion recovery manifest is empty").into());
    }
    let mut dates = HashSet::new();
    for date in &manifest.dates {
        validate_date(date)?;
        if !dates.insert(date) {
            return Err(text(
                "删除恢复清单包含重复日期",
                "Deletion recovery manifest contains duplicate dates",
            )
            .into());
        }
    }
    let mut ids = HashSet::new();
    for id in &manifest.ids {
        let parsed = uuid::Uuid::parse_str(id)?;
        if parsed.to_string() != *id || !ids.insert(id) {
            return Err(text(
                "删除恢复清单包含无效 ID",
                "Deletion recovery manifest contains an invalid ID",
            )
            .into());
        }
    }
    Ok(())
}

fn replace_if_staged(source: &Path, destination: &Path) -> Result<()> {
    if source.try_exists()? {
        if !fs::symlink_metadata(source)?.file_type().is_file() {
            return Err(text("删除恢复文件无效", "Invalid deletion recovery file").into());
        }
        fs::rename(source, destination)?;
    }
    Ok(())
}

fn day_path(root: &Path, date: &str) -> Result<PathBuf> {
    validate_date(date)?;
    let year = root.join(&date[..4]);
    let month = year.join(&date[5..7]);
    for dir in [&year, &month] {
        if !fs::symlink_metadata(dir)?.file_type().is_dir() {
            return Err(text(
                "日期目录不能是符号链接",
                "Date directories must not be symlinks",
            )
            .into());
        }
    }
    Ok(month.join(format!("{date}.md")))
}

fn validate_date(date: &str) -> Result<()> {
    if date.len() != 10 || !date.is_ascii() || date.parse::<jiff::civil::Date>().is_err() {
        return Err(text("删除恢复日期无效", "Invalid deletion recovery date").into());
    }
    Ok(())
}

fn stage_name(date: &str) -> String {
    format!("day-{date}.next")
}

fn trash_stage_name(id: &str) -> String {
    format!("trash-{id}.json.next")
}

fn write_stage(path: &Path, bytes: &[u8]) -> Result<()> {
    let (mut file, fresh) = open_rw(path, false)?;
    if !fresh {
        return Err(text("删除暂存文件已存在", "Deletion staging file already exists").into());
    }
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}

fn ensure_regular_dir(stage: &Path) -> Result<()> {
    if !fs::symlink_metadata(stage)?.file_type().is_dir() {
        return Err(text(
            "删除恢复目录必须是普通目录",
            "Deletion recovery path must be a regular directory",
        )
        .into());
    }
    Ok(())
}

fn cleanup_uncommitted(stage: &Path) -> Result<()> {
    if !stage.try_exists()? {
        return Ok(());
    }
    ensure_regular_dir(stage)?;
    if stage.join("manifest.json").try_exists()? {
        return Err(text(
            "存在尚未恢复的删除事务",
            "A deletion transaction still needs recovery",
        )
        .into());
    }
    for entry in fs::read_dir(stage)? {
        let entry = entry?;
        let name = entry.file_name();
        let name = name
            .to_str()
            .ok_or_else(|| text("删除暂存文件名无效", "Invalid deletion staging filename"))?;
        let known = matches!(name, "day.next" | "views.next" | "manifest.tmp")
            || name
                .strip_prefix("day-")
                .and_then(|s| s.strip_suffix(".next"))
                .is_some_and(|date| validate_date(date).is_ok())
            || name
                .strip_prefix("trash-")
                .and_then(|s| s.strip_suffix(".json.next"))
                .is_some_and(|id| {
                    uuid::Uuid::parse_str(id).is_ok_and(|parsed| parsed.to_string() == id)
                });
        if !known || !entry.file_type()?.is_file() {
            return Err(text(
                "删除恢复目录包含未知文件",
                "Deletion recovery directory contains an unknown file",
            )
            .into());
        }
        fs::remove_file(entry.path())?;
    }
    fs::remove_dir(stage)?;
    Ok(())
}

fn cleanup_committed(stage: &Path, manifest: &Manifest) -> Result<()> {
    for date in &manifest.dates {
        remove_if_exists(&stage.join(stage_name(date)))?;
    }
    for id in &manifest.ids {
        remove_if_exists(&stage.join(trash_stage_name(id)))?;
    }
    for name in ["day.next", "views.next", "manifest.tmp", "manifest.json"] {
        remove_if_exists(&stage.join(name))?;
    }
    fs::remove_dir(stage)?;
    Ok(())
}

fn remove_if_exists(path: &Path) -> Result<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e.into()),
    }
}
