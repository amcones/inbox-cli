use crate::Result;
use jiff::{Timestamp, Zoned};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const MAX_CONTENT_BYTES: usize = 1024 * 1024;
pub const MAX_TAGS: usize = 64;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Metadata {
    pub id: String,
    pub created: String,
    pub tags: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct Note {
    pub meta: Metadata,
    pub timestamp: Timestamp,
    pub content: String,
}

impl Note {
    pub fn new(content: String, tags: Vec<String>, now: &Zoned) -> Result<Self> {
        // Check the raw input before CRLF normalization. stdin is read with a
        // MAX+1 sentinel; normalizing first could hide that it was truncated.
        let content = normalize_content(content)?;
        let created = format!(
            "{}.{:09}{}",
            now.strftime("%Y-%m-%dT%H:%M:%S"),
            now.subsec_nanosecond(),
            now.strftime("%:z")
        );
        let timestamp = created.parse()?;
        Ok(Self {
            meta: Metadata {
                id: Uuid::new_v4().to_string(),
                created,
                tags: normalize_tags(tags)?,
            },
            timestamp,
            content,
        })
    }

    pub fn short_id(&self) -> &str {
        &self.meta.id[..8]
    }

    pub fn matches(&self, tags: &[String], any: bool) -> bool {
        tags.is_empty()
            || if any {
                tags.iter().any(|tag| self.meta.tags.contains(tag))
            } else {
                tags.iter().all(|tag| self.meta.tags.contains(tag))
            }
    }

    pub fn updated(&self, content: Option<String>, tags: Option<Vec<String>>) -> Result<Self> {
        Ok(Self {
            meta: Metadata {
                id: self.meta.id.clone(),
                created: self.meta.created.clone(),
                tags: match tags {
                    Some(tags) => normalize_tags(tags)?,
                    None => self.meta.tags.clone(),
                },
            },
            timestamp: self.timestamp,
            content: match content {
                Some(content) => normalize_content(content)?,
                None => self.content.clone(),
            },
        })
    }
}

pub fn normalize_content(content: String) -> Result<String> {
    if content.len() > MAX_CONTENT_BYTES {
        return Err(
            crate::i18n::text("单条内容不能超过 1 MiB", "A note must not exceed 1 MiB").into(),
        );
    }
    let content = content.replace("\r\n", "\n");
    if content.trim().is_empty() {
        return Err(crate::i18n::text("内容不能为空", "Content must not be empty").into());
    }
    if content.contains('\0') || content.contains('\r') {
        return Err(crate::i18n::text(
            "内容不能包含 NUL 或单独的回车字符",
            "Content must not contain NUL or standalone carriage returns",
        )
        .into());
    }
    Ok(content)
}

pub fn normalize_tags(tags: Vec<String>) -> Result<Vec<String>> {
    let mut out = Vec::new();
    for tag in tags {
        let tag = tag.trim().trim_start_matches('#');
        if tag.is_empty()
            || tag.len() > 128
            || tag.chars().any(|c| c.is_whitespace() || c.is_control())
        {
            return Err(crate::i18n::text(
                "标签需为 1–128 字节，不能包含空白或控制字符",
                "Tags must be 1–128 bytes and contain no whitespace or control characters",
            )
            .into());
        }
        let tag = tag.to_ascii_lowercase();
        if !out.contains(&tag) {
            out.push(tag);
        }
    }
    if out.len() > MAX_TAGS {
        return Err(
            crate::i18n::text("每条记录最多 64 个标签", "A note may have at most 64 tags").into(),
        );
    }
    Ok(out)
}

pub fn validate_meta(meta: &Metadata) -> Result<Timestamp> {
    let id = Uuid::parse_str(&meta.id)?;
    if id.to_string() != meta.id {
        return Err(crate::i18n::text(
            "ID 必须是小写、带连字符的 UUID",
            "ID must be a lowercase, hyphenated UUID",
        )
        .into());
    }
    if normalize_tags(meta.tags.clone())? != meta.tags {
        return Err(crate::i18n::text(
            "标签元数据不规范或包含重复标签",
            "Tag metadata is not normalized or contains duplicates",
        )
        .into());
    }
    // Date and HH:MM:SS occupy fixed positions; fractional seconds are optional.
    // An explicit offset is required, and Jiff validates the calendar/time.
    let bytes = meta.created.as_bytes();
    if !bytes.is_ascii()
        || !(bytes.len() == 25 || (27..=35).contains(&bytes.len()))
        || bytes[10] != b'T'
        || !matches!(bytes[bytes.len() - 6], b'+' | b'-')
        || (bytes.len() != 25
            && (bytes[19] != b'.' || !bytes[20..bytes.len() - 6].iter().all(u8::is_ascii_digit)))
    {
        return Err(crate::i18n::text(
            "创建时间需为带 UTC 偏移的 RFC 3339 格式，小数秒最多 9 位",
            "Creation time must be RFC 3339 with a UTC offset and at most 9 fractional digits",
        )
        .into());
    }
    Ok(meta.created.parse()?)
}

/// Keep untrusted note text from issuing terminal control sequences.
pub fn terminal_text(text: &str, multiline: bool) -> String {
    text.chars()
        .map(|c| {
            if multiline && (c == '\n' || c == '\t') {
                c
            } else if c.is_whitespace() {
                ' '
            } else if c.is_control() {
                '�'
            } else {
                c
            }
        })
        .collect()
}
