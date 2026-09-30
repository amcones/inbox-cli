use inbox::{
    Result,
    cli::{self, Cli, Command, CompletionKind, DeleteTarget},
    deletion, editing, i18n,
    model::{MAX_CONTENT_BYTES, Note, terminal_text},
    query,
    storage::Store,
    trash, views,
};
use std::{
    io::{self, Read, Write},
    process::ExitCode,
};

fn main() -> ExitCode {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let cli = match i18n::configure(&args).and_then(|_| cli::parse(args)) {
        Ok(cli) => cli,
        Err(e) => {
            eprintln!(
                "inbox: {e}\n{}",
                i18n::text("运行 inbox --help 查看用法", "Run inbox --help for usage")
            );
            return ExitCode::from(2);
        }
    };
    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e)
            if e.downcast_ref::<io::Error>()
                .is_some_and(|e| e.kind() == io::ErrorKind::BrokenPipe) =>
        {
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("inbox: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<()> {
    let mut out = io::BufWriter::new(io::stdout().lock());
    match cli.command {
        Command::Help => {
            write!(out, "{}", cli::help())?;
            out.flush()?;
            return Ok(());
        }
        Command::Version => {
            writeln!(out, "inbox {}", env!("CARGO_PKG_VERSION"))?;
            out.flush()?;
            return Ok(());
        }
        _ => (),
    }
    let root = cli::data_dir(cli.dir)?;
    match cli.command {
        Command::Add { mut content, tags } => {
            read_stdin_content(&mut content)?;
            let now = jiff::Timestamp::now().to_zoned(jiff::tz::TimeZone::try_system()?);
            let note = Note::new(content, tags, &now)?;
            let store = Store::open(&root, true)?.unwrap();
            store.add(&note)?;
            drop(store);
            writeln!(out, "{}", note.short_id())?;
        }
        Command::Edit {
            prefix,
            mut content,
            tags,
        } => {
            if let Some(value) = &mut content {
                read_stdin_content(value)?;
            }
            if !root.try_exists()? {
                return Err(i18n::text("inbox 为空", "The inbox is empty").into());
            }
            let store = Store::open(&root, true)?.unwrap();
            let note = editing::edit(&store, &prefix, content, tags)?;
            writeln!(
                out,
                "{}",
                inbox::message!("已编辑 {}", "Edited {}", note.short_id())
            )?;
        }
        Command::List {
            tags,
            any,
            sort,
            limit,
        } => {
            let notes = match Store::open(&root, false)? {
                Some(store) => {
                    query::list(&store, &tags, any, sort, limit, jiff::Timestamp::now())?
                }
                None => Vec::new(),
            };
            for note in notes {
                write_summary(&mut out, &note)?;
            }
        }
        Command::Search {
            query: text,
            tags,
            any,
            sort,
            limit,
        } => {
            let notes = match Store::open(&root, false)? {
                Some(store) => query::search(
                    &store,
                    &text,
                    &tags,
                    any,
                    sort,
                    limit,
                    jiff::Timestamp::now(),
                )?,
                None => Vec::new(),
            };
            for note in notes {
                write_summary(&mut out, &note)?;
            }
        }
        Command::Review { limit } => {
            let now = jiff::Timestamp::now();
            let items = match Store::open(&root, false)? {
                Some(store) => query::review(&store, limit, now)?,
                None => Vec::new(),
            };
            for item in items {
                write_review(&mut out, &item, now)?;
            }
        }
        Command::Show { prefix, track } => {
            // Hold the lock until tracking is complete so deletion cannot
            // remove this note between displaying it and recording the view.
            if !root.try_exists()? {
                return Err(i18n::text("inbox 为空", "The inbox is empty").into());
            }
            let store = Store::open(&root, track)?
                .ok_or(crate::i18n::text("inbox 为空", "The inbox is empty"))?;
            let note = query::find(&store, &prefix)?;
            writeln!(out, "{}  {}", note.meta.id, note.meta.created)?;
            if !note.meta.tags.is_empty() {
                writeln!(
                    out,
                    "{}",
                    note.meta
                        .tags
                        .iter()
                        .map(|t| format!("#{}", terminal_text(t, false)))
                        .collect::<Vec<_>>()
                        .join(" ")
                )?;
            }
            writeln!(out, "\n{}", terminal_text(&note.content, true))?;
            // Do not count a failed/broken-pipe output as a successful view.
            out.flush()?;
            if track {
                views::record(&store, &note.meta.id).map_err(|e| {
                    inbox::message!(
                        "内容已显示，但浏览计数保存失败：{e}",
                        "Content was displayed, but saving the view count failed: {e}"
                    )
                })?;
            }
        }
        Command::Delete { target, yes } => {
            if !root.try_exists()? {
                return Err(i18n::text("inbox 为空", "The inbox is empty").into());
            }
            match target {
                DeleteTarget::Id(prefix) => {
                    let store = Store::open(&root, true)?.unwrap();
                    let plan = deletion::plan_id(&store, &prefix)?;
                    let id = plan.ids[0].clone();
                    deletion::execute(&store, &plan)?;
                    writeln!(
                        out,
                        "{}",
                        inbox::message!("已移入回收站 {id}", "Moved to trash {id}")
                    )?;
                }
                target @ (DeleteTarget::Today | DeleteTarget::Range { .. } | DeleteTarget::All) => {
                    let (plan, label) = {
                        let store = Store::open(&root, false)?.unwrap();
                        match target {
                            DeleteTarget::Today => {
                                let date = jiff::Timestamp::now()
                                    .to_zoned(jiff::tz::TimeZone::try_system()?)
                                    .strftime("%Y-%m-%d")
                                    .to_string();
                                (
                                    deletion::plan_date(&store, &date)?,
                                    inbox::message!("今天（{date}）", "today ({date})"),
                                )
                            }
                            DeleteTarget::All => (
                                deletion::plan_all(&store)?,
                                i18n::text("全部", "all").to_owned(),
                            ),
                            DeleteTarget::Range {
                                year,
                                month,
                                day,
                                start_hour,
                                end_hour,
                            } => {
                                let now = jiff::Timestamp::now()
                                    .to_zoned(jiff::tz::TimeZone::try_system()?);
                                let current = now.date();
                                let date_text = format!(
                                    "{:04}-{:02}-{:02}",
                                    year.unwrap_or(current.year() as u16),
                                    month.unwrap_or(current.month() as u8),
                                    day.unwrap_or(current.day() as u8)
                                );
                                let date = date_text
                                    .parse::<jiff::civil::Date>()
                                    .map_err(|_| {
                                        i18n::text("指定日期无效", "The specified date is invalid")
                                    })?
                                    .to_string();
                                (
                                    deletion::plan_range(&store, &date, start_hour, end_hour)?,
                                    inbox::message!(
                                        "{date} {start_hour:02}:00–{end_hour:02}:00",
                                        "{date} {start_hour:02}:00–{end_hour:02}:00"
                                    ),
                                )
                            }
                            DeleteTarget::Id(_) => unreachable!(),
                        }
                    };
                    if plan.ids.is_empty() {
                        writeln!(
                            out,
                            "{}",
                            inbox::message!("没有可移动的灵感", "No notes to move")
                        )?;
                    } else if yes || confirm_delete(&label, plan.ids.len())? {
                        let store = Store::open(&root, true)?.unwrap();
                        let count = deletion::execute(&store, &plan)?;
                        writeln!(
                            out,
                            "{}",
                            inbox::message!(
                                "已将 {count} 条灵感移入回收站",
                                "Moved {count} notes to trash"
                            )
                        )?;
                    } else {
                        writeln!(
                            out,
                            "{}",
                            i18n::text("已取消，未移动任何灵感", "Cancelled; no notes were moved")
                        )?;
                    }
                }
            }
        }
        Command::DeleteTags { prefix, tags } => {
            if !root.try_exists()? {
                return Err(i18n::text("inbox 为空", "The inbox is empty").into());
            }
            let store = Store::open(&root, true)?.unwrap();
            let original = query::find(&store, &prefix)?;
            let kept = original
                .meta
                .tags
                .iter()
                .filter(|tag| !tags.contains(tag))
                .cloned()
                .collect::<Vec<_>>();
            let removed = original.meta.tags.len() - kept.len();
            if removed == 0 {
                writeln!(
                    out,
                    "{}",
                    inbox::message!(
                        "没有匹配的标签，记录未改变",
                        "No matching tags; the note was unchanged"
                    )
                )?;
            } else {
                let note = editing::edit(&store, &prefix, None, Some(kept))?;
                writeln!(
                    out,
                    "{}",
                    inbox::message!(
                        "已从 {} 删除 {removed} 个标签",
                        "Removed {removed} tags from {}",
                        note.short_id()
                    )
                )?;
            }
        }
        Command::Trash { empty, yes } => {
            if empty {
                let ids = match Store::open(&root, false)? {
                    Some(store) => trash::list(&store)?
                        .into_iter()
                        .map(|(_, note)| note.meta.id)
                        .collect::<Vec<_>>(),
                    None => Vec::new(),
                };
                let count = ids.len();
                if count == 0 {
                    writeln!(out, "{}", i18n::text("回收站为空", "Trash is empty"))?;
                } else if yes || confirm_empty_trash(count)? {
                    let store = Store::open(&root, true)?.unwrap();
                    let count = trash::empty(&store, &ids)?;
                    writeln!(
                        out,
                        "{}",
                        inbox::message!(
                            "已永久删除 {count} 条灵感",
                            "Permanently deleted {count} notes"
                        )
                    )?;
                } else {
                    writeln!(
                        out,
                        "{}",
                        i18n::text("已取消，回收站未改变", "Cancelled; trash was not changed")
                    )?;
                }
            } else if let Some(store) = Store::open(&root, false)? {
                for (entry, note) in trash::list(&store)? {
                    write!(out, "{}  {}  ", note.short_id(), &entry.deleted_at[..19])?;
                    write_summary_body(&mut out, &note)?;
                }
            }
        }
        Command::Restore { prefix } => {
            if !root.try_exists()? {
                return Err(i18n::text("回收站为空", "Trash is empty").into());
            }
            let store = Store::open(&root, true)?.unwrap();
            let note = trash::restore(&store, &prefix)?;
            writeln!(
                out,
                "{}",
                inbox::message!("已恢复 {}", "Restored {}", note.short_id())
            )?;
        }
        Command::Tags => {
            if let Some(store) = Store::open(&root, false)? {
                let tags = query::tags(&store)?;
                drop(store);
                for (tag, count) in tags {
                    writeln!(out, "{}\t{count}", terminal_text(&tag, false))?;
                }
            }
        }
        Command::Doctor => {
            let (files, notes, views, trashed) = match Store::open(&root, false)? {
                Some(store) => {
                    let (files, notes, views) = query::doctor(&store)?;
                    (files, notes, views, trash::list(&store)?.len())
                }
                None => (0, 0, 0, 0),
            };
            writeln!(
                out,
                "{}",
                inbox::message!(
                    "OK: {files} 个日期文件，{notes} 条记录，{views} 次浏览，回收站 {trashed} 条",
                    "OK: {files} day files, {notes} notes, {views} views, {trashed} in trash"
                )
            )?;
        }
        Command::Complete { kind } => {
            if let Some(store) = Store::open(&root, false)? {
                match kind {
                    CompletionKind::ActiveIds => {
                        for note in query::list(
                            &store,
                            &[],
                            false,
                            cli::Sort::Time,
                            100,
                            jiff::Timestamp::now(),
                        )? {
                            writeln!(out, "{}", note.short_id())?;
                        }
                    }
                    CompletionKind::TrashIds => {
                        for (_, note) in trash::list(&store)? {
                            writeln!(out, "{}", note.short_id())?;
                        }
                    }
                    CompletionKind::Tags => {
                        for (tag, _) in query::tags(&store)? {
                            writeln!(out, "{}", terminal_text(&tag, false))?;
                        }
                    }
                }
            }
        }
        Command::Help | Command::Version => unreachable!(),
    }
    out.flush()?;
    Ok(())
}

fn write_summary(out: &mut impl Write, note: &Note) -> Result<()> {
    write!(
        out,
        "{}  {} {}  ",
        note.short_id(),
        &note.meta.created[..10],
        &note.meta.created[11..19]
    )?;
    write_summary_body(out, note)
}

fn write_summary_body(out: &mut impl Write, note: &Note) -> Result<()> {
    let clean = terminal_text(&note.content, false);
    let mut chars = clean.chars();
    let mut summary: String = chars.by_ref().take(80).collect();
    if chars.next().is_some() {
        summary.push('…');
    }
    let tags = note
        .meta
        .tags
        .iter()
        .map(|tag| format!("#{}", terminal_text(tag, false)))
        .collect::<Vec<_>>()
        .join(" ");
    writeln!(
        out,
        "{}{}{}",
        summary,
        if tags.is_empty() { "" } else { "  " },
        tags
    )?;
    Ok(())
}

fn write_review(
    out: &mut impl Write,
    item: &query::ReviewItem,
    now: jiff::Timestamp,
) -> Result<()> {
    write_summary(out, &item.note)?;
    let age_seconds = (now.as_second() - item.note.timestamp.as_second()).max(0);
    let age_days = age_seconds / 86_400;
    let time_reason = if age_days <= 14 {
        i18n::text("近期", "recent").to_owned()
    } else {
        inbox::message!("相对较新（{age_days} 天）", "newer ({age_days} days old)")
    };
    let view_reason = match item.views {
        0 => None,
        1 => Some(i18n::text("看过 1 次", "viewed once").to_owned()),
        count => Some(inbox::message!(
            "常看（{count} 次）",
            "frequently viewed ({count} times)"
        )),
    };
    let reasons = match view_reason {
        Some(reason) => inbox::message!("{time_reason}；{reason}", "{time_reason}; {reason}"),
        None => time_reason,
    };
    writeln!(
        out,
        "  {}",
        inbox::message!("原因：{reasons}", "Reason: {reasons}")
    )?;
    Ok(())
}

fn read_stdin_content(content: &mut String) -> Result<()> {
    if content == "-" {
        content.clear();
        io::stdin()
            .lock()
            .take(MAX_CONTENT_BYTES as u64 + 1)
            .read_to_string(content)?;
    }
    Ok(())
}

fn confirm_delete(label: &str, count: usize) -> Result<bool> {
    eprint!(
        "{}",
        inbox::message!(
            "将{label}的 {count} 条灵感移入回收站，并清除其浏览记录。输入 yes 确认：",
            "Move {count} notes from {label} to trash and clear their view history. Type yes to confirm: "
        )
    );
    io::stderr().flush()?;
    let mut answer = String::new();
    io::stdin().read_line(&mut answer)?;
    Ok(
        matches!(answer.trim().to_ascii_lowercase().as_str(), "yes" | "y")
            || matches!(answer.trim(), "是" | "确认"),
    )
}

fn confirm_empty_trash(count: usize) -> Result<bool> {
    eprint!(
        "{}",
        inbox::message!(
            "将永久删除回收站中的 {count} 条灵感。输入 yes 确认：",
            "Permanently delete {count} notes from trash. Type yes to confirm: "
        )
    );
    io::stderr().flush()?;
    let mut answer = String::new();
    io::stdin().read_line(&mut answer)?;
    Ok(
        matches!(answer.trim().to_ascii_lowercase().as_str(), "yes" | "y")
            || matches!(answer.trim(), "是" | "确认"),
    )
}
