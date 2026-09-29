use crate::{Result, model::normalize_tags};
use lexopt::Arg::{Long, Short, Value};
use lexopt::ValueExt;
use std::{ffi::OsString, path::PathBuf};

pub const HELP: &str = "inbox — 随手记录，本地 Markdown 保存

用法:
  inbox -m '内容' [-t 标签]...       记录灵感（-m - 从标准输入读取）
  inbox [list] [-t 标签]...         最近的灵感，默认 20 条
  inbox list --sort priority       按隐藏优先级排序
  inbox show <ID前缀> [--no-track]  查看完整内容；默认计一次浏览
  inbox tags                      标签及记录数量
  inbox doctor                    检查全部记录和浏览日志

选项:
  -m, --message <内容>   新增内容，最多 1 MiB
  -t, --tag <标签>       添加或筛选标签，可重复，区分大小写
  -n, --limit <数量>     列表数量，必须大于 0
      --any             多标签匹配任意一个（默认全部匹配）
      --sort <方式>     time（默认）或 priority
      --no-track        show 不增加浏览次数
      --dir <目录>      数据目录（优先于 INBOX_DIR，默认 ~/inbox）
  -h, --help            显示帮助
  -V, --version         显示版本

列表不计浏览次数；正文里的 #文字 不会自动成为标签。
";

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
    Tags,
    Doctor,
    Help,
    Version,
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
    while let Some(arg) = parser.next()? {
        match arg {
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
                    return Err("--dir 只能指定一次".into());
                }
                let path = parser.value()?;
                if path.is_empty() {
                    return Err("数据目录不能为空".into());
                }
                dir = Some(PathBuf::from(path));
            }
            Short('m') | Long("message") => {
                if message.is_some() {
                    return Err("-m 只能指定一次".into());
                }
                message = Some(parser.value()?.string()?);
            }
            Short('t') | Long("tag") => tags.push(parser.value()?.string()?),
            Long("any") => any = true,
            Long("no-track") => no_track = true,
            Short('n') | Long("limit") => {
                if limit.is_some() {
                    return Err("--limit 只能指定一次".into());
                }
                let n = parser.value()?.string()?.parse::<usize>()?;
                if n == 0 {
                    return Err("--limit 必须大于 0".into());
                }
                limit = Some(n);
            }
            Long("sort") => {
                if sort.is_some() {
                    return Err("--sort 只能指定一次".into());
                }
                sort = Some(match parser.value()?.string()?.as_str() {
                    "time" => Sort::Time,
                    "priority" => Sort::Priority,
                    _ => return Err("--sort 只支持 time 或 priority".into()),
                });
            }
            Value(value) => positional.push(value.string()?),
            arg => return Err(arg.unexpected().into()),
        }
    }
    let tags = normalize_tags(tags)?;
    let list_options = any || sort.is_some() || limit.is_some();
    let command = if let Some(content) = message {
        if !positional.is_empty() || list_options || no_track {
            return Err("-m 不能与子命令、列表选项或 --no-track 同时使用".into());
        }
        Command::Add { content, tags }
    } else {
        match positional.first().map(String::as_str).unwrap_or("list") {
            "list" if positional.len() <= 1 => {
                if no_track {
                    return Err("--no-track 仅用于 show".into());
                }
                if any && tags.is_empty() {
                    return Err("--any 需要至少一个 -t 标签".into());
                }
                Command::List {
                    tags,
                    any,
                    sort: sort.unwrap_or(Sort::Time),
                    limit: limit.unwrap_or(20),
                }
            }
            "show" if positional.len() == 2 => {
                if list_options || !tags.is_empty() {
                    return Err("show 不接受列表或标签选项".into());
                }
                let prefix = positional.pop().unwrap().to_ascii_lowercase();
                if prefix.len() < 4
                    || prefix.len() > 36
                    || !prefix.bytes().all(|b| b.is_ascii_hexdigit() || b == b'-')
                {
                    return Err("ID 前缀需为 4–36 个十六进制字符或连字符".into());
                }
                Command::Show {
                    prefix,
                    track: !no_track,
                }
            }
            "tags" | "doctor" if positional.len() == 1 => {
                if list_options || no_track || !tags.is_empty() {
                    return Err("该子命令不接受列表、标签或浏览选项".into());
                }
                if positional[0] == "tags" {
                    Command::Tags
                } else {
                    Command::Doctor
                }
            }
            _ => return Err("未知命令或缺少参数；运行 inbox --help 查看用法".into()),
        }
    };
    Ok(Cli { dir, command })
}

pub fn data_dir(explicit: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(path) = explicit {
        return Ok(path);
    }
    if let Some(path) = std::env::var_os("INBOX_DIR") {
        if path.is_empty() {
            return Err("INBOX_DIR 不能为空".into());
        }
        return Ok(path.into());
    }
    let home = std::env::var_os("HOME")
        .filter(|s| !s.is_empty())
        .ok_or("找不到 HOME，请使用 --dir 指定数据目录")?;
    Ok(PathBuf::from(home).join("inbox"))
}
