use inbox::{
    Result, backup,
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
    time::{Duration, Instant},
};

mod confirmation;
mod terminal;

fn main() -> ExitCode {
    let started = Instant::now();
    let settings = Settings::from_env();
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let cli = match i18n::configure(&args).and_then(|_| cli::parse(args)) {
        Ok(cli) => cli,
        Err(e) => {
            eprintln!(
                "inbox: {e}\n{}",
                i18n::text("运行 inbox help 查看用法", "Run inbox help for usage")
            );
            write_elapsed_if_enabled(started.elapsed(), settings.show_elapsed);
            return ExitCode::from(2);
        }
    };
    match run(cli, &settings) {
        Ok(()) => {
            write_elapsed_if_enabled(started.elapsed(), settings.show_elapsed);
            ExitCode::SUCCESS
        }
        Err(e)
            if e.downcast_ref::<io::Error>()
                .is_some_and(|e| e.kind() == io::ErrorKind::BrokenPipe) =>
        {
            write_elapsed_if_enabled(started.elapsed(), settings.show_elapsed);
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("inbox: {e}");
            write_elapsed_if_enabled(started.elapsed(), settings.show_elapsed);
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli, settings: &Settings) -> Result<()> {
    let mut out = io::BufWriter::new(io::stdout().lock());
    let theme = terminal::Theme::stdout();
    let error_theme = terminal::Theme::stderr();
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
            let now = jiff::Timestamp::now();
            let notes = match Store::open(&root, false)? {
                Some(store) => query::list(&store, &tags, any, sort, limit, now)?,
                None => Vec::new(),
            };
            for note in &notes {
                write_summary(&mut out, note, now, &theme, None)?;
            }
            out.flush()?;
            write_result_count(notes.len(), &error_theme)?;
        }
        Command::Search {
            query: text,
            tags,
            any,
            sort,
            limit,
        } => {
            let now = jiff::Timestamp::now();
            let notes = match Store::open(&root, false)? {
                Some(store) => query::search(&store, &text, &tags, any, sort, limit, now)?,
                None => Vec::new(),
            };
            for note in &notes {
                write_summary(&mut out, note, now, &theme, Some(&text))?;
            }
            out.flush()?;
            write_result_count(notes.len(), &error_theme)?;
        }
        Command::Review { limit } => {
            let now = jiff::Timestamp::now();
            let items = match Store::open(&root, false)? {
                Some(store) => query::review(&store, limit, now)?,
                None => Vec::new(),
            };
            for item in &items {
                write_review(&mut out, item, now, &theme, settings.review_reasons)?;
            }
            out.flush()?;
            write_result_count(items.len(), &error_theme)?;
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
            theme.write(&mut out, terminal::BLUE, &note.meta.id)?;
            write!(out, "  ")?;
            theme.write(&mut out, terminal::CYAN, &note.meta.created)?;
            writeln!(out)?;
            if !note.meta.tags.is_empty() {
                for (index, tag) in note.meta.tags.iter().enumerate() {
                    if index != 0 {
                        write!(out, " ")?;
                    }
                    theme.write(
                        &mut out,
                        terminal::MAGENTA,
                        format_args!("#{}", terminal_text(tag, false)),
                    )?;
                }
                writeln!(out)?;
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
                let now = jiff::Timestamp::now();
                let items = trash::list(&store)?;
                for (entry, note) in &items {
                    let deleted = entry.deleted_at.parse::<jiff::Timestamp>()?;
                    write_summary_at(&mut out, note, rough_time(deleted, now), &theme, None)?;
                }
                out.flush()?;
                write_result_count(items.len(), &error_theme)?;
            } else {
                out.flush()?;
                write_result_count(0, &error_theme)?;
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
        Command::Backup { destination } => {
            if !root.try_exists()? {
                return Err(i18n::text("inbox 为空", "The inbox is empty").into());
            }
            let store = Store::open(&root, true)?.unwrap();
            let stats = backup::create(&store, &destination)?;
            writeln!(
                out,
                "{}",
                inbox::message!(
                    "备份完成：{}（{} 个文件，{} 字节）",
                    "Backup created: {} ({} files, {} bytes)",
                    destination.display(),
                    stats.files,
                    stats.bytes
                )
            )?;
        }
        Command::VerifyBackup { source } => {
            let info = backup::validate(&source)?;
            writeln!(
                out,
                "{}",
                inbox::message!(
                    "备份有效：{}（由 inbox {} 于 {} 创建；{} 个文件，{} 字节）",
                    "Backup OK: {} (created by inbox {} at {}; {} files, {} bytes)",
                    source.display(),
                    info.inbox_version,
                    info.created_at,
                    info.stats.files,
                    info.stats.bytes
                )
            )?;
        }
        Command::RestoreBackup { source, yes } => {
            let info = backup::validate(&source)?;
            if !yes && !confirm_restore(&source, info.stats.files, info.stats.bytes)? {
                writeln!(
                    out,
                    "{}",
                    i18n::text("已取消，未还原任何数据", "Cancelled; no data was restored")
                )?;
            } else {
                let store = Store::open(&root, true)?.unwrap();
                let restored = backup::restore(&store, &source)?;
                writeln!(
                    out,
                    "{}",
                    inbox::message!(
                        "还原完成：{} 个文件，{} 字节；原数据已备份至 {}",
                        "Restore complete: {} files, {} bytes; previous data backed up to {}",
                        restored.stats.files,
                        restored.stats.bytes,
                        restored.safety_backup.display()
                    )
                )?;
            }
        }
        Command::Tags => {
            if let Some(store) = Store::open(&root, false)? {
                let tags = query::tags(&store)?;
                drop(store);
                for (tag, count) in tags {
                    theme.write(
                        &mut out,
                        terminal::MAGENTA,
                        format_args!("#{}", terminal_text(&tag, false)),
                    )?;
                    writeln!(out, "\t{count}")?;
                }
            }
        }
        Command::Info => {
            let display_root = if root.is_absolute() {
                root.clone()
            } else {
                std::env::current_dir()?.join(&root)
            };
            let (initialized, files, notes, views, trashed) = match Store::open(&root, false)? {
                Some(store) => {
                    let (files, notes, views) = query::doctor(&store)?;
                    (true, files, notes, views, trash::list(&store)?.len())
                }
                None => (false, 0, 0, 0, 0),
            };
            write_info(
                &mut out,
                &theme,
                &display_root,
                (initialized, files, notes, views, trashed),
            )?;
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

fn write_summary(
    out: &mut impl Write,
    note: &Note,
    now: jiff::Timestamp,
    theme: &terminal::Theme,
    highlight: Option<&str>,
) -> Result<()> {
    write_summary_at(out, note, rough_time(note.timestamp, now), theme, highlight)
}

fn write_summary_at(
    out: &mut impl Write,
    note: &Note,
    time: String,
    theme: &terminal::Theme,
    highlight: Option<&str>,
) -> Result<()> {
    theme.write(out, terminal::CYAN, format_args!("{time:>8}"))?;
    write!(out, "  ")?;
    theme.write(out, terminal::BLUE, note.short_id())?;
    write!(out, "  ")?;

    let clean = terminal_text(&note.content, false);
    let summary = excerpt(&clean, highlight, 80);
    if let Some(query) = highlight {
        write_highlighted(out, &summary, query, theme)?;
    } else {
        write!(out, "{summary}")?;
    }
    for tag in &note.meta.tags {
        write!(out, "  ")?;
        theme.write(
            out,
            terminal::MAGENTA,
            format_args!("#{}", terminal_text(tag, false)),
        )?;
    }
    writeln!(out)?;
    Ok(())
}

fn write_review(
    out: &mut impl Write,
    item: &query::ReviewItem,
    now: jiff::Timestamp,
    theme: &terminal::Theme,
    show_reason: bool,
) -> Result<()> {
    write_summary(out, &item.note, now, theme, None)?;
    if !show_reason {
        return Ok(());
    }
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
    write!(out, "          ")?;
    theme.write(out, terminal::DIM, i18n::text("原因：", "Reason:"))?;
    theme.write(out, terminal::DIM, format_args!(" {reasons}"))?;
    writeln!(out)?;
    Ok(())
}

fn rough_time(timestamp: jiff::Timestamp, now: jiff::Timestamp) -> String {
    let seconds = (now.as_second() - timestamp.as_second()).max(0);
    match seconds {
        0..=59 => i18n::text("刚刚", "now").to_owned(),
        60..=3_599 => inbox::message!("{} 分钟", "{}m", seconds / 60),
        3_600..=86_399 => inbox::message!("{} 小时", "{}h", seconds / 3_600),
        86_400..=2_591_999 => inbox::message!("{} 天", "{}d", seconds / 86_400),
        2_592_000..=31_535_999 => {
            inbox::message!("{} 个月", "{}mo", seconds / 2_592_000)
        }
        _ => inbox::message!("{} 年", "{}y", seconds / 31_536_000),
    }
}

fn excerpt(text: &str, query: Option<&str>, limit: usize) -> String {
    let boundaries = text
        .char_indices()
        .map(|(index, _)| index)
        .chain(std::iter::once(text.len()))
        .collect::<Vec<_>>();
    let character_count = boundaries.len().saturating_sub(1);
    if character_count <= limit {
        return text.to_owned();
    }

    let match_range = query.and_then(|query| find_case_insensitive(text, query, 0));
    let mut start_character = match_range
        .and_then(|(start, _)| boundaries.binary_search(&start).ok())
        .map_or(0, |position| position.saturating_sub(20));
    start_character = start_character.min(character_count - limit);
    if let Some((_, end)) = match_range
        && let Ok(end_character) = boundaries.binary_search(&end)
        && end_character > start_character + limit
    {
        start_character = end_character
            .saturating_sub(limit)
            .min(character_count - limit);
    }
    let end_character = (start_character + limit).min(character_count);
    let mut summary = String::new();
    if start_character != 0 {
        summary.push('…');
    }
    summary.push_str(&text[boundaries[start_character]..boundaries[end_character]]);
    if end_character != character_count {
        summary.push('…');
    }
    summary
}

fn find_case_insensitive(text: &str, query: &str, from: usize) -> Option<(usize, usize)> {
    let needle = query.to_lowercase();
    if needle.is_empty() {
        return None;
    }
    for (relative_start, _) in text[from..].char_indices() {
        let start = from + relative_start;
        let mut folded = String::new();
        for (relative_end, character) in text[start..].char_indices() {
            folded.extend(character.to_lowercase());
            if folded.len() >= needle.len() {
                if folded == needle {
                    return Some((start, start + relative_end + character.len_utf8()));
                }
                break;
            }
        }
    }
    None
}

fn write_highlighted(
    out: &mut impl Write,
    text: &str,
    query: &str,
    theme: &terminal::Theme,
) -> Result<()> {
    let mut cursor = 0;
    while let Some((start, end)) = find_case_insensitive(text, query, cursor) {
        write!(out, "{}", &text[cursor..start])?;
        theme.write(out, terminal::MATCH, &text[start..end])?;
        cursor = end;
    }
    write!(out, "{}", &text[cursor..])?;
    Ok(())
}

fn write_result_count(count: usize, theme: &terminal::Theme) -> Result<()> {
    let mut error = io::stderr().lock();
    let message = inbox::message!("显示 {count} 条", "Showing {count} notes");
    theme.write(&mut error, terminal::DIM, message)?;
    writeln!(error)?;
    Ok(())
}

fn write_info(
    out: &mut impl Write,
    theme: &terminal::Theme,
    root: &std::path::Path,
    stats: (bool, usize, usize, u64, usize),
) -> Result<()> {
    let (initialized, files, notes, views, trashed) = stats;
    const ART: &str = r#"               ...:::.
            ..:::.:**#**:.
        ....:::::.:**#******:.
    ....::..:::::.:**#***@@*#***:
 ....:::::..:::::.:**@@@@@@@#*******:
:::..:::::..:::.::@@@@@@@@@@*******#*:
:::..:::::..::@@@@@@@@@@@@@@*******#*:
:::..:::::@@@@@@@@@@@@@@@@@@*******#*:
:::..::@@@@@@@@@@@@@@@@@@@@@.:.:***#*:
:::..:.@@@@@@@@@@@@@@@@@@@@@.::..:.:*:
**...:.@@@@@@@@@@@@@@@@@@@@@.::....@@@
..::::.@@@@@@@@@@@@@@@@@@@@@.::.@@@@@@
....@*:@@@@@@@@@@@@@@@@@@@@@.:::@@@@@@
....:@@...:@@@@@@@@@@@@@@@@:::*@@@@@@@
....@@@@......:@@@@@@@@@@.:@@@@@@@@@@@
 ....::.*@:.......*@@@@@@@@@@@@@@@@@@
    ....:@@@@:....:@@@@@@@@@@@@@@.
        ....*:....:@@@@@@@@@@.
            ......:@@@@@@@
               ...:@@@"#;
    let epoch = env!("INBOX_BUILD_UNIX_EPOCH").parse::<i64>()?;
    let built_at = jiff::Timestamp::new(epoch, 0)?.to_string();
    let fields = [
        (
            i18n::text("版本", "Version"),
            env!("CARGO_PKG_VERSION").to_owned(),
        ),
        (i18n::text("构建时间", "Built"), built_at),
        (
            i18n::text("开源协议", "License"),
            env!("CARGO_PKG_LICENSE").to_owned(),
        ),
        (
            i18n::text("作者", "Author"),
            env!("CARGO_PKG_AUTHORS").to_owned(),
        ),
        (
            i18n::text("数据目录", "Data directory"),
            root.display().to_string(),
        ),
        (
            i18n::text("存储", "Storage"),
            if initialized {
                i18n::text("格式 1", "format 1")
            } else {
                i18n::text("尚未初始化", "not initialized")
            }
            .to_owned(),
        ),
        (i18n::text("日期文件", "Day files"), files.to_string()),
        (i18n::text("灵感", "Notes"), notes.to_string()),
        (i18n::text("浏览", "Views"), views.to_string()),
        (i18n::text("回收站", "Trash"), trashed.to_string()),
    ];
    let art: Vec<_> = ART.lines().collect();
    let art_width = art
        .iter()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(0);
    for row in 0..art.len().max(fields.len() + 2) {
        let line = art.get(row).copied().unwrap_or("");
        write_info_art_line(out, theme, line)?;
        write!(out, "{}  ", " ".repeat(art_width - line.chars().count()))?;
        match row {
            0 => theme.write(out, terminal::BLUE, "inbox")?,
            1 => theme.write(out, terminal::DIM, "-----")?,
            _ => {
                if let Some((label, value)) = fields.get(row - 2) {
                    theme.write(out, terminal::MATCH, label)?;
                    write!(out, ": {value}")?;
                }
            }
        }
        writeln!(out)?;
    }
    Ok(())
}

fn write_info_art_line(out: &mut impl Write, theme: &terminal::Theme, line: &str) -> Result<()> {
    let mut rest = line;
    while let Some(index) = rest.find('@') {
        theme.write(out, terminal::BLUE, &rest[..index])?;
        let end = rest[index..]
            .find(|character| character != '@')
            .map_or(rest.len(), |offset| index + offset);
        theme.write(out, terminal::WHITE, &rest[index..end])?;
        rest = &rest[end..];
    }
    theme.write(out, terminal::BLUE, rest)?;
    Ok(())
}

fn write_elapsed(duration: Duration, theme: &terminal::Theme) -> io::Result<()> {
    let value = if duration.as_micros() < 1_000 {
        format!("{} us", duration.as_micros())
    } else if duration.as_secs_f64() < 1.0 {
        format!("{:.2} ms", duration.as_secs_f64() * 1_000.0)
    } else {
        format!("{:.2} s", duration.as_secs_f64())
    };
    let mut error = io::stderr().lock();
    theme.write(
        &mut error,
        terminal::DIM,
        inbox::message!("耗时 {value}", "Elapsed {value}"),
    )?;
    writeln!(error)
}

fn write_elapsed_if_enabled(duration: Duration, enabled: bool) {
    if enabled {
        let _ = write_elapsed(duration, &terminal::Theme::stderr());
    }
}

#[derive(Clone, Copy)]
struct Settings {
    show_elapsed: bool,
    review_reasons: bool,
}

impl Settings {
    fn from_env() -> Self {
        Self {
            show_elapsed: boolean_env("INBOX_SHOW_ELAPSED", true),
            review_reasons: boolean_env("INBOX_REVIEW_REASONS", true),
        }
    }
}

fn boolean_env(name: &str, default: bool) -> bool {
    let Ok(value) = std::env::var(name) else {
        return default;
    };
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => true,
        "0" | "false" | "no" | "off" => false,
        _ => default,
    }
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
            "将{label}的 {count} 条灵感移入回收站，并清除其浏览记录。按 y 确认：",
            "Move {count} notes from {label} to trash and clear their view history. Press y to confirm: "
        )
    );
    io::stderr().flush()?;
    let confirmed = confirmation::read_key()?;
    eprintln!();
    Ok(confirmed)
}

fn confirm_restore(path: &std::path::Path, files: u64, bytes: u64) -> Result<bool> {
    eprint!(
        "{}",
        inbox::message!(
            "将用备份 {}（{files} 个文件，{bytes} 字节）替换当前 inbox。按 y 确认：",
            "Replace the current inbox with backup {} ({files} files, {bytes} bytes). Press y to confirm: ",
            path.display()
        )
    );
    io::stderr().flush()?;
    let confirmed = confirmation::read_key()?;
    eprintln!();
    Ok(confirmed)
}

fn confirm_empty_trash(count: usize) -> Result<bool> {
    eprint!(
        "{}",
        inbox::message!(
            "将永久删除回收站中的 {count} 条灵感。按 y 确认：",
            "Permanently delete {count} notes from trash. Press y to confirm: "
        )
    );
    io::stderr().flush()?;
    let confirmed = confirmation::read_key()?;
    eprintln!();
    Ok(confirmed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn excerpt_keeps_a_late_search_match_visible() {
        let text = format!("{}Rust makes the match visible", "x".repeat(100));
        let summary = excerpt(&text, Some("rust"), 40);
        assert!(summary.starts_with('…'));
        assert!(summary.contains("Rust"));
        assert!(summary.chars().count() <= 42);
    }

    #[test]
    fn search_highlight_is_case_insensitive() {
        let mut output = Vec::new();
        write_highlighted(
            &mut output,
            "Learn Rust, then RUST again",
            "rust",
            &terminal::Theme::colored(),
        )
        .unwrap();
        assert_eq!(
            String::from_utf8(output).unwrap(),
            "Learn \x1b[1;33mRust\x1b[0m, then \x1b[1;33mRUST\x1b[0m again"
        );
    }
}
