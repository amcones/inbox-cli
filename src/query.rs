use crate::{Result, cli::Sort, model::Note, ranking, storage::Store, views};
use jiff::{Timestamp, civil::Date, tz::TimeZone};
use std::{
    cmp::{Ordering, Reverse},
    collections::{BTreeMap, BinaryHeap, HashSet},
};

struct Ranked {
    score: f64,
    note: Note,
}

impl PartialEq for Ranked {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}
impl Eq for Ranked {}
impl PartialOrd for Ranked {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Ranked {
    fn cmp(&self, other: &Self) -> Ordering {
        self.score
            .total_cmp(&other.score)
            .then_with(|| self.note.timestamp.cmp(&other.note.timestamp))
            .then_with(|| self.note.meta.id.cmp(&other.note.meta.id))
    }
}

pub fn list(
    store: &Store,
    tags: &[String],
    any: bool,
    sort: Sort,
    limit: usize,
    now: Timestamp,
) -> Result<Vec<Note>> {
    select(store, tags, any, sort, limit, now, None)
}

pub fn search(
    store: &Store,
    query: &str,
    tags: &[String],
    any: bool,
    sort: Sort,
    limit: usize,
    now: Timestamp,
) -> Result<Vec<Note>> {
    let query = query.to_lowercase();
    select(store, tags, any, sort, limit, now, Some(&query))
}

fn select(
    store: &Store,
    tags: &[String],
    any: bool,
    sort: Sort,
    limit: usize,
    now: Timestamp,
    query: Option<&str>,
) -> Result<Vec<Note>> {
    if limit == 0 {
        return Ok(Vec::new());
    }
    let counts = if sort == Sort::Priority {
        views::counts(store)?
    } else {
        Default::default()
    };
    let mut top = BinaryHeap::<Reverse<Ranked>>::new();
    for path in store.day_files()? {
        if sort == Sort::Time && top.len() == limit {
            let date: Date = path.file_stem().unwrap().to_str().unwrap().parse()?;
            // Civil days can overlap in UTC after a time-zone change. The
            // timestamp grammar allows offsets up to 25:59:59, so 50 hours
            // after midnight is a conservative upper bound for this file.
            let upper = date
                .at(0, 0, 0, 0)
                .to_zoned(TimeZone::UTC)?
                .timestamp()
                .as_second()
                + 50 * 3600;
            if top.peek().unwrap().0.note.timestamp.as_second() > upper {
                break;
            }
        }
        store.scan_file(&path, |note| {
            if note.matches(tags, any)
                && query.is_none_or(|query| note.content.to_lowercase().contains(query))
            {
                let score = match sort {
                    Sort::Time => 0.0,
                    Sort::Priority => ranking::score(
                        note.timestamp,
                        *counts.get(&note.meta.id).unwrap_or(&0),
                        now,
                    ),
                };
                let ranked = Ranked { score, note };
                if top.len() < limit {
                    top.push(Reverse(ranked));
                } else if ranked > top.peek().unwrap().0 {
                    *top.peek_mut().unwrap() = Reverse(ranked);
                }
            }
            Ok(())
        })?;
    }
    Ok(top
        .into_sorted_vec()
        .into_iter()
        .map(|r| r.0.note)
        .collect())
}

pub fn find(store: &Store, prefix: &str) -> Result<Note> {
    let mut found = None;
    for path in store.day_files()? {
        store.scan_file(&path, |note| {
            if note.meta.id.starts_with(prefix) {
                if found.is_some() {
                    return Err(crate::message!("ID 前缀 {prefix} 匹配多条记录，请提供更长的 ID；若完整 ID 重复，请运行 doctor", "ID prefix {prefix} matches multiple notes; use a longer ID, or run doctor if full IDs are duplicated").into());
                }
                found = Some(note);
            }
            Ok(())
        })?;
    }
    found.ok_or_else(|| {
        crate::message!(
            "没有找到 ID 为 {prefix} 的记录",
            "No note found for ID {prefix}"
        )
        .into()
    })
}

pub fn tags(store: &Store) -> Result<BTreeMap<String, u64>> {
    let mut counts = BTreeMap::new();
    for path in store.day_files()? {
        store.scan_file(&path, |note| {
            for tag in note.meta.tags {
                *counts.entry(tag).or_insert(0) += 1;
            }
            Ok(())
        })?;
    }
    Ok(counts)
}

pub fn doctor(store: &Store) -> Result<(usize, usize, u64)> {
    let mut ids = HashSet::new();
    let files = store.day_files()?;
    for path in &files {
        store.scan_file(path, |note| {
            if !ids.insert(note.meta.id.clone()) {
                return Err(
                    crate::message!("重复的 ID：{}", "Duplicate ID: {}", note.meta.id).into(),
                );
            }
            Ok(())
        })?;
    }
    let counts = views::counts(store)?;
    for id in counts.keys() {
        if !ids.contains(id) {
            return Err(crate::message!(
                "浏览日志引用了不存在的记录：{id}",
                "The view log references a missing note: {id}"
            )
            .into());
        }
    }
    Ok((
        files.len(),
        ids.len(),
        counts.values().fold(0u64, |sum, n| sum.saturating_add(*n)),
    ))
}
