use crate::{Result, model::normalize_tags};
use lexopt::Arg::{Long, Short, Value};
use lexopt::ValueExt;
use std::{ffi::OsString, path::PathBuf};

pub const HELP_ZH: &str = "inbox — 随手记录，本地 Markdown 保存

用法:
  inbox add '内容' [-t 标签]...      记录灵感（内容为 - 时读取标准输入）
  inbox edit <ID前缀> ['内容']       编辑正文（内容为 - 时读取标准输入）
  inbox edit <ID前缀> -t 标签...    替换标签（--clear-tags 清空标签）
  inbox [list] [-t 标签]...         最近的灵感，默认 20 条
  inbox list --sort priority       按隐藏优先级排序
  inbox review [-n 数量]           回顾优先候选，默认 5 条
  inbox search <关键词> [-t 标签]... 搜索完整正文
  inbox show <ID前缀> [--no-track]  查看完整内容；默认计一次浏览
  inbox delete <ID前缀>            无标签时移入回收站；指定 -t 时删除这些标签
  inbox delete today [--yes]       确认后将今天的灵感移入回收站
  inbox delete range [-y 年] [-m 月] [-d 日] [-h 开始 结束]
                                    按当天小时范围移入回收站（默认当前日期、00 24）
  inbox delete all [--yes]         确认后将全部灵感移入回收站
  inbox trash                     查看回收站
  inbox restore <ID前缀>           恢复灵感
  inbox backup <目录>              创建完整、可校验的备份
  inbox backup verify <目录>       只读检查完整备份
  inbox restore --from <备份目录>  确认后还原完整备份
  inbox trash empty [--yes]        确认后永久清空回收站
  inbox tags                      标签及记录数量
  inbox info                      显示版本、数据目录和统计
  inbox doctor                    检查全部记录和浏览日志
  inbox help                      显示帮助

选项:
  -t, --tag <标签>       添加或筛选标签，可重复；英文统一为小写
      --clear-tags      编辑时清空全部标签
  -n, --limit <数量>     列表数量，必须大于 0
      --any             多标签匹配任意一个（默认全部匹配）
      --sort <方式>     time（默认）或 priority
      --no-track        show 不增加浏览次数
      --from <目录>      restore 使用的完整备份目录
  -y, --yes             跳过删除或完整还原确认（range 中 -y 表示年份）
      --dir <目录>      数据目录（优先于 INBOX_DIR，默认 ~/inbox）
      --lang <语言>     auto（默认）、zh（中文）或 en（英文）
  -V, --version         显示版本

列表、搜索和回顾不计浏览次数；搜索不区分大小写；正文里的 #文字不会自动成为标签。
将 INBOX_SHOW_ELAPSED 或 INBOX_REVIEW_REASONS 设为 off 可隐藏对应显示内容。
";

pub const HELP_EN: &str = "inbox — Capture ideas in local Markdown files

Usage:
  inbox add 'content' [-t tag]...   Add an idea (content - reads stdin)
  inbox edit <ID-prefix> ['content'] Edit content (content - reads stdin)
  inbox edit <ID-prefix> -t tag...  Replace tags (--clear-tags removes all)
  inbox [list] [-t tag]...         Recent ideas, default 20
  inbox list --sort priority       Sort by hidden priority
  inbox review [-n count]          Review priority candidates, default 5
  inbox search <query> [-t tag]... Search complete note bodies
  inbox show <ID-prefix> [--no-track] Show full content; counts one view
  inbox delete <ID-prefix>         Move to trash, or remove the supplied tags
  inbox delete today [--yes]       Move today's ideas to trash after confirmation
  inbox delete range [-y year] [-m month] [-d day] [-h start end]
                                    Move a day's hour range (defaults: today, 00 24)
  inbox delete all [--yes]         Move all ideas to trash after confirmation
  inbox trash                     List trashed ideas
  inbox restore <ID-prefix>        Restore an idea
  inbox backup <directory>         Create a complete, verified backup
  inbox backup verify <directory>  Verify a complete backup read-only
  inbox restore --from <backup>    Restore a complete backup after confirmation
  inbox trash empty [--yes]        Permanently empty trash after confirmation
  inbox tags                      Tags and note counts
  inbox info                      Show version, data directory, and statistics
  inbox doctor                    Check all notes and the view log
  inbox help                      Show help

