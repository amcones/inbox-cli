use crate::{Result, model::normalize_tags};
use lexopt::Arg::{Long, Short, Value};
use lexopt::ValueExt;
use std::{ffi::OsString, path::PathBuf};

pub const HELP_ZH: &str = "inbox — 随手记录，本地 Markdown 保存

用法:
  inbox -m '内容' [-t 标签]...       记录灵感（-m - 从标准输入读取）
  inbox [list] [-t 标签]...         最近的灵感，默认 20 条
  inbox list --sort priority       按隐藏优先级排序
  inbox show <ID前缀> [--no-track]  查看完整内容；默认计一次浏览
  inbox delete <ID前缀>            删除灵感及其浏览记录
  inbox delete today [--yes]       确认后删除今天的全部灵感
  inbox delete all [--yes]         确认后删除全部灵感
  inbox tags                      标签及记录数量
  inbox doctor                    检查全部记录和浏览日志

选项:
  -m, --message <内容>   新增内容，最多 1 MiB
  -t, --tag <标签>       添加或筛选标签，可重复，区分大小写
  -n, --limit <数量>     列表数量，必须大于 0
      --any             多标签匹配任意一个（默认全部匹配）
      --sort <方式>     time（默认）或 priority
      --no-track        show 不增加浏览次数
  -y, --yes             跳过批量删除确认
      --dir <目录>      数据目录（优先于 INBOX_DIR，默认 ~/inbox）
      --lang <语言>     auto（默认）、zh（中文）或 en（英文）
  -h, --help            显示帮助
  -V, --version         显示版本

列表不计浏览次数；正文里的 #文字 不会自动成为标签。
";

pub const HELP_EN: &str = "inbox — Capture ideas in local Markdown files

Usage:
  inbox -m 'content' [-t tag]...    Add an idea (-m - reads stdin)
  inbox [list] [-t tag]...         Recent ideas, default 20
  inbox list --sort priority       Sort by hidden priority
  inbox show <ID-prefix> [--no-track] Show full content; counts one view
  inbox delete <ID-prefix>         Delete an idea and its view history
  inbox delete today [--yes]       Delete today's ideas after confirmation
  inbox delete all [--yes]         Delete all ideas after confirmation
  inbox tags                      Tags and note counts
  inbox doctor                    Check all notes and the view log

Options:
  -m, --message <content> Add content, up to 1 MiB
  -t, --tag <tag>         Add/filter a tag; repeatable, case-sensitive
  -n, --limit <count>     List limit, must be positive
      --any              Match any supplied tag (default: all)
      --sort <order>     time (default) or priority
      --no-track         Do not count this show as a view
  -y, --yes              Skip bulk-delete confirmation
      --dir <directory>  Overrides INBOX_DIR; default ~/inbox
      --lang <language>  auto (default), zh (Chinese), or en (English)
  -h, --help             Show help
  -V, --version          Show version

Lists do not count as views. #words in content do not become tags.
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
    List {
        tags: Vec<String>,
        any: bool,
        sort: Sort,
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
    Tags,
    Doctor,
    Help,
    Version,
}

#[derive(Debug)]
pub enum DeleteTarget {
    Id(String),
    Today,
    All,
}

#[derive(Debug)]
pub struct Cli {
    pub dir: Option<PathBuf>,
    pub command: Command,
}

pub fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Cli> {
    let mut parser = lexopt::Parser::from_args(args);
    let mut dir = None;
    let mut message = None;
    let mut tags = Vec::new();
    let mut positional = Vec::new();
    let mut any = false;
    let mut sort = None;
    let mut limit = None;
    let mut no_track = false;
    let mut yes = false;
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
            Short('h') | Long("help") => {
                return Ok(Cli {
                    dir,
                    command: Command::Help,
                });
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
            Short('m') | Long("message") => {
                if message.is_some() {
                    return Err(crate::i18n::text(
                        "-m 只能指定一次",
                        "-m may only be specified once",
                    )
                    .into());
                }
                message = Some(parser.value()?.string()?);
            }
            Short('t') | Long("tag") => tags.push(parser.value()?.string()?),
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
    let command = if let Some(content) = message {
        if !positional.is_empty() || list_options || no_track || yes {
            return Err(crate::i18n::text(
                "-m 不能与子命令、列表选项或 --no-track 同时使用",
                "-m cannot be combined with subcommands, list options, or --no-track",
            )
            .into());
        }
        Command::Add { content, tags }
    } else {
        match positional.first().map(String::as_str).unwrap_or("list") {
            "list" if positional.len() <= 1 => {
                if no_track || yes {
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
            "show" | "delete" if positional.len() == 2 => {
                if list_options || !tags.is_empty() {
                    return Err(crate::i18n::text(
                        "show/delete 不接受列表或标签选项",
                        "show/delete do not accept list or tag options",
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
                                    "--yes 仅用于 delete today/all",
                                    "--yes is only valid with delete today/all",
                                )
                                .into());
                            }
                            DeleteTarget::Id(value)
                        }
                    };
                    Command::Delete { target, yes }
                } else {
                    if yes {
                        return Err(crate::i18n::text(
                            "--yes 仅用于 delete today/all",
                            "--yes is only valid with delete today/all",
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
            "tags" | "doctor" if positional.len() == 1 => {
                if list_options || no_track || yes || !tags.is_empty() {
                    return Err(crate::i18n::text(
                        "该子命令不接受列表、标签或浏览选项",
                        "This subcommand does not accept list, tag, or tracking options",
                    )
                    .into());
                }
                if positional[0] == "tags" {
                    Command::Tags
                } else {
                    Command::Doctor
                }
            }
            _ => {
                return Err(crate::i18n::text(
                    "未知命令或缺少参数；运行 inbox --help 查看用法",
                    "Unknown command or missing argument; run inbox --help for usage",
                )
                .into());
            }
        }
    };
    Ok(Cli { dir, command })
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
