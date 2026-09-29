use crate::{Result, markdown, model::Note};
use std::{
    fs::{self, File, OpenOptions},
    io::{BufReader, Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};

const FORMAT: &[u8] = b"1\n";

pub struct Store {
    pub root: PathBuf,
    // The OS releases this advisory lock even if the process is killed.
    _lock: File,
    exclusive: bool,
}

impl Store {
    /// A missing store is an empty inbox when reading; writes initialize it.
    pub fn open(root: &Path, exclusive: bool) -> Result<Option<Self>> {
        let root = if root.is_absolute() {
            root.to_owned()
        } else {
            std::env::current_dir()?.join(root)
        };
        if !root.try_exists()? && !exclusive {
            return Ok(None);
        }
        ensure_dir(&root)?;
        let state = root.join(".inbox");
        ensure_dir(&state)?;
        let (lock, _) = open_rw(&state.join("write.lock"), false)?;
        let version = state.join("format-version");
        if exclusive || !version.try_exists()? {
            lock.lock()?;
            if !version.try_exists()? {
                let (mut file, _) = open_rw(&version, false)?;
                file.write_all(FORMAT)?;
                file.sync_all()?;
                sync_dir(&state)?;
            }
            if !exclusive {
                lock.unlock()?;
                lock.lock_shared()?;
            }
        } else {
            lock.lock_shared()?;
        }
        if fs::read(&version)? != FORMAT {
            return Err(format!("{}: 不支持或损坏的存储格式版本", version.display()).into());
        }
        Ok(Some(Self {
            root,
            _lock: lock,
            exclusive,
        }))
    }

    pub fn add(&self, note: &Note) -> Result<PathBuf> {
        if !self.exclusive {
            return Err("写入需要独占锁".into());
        }
        crate::model::validate_meta(&note.meta)?;
        let date = &note.meta.created[..10];
        let year_dir = self.root.join(&date[..4]);
        let dir = year_dir.join(&date[5..7]);
        for directory in [&year_dir, &dir] {
            ensure_dir(directory)?;
            if fs::symlink_metadata(directory)?.file_type().is_symlink() {
                return Err(format!("{}: 日期目录不能是符号链接", directory.display()).into());
            }
        }
        let path = dir.join(format!("{date}.md"));
        let (mut file, is_new) = open_rw(&path, true)?;
        let len = file.metadata()?.len();
        let header = format!("# {date}\n\n");
        let mut text = String::new();
        if len == 0 {
            text.push_str(&header);
        } else {
            // Constant-size tail read: adding never parses the day's history.
            let tail = read_tail(&mut file, 256)?;
            let valid_header = len == header.len() as u64 && tail == header.as_bytes();
            let last = tail
                .strip_suffix(b"\n\n")
                .or_else(|| tail.strip_suffix(b"\n"));
            let valid_end = last
                .and_then(|b| b.rsplit(|b| *b == b'\n').next())
                .and_then(|b| std::str::from_utf8(b).ok())
                .and_then(|s| s.strip_prefix(markdown::END))
                .and_then(|s| s.strip_suffix(" -->"))
                .is_some_and(|s| uuid::Uuid::parse_str(s).is_ok());
            if !valid_header && !valid_end {
                return Err(format!("{}: 尾部不完整或格式已改变，拒绝继续追加；请运行 inbox doctor 检查并保留备份后修复", path.display()).into());
            }
            if !tail.ends_with(b"\n\n") {
                text.push('\n');
            }
        }
        text.push_str(&markdown::encode(note)?);
        // A partial write is not atomic. Its missing footer is detected on the
        // next append/read; preserve the bytes instead of silently truncating.
        file.write_all(text.as_bytes())
            .map_err(|e| format!("{}: 写入失败（可能留下部分记录）：{e}", path.display()))?;
        file.sync_all()
            .map_err(|e| format!("{}: 持久化失败，请检查记录后再重试：{e}", path.display()))?;
        if is_new {
            sync_dir(&dir)?;
        }
        Ok(path)
    }

    /// Returns only canonical YYYY/MM/YYYY-MM-DD.md paths, newest civil day first.
    pub fn day_files(&self) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        for year in fs::read_dir(&self.root)? {
            let year = year?;
            let y = year.file_name();
            let Some(y) = y.to_str().filter(|s| digits(s, 4)) else {
                continue;
            };
            if !year.file_type()?.is_dir() {
                continue;
            }
            for month in fs::read_dir(year.path())? {
                let month = month?;
                let m = month.file_name();
                let Some(m) = m
                    .to_str()
                    .filter(|s| digits(s, 2) && *s >= "01" && *s <= "12")
                else {
                    continue;
                };
                if !month.file_type()?.is_dir() {
                    continue;
                }
                for entry in fs::read_dir(month.path())? {
                    let entry = entry?;
                    if !entry.file_type()?.is_file() {
                        continue;
                    }
                    let name = entry.file_name();
                    let Some(name) = name.to_str() else {
                        continue;
                    };
                    if name.len() == 13
                        && name.is_ascii()
                        && name.ends_with(".md")
                        && name.starts_with(&format!("{y}-{m}-"))
                        && name[..10].parse::<jiff::civil::Date>().is_ok()
                    {
                        files.push(entry.path());
                    }
                }
            }
        }
        files.sort_unstable_by(|a, b| b.cmp(a));
        Ok(files)
    }

    pub fn scan_file(&self, path: &Path, visit: impl FnMut(Note) -> Result<()>) -> Result<()> {
        markdown::scan(BufReader::new(File::open(path)?), path, visit)
    }

    pub fn require_exclusive(&self) -> Result<()> {
        if self.exclusive {
            Ok(())
        } else {
            Err("写入需要独占锁".into())
        }
    }
}

fn digits(s: &str, len: usize) -> bool {
    s.len() == len && s.bytes().all(|b| b.is_ascii_digit())
}

pub(crate) fn read_tail(file: &mut File, max: u64) -> Result<Vec<u8>> {
    let len = file.metadata()?.len();
    file.seek(SeekFrom::Start(len.saturating_sub(max)))?;
    let mut tail = Vec::new();
    file.read_to_end(&mut tail)?;
    Ok(tail)
}

pub(crate) fn open_rw(path: &Path, append: bool) -> Result<(File, bool)> {
    let mut options = OpenOptions::new();
    options.read(true).write(true).append(append);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    match options.create_new(true).open(path) {
        Ok(file) => Ok((file, true)),
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            if !fs::symlink_metadata(path)?.file_type().is_file() {
                return Err(
                    format!("{}: 需要普通文件，不能是符号链接或设备", path.display()).into(),
                );
            }
            options.create_new(false);
            Ok((
                options
                    .open(path)
                    .map_err(|e| format!("{}: {e}", path.display()))?,
                false,
            ))
        }
        Err(e) => Err(format!("{}: {e}", path.display()).into()),
    }
}

pub(crate) fn ensure_dir(path: &Path) -> Result<()> {
    if path.is_dir() {
        return Ok(());
    }
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        ensure_dir(parent)?;
    }
    match fs::create_dir(path) {
        Ok(()) => {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
            }
            if let Some(parent) = path.parent() {
                sync_dir(parent)?;
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists && path.is_dir() => (),
        Err(e) => return Err(format!("{}: {e}", path.display()).into()),
    }
    Ok(())
}

pub(crate) fn sync_dir(path: &Path) -> Result<()> {
    #[cfg(unix)]
    File::open(path)?.sync_all()?;
    #[cfg(not(unix))]
    let _ = path;
    Ok(())
}
