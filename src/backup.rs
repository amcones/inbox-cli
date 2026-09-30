use crate::{
    Result, query,
    storage::{Store, ensure_dir, sync_dir},
    trash,
};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Component, Path, PathBuf},
};

const MANIFEST: &str = "inbox-backup.json";
const DATA: &str = "data";
const PENDING: &str = ".inbox/restore-pending";
const TRANSACTION: &str = "manifest.json";

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    version: u8,
    created_at: String,
    inbox_version: String,
    files: u64,
    bytes: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Transaction {
    version: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stats {
    pub files: u64,
    pub bytes: u64,
}

pub struct RestoreResult {
    pub safety_backup: PathBuf,
    pub stats: Stats,
}

pub struct BackupInfo {
    pub stats: Stats,
    pub created_at: String,
    pub inbox_version: String,
}

pub fn create(store: &Store, destination: &Path) -> Result<Stats> {
    store.require_exclusive()?;
    validate_store(store)?;
    let destination = absolute(destination)?;
    reject_overlapping(&store.root, &destination)?;
    if destination.try_exists()? {
        return Err(crate::message!(
            "{}: 备份目录已存在",
            "{}: backup destination already exists",
            destination.display()
        )
        .into());
    }
    let parent = destination.parent().ok_or_else(|| {
        crate::i18n::text(
            "备份目录必须有父目录",
            "The backup destination must have a parent directory",
        )
    })?;
    ensure_dir(parent)?;
    let staging = parent.join(format!(".inbox-backup-{}-tmp", uuid::Uuid::new_v4()));
    let result = (|| {
        ensure_dir(&staging)?;
        let data = staging.join(DATA);
        let stats = copy_tree(&store.root, &data, CopyMode::Backup)?;
        let manifest = Manifest {
            version: 1,
            created_at: jiff::Timestamp::now().to_string(),
            inbox_version: env!("CARGO_PKG_VERSION").to_owned(),
            files: stats.files,
            bytes: stats.bytes,
        };
        write_new(
            &staging.join(MANIFEST),
            &serde_json::to_vec_pretty(&manifest)?,
        )?;
        sync_dir(&staging)?;
        validate(&staging)?;
        fs::rename(&staging, &destination)?;
        sync_dir(parent)?;
        Ok(stats)
    })();
    if result.is_err() {
        let _ = fs::remove_dir_all(&staging);
    }
    result
}

pub fn validate(path: &Path) -> Result<BackupInfo> {
    let path = absolute(path)?;
    let metadata = fs::symlink_metadata(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    if !metadata.file_type().is_dir() {
        return Err(crate::message!(
            "{}: 备份必须是普通目录",
            "{}: backup must be a regular directory",
            path.display()
        )
        .into());
    }
    for entry in fs::read_dir(&path)? {
        let entry = entry?;
        let name = entry.file_name();
        if name != MANIFEST && name != DATA {
            return Err(crate::message!(
                "{}: 备份包含未知项目",
                "{}: backup contains an unknown item",
                entry.path().display()
            )
            .into());
        }
        let ty = entry.file_type()?;
        if (name == MANIFEST && !ty.is_file()) || (name == DATA && !ty.is_dir()) {
            return Err(crate::message!(
                "{}: 备份结构无效",
                "{}: invalid backup structure",
                entry.path().display()
            )
            .into());
        }
    }
    let manifest: Manifest = serde_json::from_slice(&fs::read(path.join(MANIFEST))?)?;
    if manifest.version != 1 {
        return Err(
            crate::i18n::text("不支持的备份格式版本", "Unsupported backup format version").into(),
        );
    }
    let created: jiff::Timestamp = manifest.created_at.parse()?;
    if created.to_string() != manifest.created_at || manifest.inbox_version.is_empty() {
        return Err(crate::i18n::text("备份清单无效", "Invalid backup manifest").into());
    }
    let data = path.join(DATA);
    if pending(&data)? || crate::deletion::pending(&data)? {
        return Err(crate::i18n::text(
            "备份包含未完成的事务",
            "The backup contains an unfinished transaction",
        )
        .into());
    }
    let stats = tree_stats(&data, CopyMode::Backup)?;
    if stats.files != manifest.files || stats.bytes != manifest.bytes {
        return Err(crate::i18n::text(
            "备份内容与清单不一致",
            "Backup contents do not match the manifest",
        )
        .into());
    }
    let store = Store::open_snapshot(&data)?;
    validate_store(&store)?;
    Ok(BackupInfo {
        stats,
        created_at: manifest.created_at,
        inbox_version: manifest.inbox_version,
    })
}

pub fn restore(store: &Store, source: &Path) -> Result<RestoreResult> {
    store.require_exclusive()?;
    let source = absolute(source)?;
    reject_overlapping(&store.root, &source)?;
    let info = validate(&source)?;
    let stats = info.stats;
    let safety_backup = safety_path(&store.root)?;
    create(store, &safety_backup)?;

    let pending = store.root.join(PENDING);
    if pending.try_exists()? {
        return Err(crate::i18n::text(
            "已有未完成的还原事务",
            "An unfinished restore transaction already exists",
        )
        .into());
    }
    ensure_dir(&pending)?;
    let prepared = (|| {
        let staged = copy_tree(&source.join(DATA), &pending.join("next"), CopyMode::Restore)?;
        if staged != stats {
            return Err(crate::i18n::text(
                "复制期间备份内容发生变化",
                "Backup contents changed while being copied",
            )
            .into());
        }
        let staged_store = Store::open(&pending.join("next"), false)?.ok_or_else(|| {
            crate::i18n::text("暂存的备份数据为空", "The staged backup data is empty")
        })?;
        validate_store(&staged_store)?;
        drop(staged_store);
        write_new(
            &pending.join(TRANSACTION),
            &serde_json::to_vec_pretty(&Transaction { version: 1 })?,
        )?;
        sync_dir(&pending)?;
        sync_dir(&store.root.join(".inbox"))?;
        apply_pending(&store.root)
    })();
    if prepared.is_err() && !pending.join(TRANSACTION).try_exists().unwrap_or(true) {
        let _ = fs::remove_dir_all(&pending);
    }
    prepared?;
    validate_store(store)?;
    Ok(RestoreResult {
        safety_backup,
        stats,
    })
}

pub(crate) fn pending(root: &Path) -> Result<bool> {
    Ok(root.join(PENDING).try_exists()?)
}

pub(crate) fn recover(root: &Path) -> Result<()> {
    let pending = root.join(PENDING);
    if !pending.try_exists()? {
        return Ok(());
    }
    if !pending.join(TRANSACTION).try_exists()? {
        fs::remove_dir_all(&pending)?;
        sync_dir(&root.join(".inbox"))?;
        return Ok(());
    }
    apply_pending(root)
}

fn apply_pending(root: &Path) -> Result<()> {
    let pending = root.join(PENDING);
    let transaction: Transaction = serde_json::from_slice(&fs::read(pending.join(TRANSACTION))?)?;
    if transaction.version != 1 {
        return Err(crate::i18n::text(
            "不支持的还原事务版本",
            "Unsupported restore transaction version",
        )
        .into());
    }
    let next = pending.join("next");
    tree_stats(&next, CopyMode::Restore)?;
    let next_store = Store::open(&next, false)?.ok_or_else(|| {
        crate::i18n::text(
            "还原事务缺少暂存数据",
            "The restore transaction is missing staged data",
        )
    })?;
    validate_store(&next_store)?;
    drop(next_store);

    for entry in fs::read_dir(root)? {
        let entry = entry?;
        if entry.file_name() == ".inbox" {
            for state in fs::read_dir(entry.path())? {
                let state = state?;
                if matches!(
                    state.file_name().to_str(),
                    Some("write.lock" | "restore-pending")
                ) {
                    continue;
                }
                remove_entry(&state.path())?;
            }
        } else {
            remove_entry(&entry.path())?;
        }
    }
    copy_contents(&next, root, CopyMode::Apply, Path::new(""))?;
    sync_dir(root)?;
    fs::remove_dir_all(&pending)?;
    sync_dir(&root.join(".inbox"))?;
    Ok(())
}

fn validate_store(store: &Store) -> Result<()> {
    query::doctor(store)?;
    trash::list(store)?;
    Ok(())
}

fn safety_path(root: &Path) -> Result<PathBuf> {
    let parent = root.parent().ok_or_else(|| {
        crate::i18n::text(
            "数据目录必须有父目录",
            "The data directory must have a parent directory",
        )
    })?;
    let name = root.file_name().and_then(|s| s.to_str()).unwrap_or("inbox");
    let stamp = jiff::Timestamp::now()
        .to_zoned(jiff::tz::TimeZone::try_system()?)
        .strftime("%Y%m%d-%H%M%S");
    Ok(parent.join(format!(
        "{name}.before-restore-{stamp}-{}",
        &uuid::Uuid::new_v4().to_string()[..8]
    )))
}

fn reject_overlapping(a: &Path, b: &Path) -> Result<()> {
    let a = resolved(a)?;
    let b = resolved(b)?;
    if a == b || a.starts_with(&b) || b.starts_with(&a) {
        return Err(crate::i18n::text(
            "数据目录与备份目录不能相同或互相包含",
            "The data and backup directories must be separate and non-nested",
        )
        .into());
    }
    Ok(())
}

fn resolved(path: &Path) -> Result<PathBuf> {
    let mut cursor = absolute(path)?;
    let mut suffix = Vec::new();
    while !cursor.try_exists()? {
        let name = cursor
            .file_name()
            .ok_or_else(|| crate::i18n::text("路径无法解析", "The path cannot be resolved"))?;
        suffix.push(name.to_owned());
        cursor = cursor
            .parent()
            .ok_or_else(|| crate::i18n::text("路径无法解析", "The path cannot be resolved"))?
            .to_owned();
    }
    let mut result = fs::canonicalize(cursor)?;
    for component in suffix.into_iter().rev() {
        result.push(component);
    }
    Ok(result)
}

fn absolute(path: &Path) -> Result<PathBuf> {
    let joined = if path.is_absolute() {
        path.to_owned()
    } else {
        std::env::current_dir()?.join(path)
    };
    let mut clean = PathBuf::new();
    for component in joined.components() {
        match component {
            Component::CurDir => (),
            Component::ParentDir => {
                clean.pop();
            }
            other => clean.push(other.as_os_str()),
        }
    }
    Ok(clean)
}

#[derive(Clone, Copy)]
enum CopyMode {
    Backup,
    Restore,
    Apply,
}

fn copy_tree(source: &Path, destination: &Path, mode: CopyMode) -> Result<Stats> {
    tree_stats(source, mode)?;
    ensure_dir(destination)?;
    let stats = copy_contents(source, destination, mode, Path::new(""))?;
    sync_dir(destination)?;
    Ok(stats)
}

fn copy_contents(
    source: &Path,
    destination: &Path,
    mode: CopyMode,
    relative: &Path,
) -> Result<Stats> {
    let mut stats = Stats { files: 0, bytes: 0 };
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let rel = relative.join(entry.file_name());
        if skip(&rel, mode) {
            continue;
        }
        let ty = entry.file_type()?;
        let target = destination.join(entry.file_name());
        if ty.is_dir() {
            ensure_dir(&target)?;
            let child = copy_contents(&entry.path(), &target, mode, &rel)?;
            stats.files += child.files;
            stats.bytes += child.bytes;
            sync_dir(&target)?;
        } else if ty.is_file() {
            let bytes = fs::copy(entry.path(), &target)?;
            private_file(&target)?;
            File::open(&target)?.sync_all()?;
            stats.files += 1;
            stats.bytes += bytes;
        } else {
            return Err(crate::message!(
                "{}: 只允许普通文件和目录",
                "{}: only regular files and directories are allowed",
                entry.path().display()
            )
            .into());
        }
    }
    Ok(stats)
}

fn tree_stats(root: &Path, mode: CopyMode) -> Result<Stats> {
    let meta = fs::symlink_metadata(root).map_err(|e| format!("{}: {e}", root.display()))?;
    if !meta.file_type().is_dir() {
        return Err(crate::message!(
            "{}: 需要普通目录",
            "{}: expected a regular directory",
            root.display()
        )
        .into());
    }
    fn walk(path: &Path, relative: &Path, mode: CopyMode, stats: &mut Stats) -> Result<()> {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let rel = relative.join(entry.file_name());
            if skip(&rel, mode) {
                continue;
            }
            let ty = entry.file_type()?;
            if ty.is_dir() {
                walk(&entry.path(), &rel, mode, stats)?;
            } else if ty.is_file() {
                stats.files += 1;
                stats.bytes += entry.metadata()?.len();
            } else {
                return Err(crate::message!(
                    "{}: 只允许普通文件和目录",
                    "{}: only regular files and directories are allowed",
                    entry.path().display()
                )
                .into());
            }
        }
        Ok(())
    }
    let mut stats = Stats { files: 0, bytes: 0 };
    walk(root, Path::new(""), mode, &mut stats)?;
    Ok(stats)
}

fn skip(relative: &Path, mode: CopyMode) -> bool {
    let text = relative.to_string_lossy().replace('\\', "/");
    match mode {
        CopyMode::Backup => text == ".inbox/restore-pending",
        CopyMode::Restore => text == ".inbox/restore-pending",
        CopyMode::Apply => matches!(
            text.as_str(),
            ".inbox/write.lock" | ".inbox/restore-pending"
        ),
    }
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}

fn private_file(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    }
    #[cfg(not(unix))]
    let _ = path;
    Ok(())
}

fn remove_entry(path: &Path) -> Result<()> {
    let ty = fs::symlink_metadata(path)?.file_type();
    if ty.is_dir() {
        fs::remove_dir_all(path)?;
    } else {
        fs::remove_file(path)?;
    }
    Ok(())
}
