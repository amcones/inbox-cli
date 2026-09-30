use crate::{
    Result,
    i18n::text,
    model::{Metadata, Note, normalize_content, normalize_tags, validate_meta},
    storage::{Store, ensure_dir, sync_dir},
};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub const DIR: &str = ".inbox/trash";

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    pub version: u8,
    pub deleted_at: String,
    pub note: StoredNote,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StoredNote {
    pub meta: Metadata,
    pub content: String,
}

impl Entry {
    pub fn new(note: &Note, deleted_at: String) -> Self {
        Self {
            version: 1,
            deleted_at,
            note: StoredNote {
                meta: note.meta.clone(),
                content: note.content.clone(),
            },
        }
    }

    pub fn validate(mut self) -> Result<(Self, Note)> {
        if self.version != 1 {
            return Err(text("不支持的回收站格式版本", "Unsupported trash format version").into());
        }
        let deleted: jiff::Timestamp = self.deleted_at.parse()?;
        if deleted.to_string() != self.deleted_at {
            return Err(text("回收站删除时间格式无效", "Invalid trash deletion timestamp").into());
        }
        // Trash written before v0.4 may contain mixed-case ASCII tags.
        self.note.meta.tags = normalize_tags(self.note.meta.tags)?;
        let timestamp = validate_meta(&self.note.meta)?;
        let content = normalize_content(self.note.content.clone())?;
        let note = Note {
            meta: self.note.meta.clone(),
            timestamp,
            content,
        };
        Ok((self, note))
    }
}

pub fn path(root: &Path, id: &str) -> PathBuf {
    root.join(DIR).join(format!("{id}.json"))
}

pub fn encode(note: &Note, deleted_at: &str) -> Result<Vec<u8>> {
    Ok(serde_json::to_vec_pretty(&Entry::new(
        note,
        deleted_at.to_owned(),
    ))?)
}

pub fn list(store: &Store) -> Result<Vec<(Entry, Note)>> {
    let dir = store.root.join(DIR);
    if !dir.try_exists()? {
        return Ok(Vec::new());
    }
    if !fs::symlink_metadata(&dir)?.file_type().is_dir() {
        return Err(text(
            "回收站路径必须是普通目录",
            "The trash path must be a regular directory",
        )
        .into());
    }
    let mut entries = Vec::new();
    for item in fs::read_dir(dir)? {
        let item = item?;
        if !item.file_type()?.is_file() {
            return Err(text("回收站包含非普通文件", "Trash contains a non-regular file").into());
        }
        let name = item.file_name();
        let name = name
            .to_str()
            .ok_or_else(|| text("回收站文件名无效", "Invalid trash filename"))?;
        let id = name
            .strip_suffix(".json")
            .ok_or_else(|| text("回收站包含未知文件", "Trash contains an unknown file"))?;
        let (entry, note) = serde_json::from_slice::<Entry>(&fs::read(item.path())?)?.validate()?;
        if id != note.meta.id {
            return Err(text(
                "回收站文件名与灵感 ID 不一致",
                "Trash filename does not match the note ID",
            )
            .into());
        }
        entries.push((entry, note));
    }
    entries.sort_unstable_by(|a, b| b.0.deleted_at.cmp(&a.0.deleted_at));
    Ok(entries)
}

pub fn find(store: &Store, prefix: &str) -> Result<(Entry, Note)> {
    let mut matches = list(store)?
        .into_iter()
        .filter(|(_, note)| note.meta.id.starts_with(prefix));
    let first = matches
        .next()
        .ok_or_else(|| text("回收站中没有匹配的灵感", "No matching note in trash"))?;
    if matches.next().is_some() {
        return Err(text(
            "ID 前缀不唯一，请输入更多字符",
            "The ID prefix is ambiguous; enter more characters",
        )
        .into());
    }
    Ok(first)
}

pub fn restore(store: &Store, prefix: &str) -> Result<Note> {
    store.require_exclusive()?;
    let (_, note) = find(store, prefix)?;
    match active_note(store, &note.meta.id)? {
        Some(active) if same_note(&active, &note) => (),
        Some(_) => {
            return Err(text(
                "已有相同 ID 但内容不同的灵感",
                "An active note has the same ID but different content",
            )
            .into());
        }
        None => {
            store.add(&note)?;
        }
    }
    fs::remove_file(path(&store.root, &note.meta.id))?;
    sync_dir(&store.root.join(DIR))?;
    Ok(note)
}

fn active_note(store: &Store, id: &str) -> Result<Option<Note>> {
    let mut found = None;
    for path in store.day_files()? {
        store.scan_file(&path, |note| {
            if note.meta.id == id {
                if found.is_some() {
                    return Err(crate::message!("重复的 ID：{id}", "Duplicate ID: {id}").into());
                }
                found = Some(note);
            }
            Ok(())
        })?;
    }
    Ok(found)
}

pub fn empty(store: &Store, ids: &[String]) -> Result<usize> {
    store.require_exclusive()?;
    let dir = store.root.join(DIR);
    for id in ids {
        let file = path(&store.root, id);
        if !file.try_exists()? || !fs::symlink_metadata(&file)?.file_type().is_file() {
            return Err(text(
                "确认后回收站内容已改变，未执行清空",
                "Trash changed after confirmation; nothing was emptied",
            )
            .into());
        }
    }
    for id in ids {
        fs::remove_file(path(&store.root, id))?;
    }
    if dir.try_exists()? {
        sync_dir(&dir)?;
    }
    Ok(ids.len())
}

pub(crate) fn ensure_trash(root: &Path) -> Result<PathBuf> {
    let dir = root.join(DIR);
    ensure_dir(&dir)?;
    if !fs::symlink_metadata(&dir)?.file_type().is_dir() {
        return Err(text(
            "回收站路径必须是普通目录",
            "The trash path must be a regular directory",
        )
        .into());
    }
    Ok(dir)
}

fn same_note(a: &Note, b: &Note) -> bool {
    a.meta.id == b.meta.id
        && a.meta.created == b.meta.created
        && a.meta.tags == b.meta.tags
        && a.content == b.content
}