Options:
  -t, --tag <tag>         Add/filter a tag; repeatable; ASCII is lowercased
      --clear-tags       Remove all tags while editing
  -n, --limit <count>     List limit, must be positive
      --any              Match any supplied tag (default: all)
      --sort <order>     time (default) or priority
      --no-track         Do not count this show as a view
      --from <directory> Complete backup used by restore
  -y, --yes              Skip deletion or full-restore confirmation
      --dir <directory>  Overrides INBOX_DIR; default ~/inbox
      --lang <language>  auto (default), zh (Chinese), or en (English)
  -V, --version          Show version

Lists, searches, and reviews do not count as views. Search is case-insensitive. #words in content do not become tags.
Set INBOX_SHOW_ELAPSED or INBOX_REVIEW_REASONS to off to hide those display details.
";

pub fn help() -> &'static str {
    crate::i18n::text(HELP_ZH, HELP_EN)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sort {
    Time,
    Priority,
}

#[derive(Debug)]
pub enum Command {
    Add {
        content: String,
        tags: Vec<String>,
    },
    Edit {
        prefix: String,
        content: Option<String>,
        tags: Option<Vec<String>>,
    },
    List {
        tags: Vec<String>,
        any: bool,
        sort: Sort,
        limit: usize,
    },
    Search {
        query: String,
        tags: Vec<String>,
        any: bool,
        sort: Sort,
        limit: usize,
    },
    Review {
        limit: usize,
    },
    Show {
        prefix: String,
        track: bool,
    },
    Delete {
        target: DeleteTarget,
        yes: bool,
    },
    DeleteTags {
        prefix: String,
        tags: Vec<String>,
    },
    Trash {
        empty: bool,
        yes: bool,
    },
    Restore {
        prefix: String,
    },
    Backup {
        destination: PathBuf,
    },
    VerifyBackup {
        source: PathBuf,
    },
    RestoreBackup {
        source: PathBuf,
        yes: bool,
    },
    Tags,
    Info,
    Doctor,
    Complete {
        kind: CompletionKind,
    },
    Help,
    Version,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompletionKind {
    ActiveIds,
    TrashIds,
    Tags,
}

#[derive(Debug)]
pub enum DeleteTarget {
    Id(String),
    Today,
    Range {
        year: Option<u16>,
        month: Option<u8>,
        day: Option<u8>,
        start_hour: u8,
        end_hour: u8,
    },
    All,
}

#[derive(Debug)]
pub struct Cli {
    pub dir: Option<PathBuf>,
    pub command: Command,
}

pub fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Cli> {
    let args: Vec<OsString> = args.into_iter().collect();
    let range_mode = args
        .windows(2)
        .any(|pair| pair[0] == "delete" && pair[1] == "range");
    let mut parser = lexopt::Parser::from_args(args);
    let mut dir = None;
    let mut tags = Vec::new();
    let mut tags_seen = false;
    let mut positional = Vec::new();
    let mut any = false;
    let mut sort = None;
    let mut limit = None;
    let mut no_track = false;
    let mut yes = false;
    let mut clear_tags = false;
    let mut from = None;
    let mut range_year = None;
    let mut range_month = None;
    let mut range_day = None;
    let mut range_hours = None;
    let mut lang_seen = false;
    while let Some(arg) = parser.next()? {
        match arg {
            Long("lang") => {
                if lang_seen {
                    return Err(crate::i18n::text(
                        "--lang 只能指定一次",
                        "--lang may only be specified once",
                    )
                    .into());
                }
                lang_seen = true;
                let value = parser.value()?.string()?;
                if !matches!(value.as_str(), "auto" | "zh" | "en") {
                    return Err(crate::i18n::text(
                        "语言只支持 auto、zh 或 en",
                        "Language must be auto, zh, or en",
                    )
                    .into());
                }
            }
            Short('h') if range_mode => {
                if range_hours.is_some() {
                    return Err(crate::i18n::text(
                        "-h 只能指定一次",
                        "-h may only be specified once",
                    )
                    .into());
                }
                let start = parser.value()?.string()?.parse::<u8>()?;
                let end = parser.value()?.string()?.parse::<u8>()?;
                if start >= end || start > 23 || end > 24 {
                    return Err(crate::i18n::text(
                        "小时范围需满足 0 <= 开始 < 结束 <= 24",
                        "The hour range must satisfy 0 <= start < end <= 24",
                    )
                    .into());
                }
                range_hours = Some((start, end));
            }
            Short('y') if range_mode => {
                if range_year.is_some() {
                    return Err(crate::i18n::text(
                        "-y 只能指定一次",
                        "-y may only be specified once",
                    )
                    .into());
                }
                let value = parser.value()?.string()?.parse::<u16>()?;
                let year = if value < 100 { 2000 + value } else { value };
                if year == 0 || year > 9999 {
                    return Err(crate::i18n::text(
                        "年份必须为 00–99 或 1–9999",
                        "The year must be 00–99 or 1–9999",
                    )
                    .into());
                }
                range_year = Some(year);
            }
            Short('m') if range_mode => {
                if range_month.is_some() {
                    return Err(crate::i18n::text(
                        "-m 只能指定一次",
                        "-m may only be specified once",
                    )
                    .into());
                }
                let value = parser.value()?.string()?.parse::<u8>()?;
                if !(1..=12).contains(&value) {
                    return Err(
                        crate::i18n::text("月份必须为 1–12", "The month must be 1–12").into(),
                    );
                }
                range_month = Some(value);
            }
            Short('d') if range_mode => {
                if range_day.is_some() {
                    return Err(crate::i18n::text(
                        "-d 只能指定一次",
                        "-d may only be specified once",
                    )
                    .into());
                }
                let value = parser.value()?.string()?.parse::<u8>()?;
                if !(1..=31).contains(&value) {
                    return Err(crate::i18n::text("日期必须为 1–31", "The day must be 1–31").into());
                }
                range_day = Some(value);
            }
            Short('V') | Long("version") => {
                return Ok(Cli {
                    dir,
                    command: Command::Version,
                });
            }
            Long("dir") => {
                if dir.is_some() {
                    return Err(crate::i18n::text(
                        "--dir 只能指定一次",
                        "--dir may only be specified once",
                    )
                    .into());
                }
                let path = parser.value()?;
                if path.is_empty() {
                    return Err(crate::i18n::text(
                        "数据目录不能为空",
                        "The data directory must not be empty",
                    )
                    .into());
                }
                dir = Some(PathBuf::from(path));
            }
            Short('t') | Long("tag") => {
                tags_seen = true;
                tags.push(parser.value()?.string()?);
            }
            Long("clear-tags") => clear_tags = true,
            Long("from") => {
                if from.is_some() {
                    return Err(crate::i18n::text(
                        "--from 只能指定一次",
                        "--from may only be specified once",
                    )
                    .into());
                }
                let path = parser.value()?;
                if path.is_empty() {
                    return Err(crate::i18n::text(
                        "备份目录不能为空",
                        "The backup directory must not be empty",
                    )
                    .into());
                }
                from = Some(PathBuf::from(path));
            }
            Long("any") => any = true,
            Long("no-track") => no_track = true,
            Short('y') | Long("yes") => yes = true,
            Short('n') | Long("limit") => {
                if limit.is_some() {
                    return Err(crate::i18n::text(
                        "--limit 只能指定一次",
                        "--limit may only be specified once",
                    )
                    .into());
                }
                let n = parser.value()?.string()?.parse::<usize>()?;
                if n == 0 {
                    return Err(crate::i18n::text(
                        "--limit 必须大于 0",
                        "--limit must be greater than 0",
                    )
                    .into());
                }
                limit = Some(n);
            }
            Long("sort") => {
                if sort.is_some() {
                    return Err(crate::i18n::text(
                        "--sort 只能指定一次",
                        "--sort may only be specified once",
                    )
                    .into());
                }
                sort = Some(match parser.value()?.string()?.as_str() {
                    "time" => Sort::Time,
                    "priority" => Sort::Priority,
                    _ => {
                        return Err(crate::i18n::text(
                            "--sort 只支持 time 或 priority",
                            "--sort must be time or priority",
                        )
                        .into());
                    }
                });
            }
            Value(value) => positional.push(value.string()?),
            arg => return Err(arg.unexpected().into()),
        }
    }
    let tags = normalize_tags(tags)?;
    let list_options = any || sort.is_some() || limit.is_some();
    let unrelated = list_options || no_track || tags_seen || clear_tags;
    if from.is_some() && positional.first().map(String::as_str) != Some("restore") {
        return Err(crate::i18n::text(
            "--from 仅用于 restore",
            "--from is only valid with restore",
        )
        .into());
    }
    let command = match positional.first().map(String::as_str).unwrap_or("list") {
        "add" if positional.len() == 2 => {
            reject(
                list_options || no_track || yes || clear_tags,
                "add 不接受该选项",
                "This option is not valid with add",
            )?;
            Command::Add {
                content: positional.pop().unwrap(),
                tags,
            }
        }
        "edit" if matches!(positional.len(), 2 | 3) => {
            reject(
                list_options || no_track || yes,
                "edit 不接受列表、浏览或确认选项",
                "List, tracking, and confirmation options are not valid with edit",
            )?;
            if clear_tags && tags_seen {
                return Err(crate::i18n::text(
                    "--clear-tags 不能与 -t 同时使用",
                    "--clear-tags cannot be combined with -t",
                )
                .into());
            }
            let content = if positional.len() == 3 {
                Some(positional.pop().unwrap())
            } else {
                None
            };
            if content.is_none() && !tags_seen && !clear_tags {
                return Err(crate::i18n::text(
                    "edit 需要新正文、-t 或 --clear-tags",
                    "edit requires new content, -t, or --clear-tags",
                )
                .into());
            }
            let prefix = positional.pop().unwrap().to_ascii_lowercase();
            validate_id_prefix(&prefix)?;
            Command::Edit {
                prefix,
                content,
                tags: if tags_seen {
                    Some(tags)
                } else if clear_tags {
                    Some(Vec::new())
                } else {
                    None
                },
            }
        }
        "list" if positional.len() <= 1 => {
            if no_track || yes || clear_tags {
                return Err(crate::i18n::text(
                    "--no-track 仅用于 show",
                    "--no-track is only valid with show",
                )
                .into());
            }
            if any && tags.is_empty() {
                return Err(crate::i18n::text(
                    "--any 需要至少一个 -t 标签",
                    "--any requires at least one -t tag",
                )
                .into());
            }
            Command::List {
                tags,
                any,
                sort: sort.unwrap_or(Sort::Time),
                limit: limit.unwrap_or(20),
            }
        }
        "search" if positional.len() == 2 => {
            if no_track || yes || clear_tags {
                return Err(crate::i18n::text(
                    "--no-track 和 --yes 不能用于 search",
                    "--no-track and --yes are not valid with search",
                )
                .into());
            }
            if any && tags.is_empty() {
                return Err(crate::i18n::text(
                    "--any 需要至少一个 -t 标签",
                    "--any requires at least one -t tag",
                )
                .into());
            }
            let query = positional.pop().unwrap();
            if query.trim().is_empty() {
                return Err(crate::i18n::text(
                    "搜索关键词不能为空",
                    "The search query must not be empty",
                )
                .into());
            }
            Command::Search {
                query,
                tags,
                any,
                sort: sort.unwrap_or(Sort::Time),
                limit: limit.unwrap_or(20),
            }
        }
        "review" if positional.len() == 1 => {
            reject(
                any || sort.is_some() || no_track || yes || tags_seen || clear_tags,
                "review 仅接受 --limit",
                "review only accepts --limit",
            )?;
            Command::Review {
                limit: limit.unwrap_or(5),
            }
        }
        "show" | "delete"
            if positional.len() == 2
                && !(positional[0] == "delete" && positional[1] == "range") =>
        {
            if list_options || clear_tags || (positional[0] == "show" && tags_seen) {
                return Err(crate::i18n::text(
                    "show/delete 不接受列表或标签选项",
                    "show does not accept list or tag options",
                )
                .into());
            }
            let value = positional.pop().unwrap().to_ascii_lowercase();
            if positional[0] == "delete" {
                if no_track {
                    return Err(crate::i18n::text(
                        "--no-track 仅用于 show",
                        "--no-track is only valid with show",
                    )
                    .into());
                }
                let target = match value.as_str() {
                    "today" => DeleteTarget::Today,
                    "all" => DeleteTarget::All,
                    _ => {
                        validate_id_prefix(&value)?;
                        if yes {
                            return Err(crate::i18n::text(
                                "--yes 仅用于 delete today/range/all",
                                "--yes is only valid with delete today/range/all",
                            )
                            .into());
                        }
                        if tags_seen {
                            if tags.is_empty() {
                                return Err(crate::i18n::text(
                                    "至少需要一个标签",
                                    "At least one tag is required",
                                )
                                .into());
                            }
                            return Ok(Cli {
                                dir,
                                command: Command::DeleteTags {
                                    prefix: value,
                                    tags,
                                },
                            });
                        }
                        DeleteTarget::Id(value)
                    }
                };
                if tags_seen {
                    return Err(crate::i18n::text(
                        "批量删除不能指定标签",
                        "Bulk deletion cannot specify tags",
                    )
                    .into());
                }
                Command::Delete { target, yes }
            } else {
                if yes {
                    return Err(crate::i18n::text(
                        "--yes 仅用于 delete today/range/all",
                        "--yes is only valid with delete today/range/all",
                    )
                    .into());
                }
                validate_id_prefix(&value)?;
                Command::Show {
                    prefix: value,
                    track: !no_track,
                }
            }
        }
        "delete" if positional.len() == 2 && positional[1] == "range" => {
            reject(
                list_options || no_track || tags_seen || clear_tags,
                "delete range 不接受列表、标签或浏览选项",
                "List, tag, and tracking options are not valid with delete range",
            )?;
            let (start_hour, end_hour) = range_hours.unwrap_or((0, 24));
            if let (Some(year), Some(month), Some(day)) = (range_year, range_month, range_day) {
                let date = format!("{year:04}-{month:02}-{day:02}");
                if date.parse::<jiff::civil::Date>().is_err() {
                    return Err(
                        crate::i18n::text("指定日期无效", "The specified date is invalid").into(),
                    );
                }
            }
            Command::Delete {
                target: DeleteTarget::Range {
                    year: range_year,
                    month: range_month,
                    day: range_day,
                    start_hour,
                    end_hour,
                },
                yes,
            }
        }
        "trash" if positional.len() == 1 || (positional.len() == 2 && positional[1] == "empty") => {
            reject(
                list_options || no_track || tags_seen || clear_tags,
                "trash 不接受列表、标签或浏览选项",
                "List, tag, and tracking options are not valid with trash",
            )?;
            let empty = positional.len() == 2;
            if yes && !empty {
                return Err(crate::i18n::text(
                    "--yes 仅用于 trash empty 或批量删除",
                    "--yes is only valid with trash empty or bulk deletion",
                )
                .into());
            }
            Command::Trash { empty, yes }
        }
        "backup" if positional.len() == 2 && positional[1] != "verify" => {
            reject(
                unrelated || yes || from.is_some(),
                "backup 不接受其它选项",
                "backup does not accept these options",
            )?;
            Command::Backup {
                destination: PathBuf::from(positional.pop().unwrap()),
            }
        }
        "backup" if positional.len() == 3 && positional[1] == "verify" => {
            reject(
                unrelated || yes || from.is_some(),
                "backup verify 不接受其它选项",
                "backup verify does not accept these options",
            )?;
            Command::VerifyBackup {
                source: PathBuf::from(positional.pop().unwrap()),
            }
        }
        "restore" if positional.len() == 1 && from.is_some() => {
            reject(
                unrelated,
                "完整还原仅接受 --from 和 --yes",
                "Full restore only accepts --from and --yes",
            )?;
            Command::RestoreBackup {
                source: from.take().unwrap(),
                yes,
            }
        }
        "restore" if positional.len() == 2 && from.is_none() => {
            reject(
                list_options || no_track || yes || tags_seen || clear_tags,
                "restore 不接受其它选项",
                "restore does not accept these options",
            )?;
            let prefix = positional.pop().unwrap().to_ascii_lowercase();
            validate_id_prefix(&prefix)?;
            Command::Restore { prefix }
        }
        "tags" | "info" | "doctor" if positional.len() == 1 => {
            if list_options || no_track || yes || tags_seen || clear_tags || from.is_some() {
                return Err(crate::i18n::text(
                    "该子命令不接受列表、标签或浏览选项",
                    "This subcommand does not accept list, tag, or tracking options",
                )
                .into());
            }
            match positional[0].as_str() {
                "tags" => Command::Tags,
                "info" => Command::Info,
                _ => Command::Doctor,
            }
        }
        "help" if positional.len() == 1 => {
            reject(
                list_options || no_track || yes || tags_seen || clear_tags || from.is_some(),
                "help 不接受其它选项",
                "help does not accept these options",
            )?;
            Command::Help
        }
        "__complete" if positional.len() == 2 => {
            reject(
                list_options || no_track || yes || tags_seen || clear_tags || from.is_some(),
                "内部补全命令不接受其它选项",
                "The internal completion command does not accept options",
            )?;
            let kind = match positional[1].as_str() {
                "active-ids" => CompletionKind::ActiveIds,
                "trash-ids" => CompletionKind::TrashIds,
                "tags" => CompletionKind::Tags,
                _ => {
                    return Err(crate::i18n::text(
                        "未知的内部补全类型",
                        "Unknown internal completion kind",
                    )
                    .into());
                }
            };
            Command::Complete { kind }
        }
        _ => {
            return Err(crate::i18n::text(
                "未知命令或缺少参数；运行 inbox help 查看用法",
                "Unknown command or missing argument; run inbox help for usage",
            )
            .into());
        }
    };
    Ok(Cli { dir, command })
}

fn reject(condition: bool, zh: &'static str, en: &'static str) -> Result<()> {
    if condition {
        Err(crate::i18n::text(zh, en).into())
    } else {
        Ok(())
    }
}

fn validate_id_prefix(prefix: &str) -> Result<()> {
    if prefix.len() < 4
        || prefix.len() > 36
        || !prefix.bytes().all(|b| b.is_ascii_hexdigit() || b == b'-')
    {
        return Err(crate::i18n::text(
            "ID 前缀需为 4–36 个十六进制字符或连字符",
            "An ID prefix must contain 4–36 hexadecimal characters or hyphens",
        )
        .into());
    }
    Ok(())
}

pub fn data_dir(explicit: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(path) = explicit {
        return Ok(path);
    }
    if let Some(path) = std::env::var_os("INBOX_DIR") {
        if path.is_empty() {
            return Err(
                crate::i18n::text("INBOX_DIR 不能为空", "INBOX_DIR must not be empty").into(),
            );
        }
        return Ok(path.into());
    }
    let home = std::env::var_os("HOME")
        .filter(|s| !s.is_empty())
        .ok_or(crate::i18n::text(
            "找不到 HOME，请使用 --dir 指定数据目录",
            "HOME is unavailable; use --dir to specify the data directory",
        ))?;
    Ok(PathBuf::from(home).join("inbox"))
}
