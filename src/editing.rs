use crate::{
    Result,
    i18n::text,
    markdown,
    model::Note,
    query,
    storage::{Store, open_rw, sync_dir},
};
use std::{fs, io::Write, path::Path};

pub fn edit(
    store: &Store,
    prefix: &str,
    content: Option<String>,
    tags: Option<Vec<String>>,
) -> Result<Note> {
    store.require_exclusive()?;
    let original = query::find(store, prefix)?;
    let updated = original.updated(content, tags)?;
    let date = &original.meta.created[..10];
    let path = store
        .root
        .join(&date[..4])
        .join(&date[5..7])
        .join(format!("{date}.md"));
    let mut notes = Vec::new();
    store.scan_file(&path, |note| {
        notes.push(note);
        Ok(())
    })?;
    let mut replaced = false;
    let mut next = format!("# {date}\n\n");
    for note in notes {
        if note.meta.id == original.meta.id {
            next.push_str(&markdown::encode(&updated)?);
            replaced = true;
        } else {
            next.push_str(&markdown::encode(&note)?);
        }
    }
    if !replaced {
        return Err(text(
            "编辑时灵感已不存在",
            "The note disappeared while it was being edited",
        )
        .into());
    }
    replace_file(&path, next.as_bytes())?;
    Ok(updated)
}

fn replace_file(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| text("日期文件路径无效", "Invalid day file path"))?;
    let temp = parent.join(format!(".edit-{}.tmp", uuid::Uuid::new_v4()));
    let result = (|| -> Result<()> {
        let (mut file, fresh) = open_rw(&temp, false)?;
        if !fresh {
            return Err(text("编辑暂存文件已存在", "Edit staging file already exists").into());
        }
        file.write_all(bytes)?;
        file.sync_all()?;
        replace_existing(&temp, path)?;
        sync_dir(parent)?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    result
}

#[cfg(not(windows))]
fn replace_existing(source: &Path, destination: &Path) -> Result<()> {
    fs::rename(source, destination)?;
    Ok(())
}

#[cfg(windows)]
fn replace_existing(source: &Path, destination: &Path) -> Result<()> {
    use std::os::windows::ffi::OsStrExt;
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn MoveFileExW(existing: *const u16, new: *const u16, flags: u32) -> i32;
    }
    const REPLACE_EXISTING: u32 = 0x1;
    const WRITE_THROUGH: u32 = 0x8;
    let source: Vec<u16> = source.as_os_str().encode_wide().chain(Some(0)).collect();
    let destination: Vec<u16> = destination
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect();
    // Both paths are local, NUL-terminated UTF-16 buffers that remain alive
    // for the call. MoveFileExW atomically replaces the existing day file.
    let ok = unsafe {
        MoveFileExW(
            source.as_ptr(),
            destination.as_ptr(),
            REPLACE_EXISTING | WRITE_THROUGH,
        )
    };
    if ok == 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    Ok(())
}
