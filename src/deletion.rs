//! A small roll-forward journal for deleting from Markdown and the view log.
//! All calls hold the store's exclusive OS lock. A durable manifest commits
//! the intent; interrupted committed transactions are completed on next open.
use crate::{
    Result,
    i18n::text,
    markdown,
    model::Metadata,
    query,
    storage::{Store, open_rw, sync_dir},
    views,
};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{Cursor, Write},
    path::{Path, PathBuf},
};

const PENDING: &str = ".inbox/delete-pending";

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    date: String,
    id: String,
}

pub fn pending(root: &Path) -> Result<bool> {
    Ok(root.join(PENDING).join("manifest.json").try_exists()?)
}

pub fn delete(store: &Store, prefix: &str) -> Result<String> {
    store.require_exclusive()?;
    let note = query::find(store, prefix)?; // Full scan also rejects ambiguous IDs.
    let date = &note.meta.created[..10];
    let path = day_path(&store.root, date)?;
    let original = fs::read_to_string(&path)?;
    let updated = without_note(&original, &path, &note.meta.id)?;
    // Validate before touching either source, including incomplete log tails.
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
                .is_none_or(|(id, _)| id != note.meta.id)
        })
        .collect::<String>();
    let stage = store.root.join(PENDING);
    cleanup(&stage)?; // Leftovers without a manifest were never committed.
    crate::storage::ensure_dir(&stage)?;
    write_stage(&stage.join("day.next"), updated.as_bytes())?;
    write_stage(&stage.join("views.next"), log.as_bytes())?;
    let manifest = serde_json::to_vec(&Manifest {
        date: date.into(),
        id: note.meta.id.clone(),
    })?;
    write_stage(&stage.join("manifest.tmp"), &manifest)?;
    fs::rename(stage.join("manifest.tmp"), stage.join("manifest.json"))?;
    sync_dir(&stage)?;
    // After this commit point, any error leaves the journal for recovery.
    recover(&store.root).map_err(|e| {
        crate::message!(
            "删除已提交但尚未完成；下次运行 inbox 将继续恢复：{e}",
            "Deletion committed but not fully applied; the next inbox command will recover it: {e}"
        )
    })?;
    Ok(note.meta.id)
}

pub fn recover(root: &Path) -> Result<()> {
    let stage = root.join(PENDING);
    let manifest_path = stage.join("manifest.json");
    if !manifest_path.try_exists()? {
        return Ok(());
    }
    if !fs::symlink_metadata(&stage)?.file_type().is_dir() {
        return Err(text(
            "删除恢复目录必须是普通目录",
            "Deletion recovery path must be a regular directory",
        )
        .into());
    }
    let manifest: Manifest = serde_json::from_slice(&fs::read(&manifest_path)?)?;
    let target = day_path(root, &manifest.date)?;
    uuid::Uuid::parse_str(&manifest.id)?;
    for (source, destination) in [
        (stage.join("day.next"), target.clone()),
        (stage.join("views.next"), root.join(".inbox/views.log")),
    ] {
        if source.try_exists()? {
            if !fs::symlink_metadata(&source)?.file_type().is_file() {
                return Err(text("删除恢复文件无效", "Invalid deletion recovery file").into());
            }
            fs::rename(&source, &destination)?;
        }
        // Also sync when rename happened before a previous process died.
        sync_dir(destination.parent().unwrap())?;
    }
    // Verify both outputs before discarding the journal. Missing staging files
    // mean a prior rename completed, not permission to lose a source silently.
    let mut still_present = false;
    markdown::scan(Cursor::new(fs::read(&target)?), &target, |note| {
        still_present |= note.meta.id == manifest.id;
        Ok(())
    })?;
    let log = fs::read_to_string(root.join(".inbox/views.log"))?;
    if still_present
        || log.lines().any(|line| {
            line.split_once('\t')
                .is_some_and(|(id, _)| id == manifest.id)
        })
    {
        return Err(text("删除恢复不完整，请保留恢复目录并检查文件", "Deletion recovery is incomplete; preserve the recovery directory and inspect the files").into());
    }
    cleanup(&stage)?;
    sync_dir(&root.join(".inbox"))?;
    Ok(())
}

fn day_path(root: &Path, date: &str) -> Result<PathBuf> {
    if date.len() != 10 || !date.is_ascii() || date.parse::<jiff::civil::Date>().is_err() {
        return Err(text("删除恢复日期无效", "Invalid deletion recovery date").into());
    }
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

fn write_stage(path: &Path, bytes: &[u8]) -> Result<()> {
    let (mut file, fresh) = open_rw(path, false)?;
    if !fresh {
        return Err(text("删除暂存文件已存在", "Deletion staging file already exists").into());
    }
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}

fn cleanup(stage: &Path) -> Result<()> {
    if !stage.try_exists()? {
        return Ok(());
    }
    if !fs::symlink_metadata(stage)?.file_type().is_dir() {
        return Err(text(
            "删除恢复目录必须是普通目录",
            "Deletion recovery path must be a regular directory",
        )
        .into());
    }
    // Only remove our own known filenames, never arbitrary directory contents.
    for name in ["day.next", "views.next", "manifest.tmp", "manifest.json"] {
        match fs::remove_file(stage.join(name)) {
            Ok(()) => (),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
            Err(e) => return Err(e.into()),
        }
    }
    fs::remove_dir(stage)?;
    Ok(())
}

fn without_note(original: &str, path: &Path, id: &str) -> Result<String> {
    markdown::scan(Cursor::new(original), path, |_| Ok(()))?;
    let mut start = None;
    let mut offset = 0;
    for raw in original.split_inclusive('\n') {
        let line = raw.trim_end_matches(['\r', '\n']);
        if let Some(json) = line
            .strip_prefix(markdown::START)
            .and_then(|s| s.strip_suffix(" -->"))
        {
            let meta: Metadata = serde_json::from_str(json)?;
            if meta.id == id {
                start = Some(offset);
            }
        }
        offset += raw.len();
        if let Some(start) = start
            && line
                .strip_prefix(markdown::END)
                .and_then(|s| s.strip_suffix(" -->"))
                == Some(id)
        {
            if original[offset..].starts_with("\r\n") {
                offset += 2;
            } else if original[offset..].starts_with('\n') {
                offset += 1;
            }
            let mut updated = String::with_capacity(original.len() - (offset - start));
            updated.push_str(&original[..start]);
            updated.push_str(&original[offset..]);
            return Ok(updated);
        }
    }
    Err(text("没有找到要删除的记录", "Note to delete was not found").into())
}
