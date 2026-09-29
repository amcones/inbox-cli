use inbox::{
    Result,
    cli::{self, Cli, Command, DeleteTarget},
    deletion, i18n,
    model::{MAX_CONTENT_BYTES, Note, terminal_text},
    query,
    storage::Store,
    views,
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
            if content == "-" {
                content.clear();
                io::stdin()
                    .lock()
                    .take(MAX_CONTENT_BYTES as u64 + 1)
                    .read_to_string(&mut content)?;
            }
            let now = jiff::Timestamp::now().to_zoned(jiff::tz::TimeZone::try_system()?);
            let note = Note::new(content, tags, &now)?;
            let store = Store::open(&root, true)?.unwrap();
            store.add(&note)?;
            drop(store);
            writeln!(out, "{}", note.short_id())?;
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
                    .map(|t| format!("#{}", terminal_text(t, false)))
                    .collect::<Vec<_>>()
                    .join(" ");
                writeln!(
                    out,
                    "{}  {} {}  {}{}{}",
                    note.short_id(),
                    &note.meta.created[..10],
                    &note.meta.created[11..19],
                    summary,
                    if tags.is_empty() { "" } else { "  " },
                    tags
                )?;
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
                    writeln!(out, "{}", inbox::message!("已删除 {id}", "Deleted {id}"))?;
                }
                target @ (DeleteTarget::Today | DeleteTarget::All) => {
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
                            DeleteTarget::Id(_) => unreachable!(),
                        }
                    };
                    if plan.ids.is_empty() {
                        writeln!(
                            out,
                            "{}",
                            inbox::message!("没有可删除的灵感", "No notes to delete")
                        )?;
                    } else if yes || confirm_delete(&label, plan.ids.len())? {
                        let store = Store::open(&root, true)?.unwrap();
                        let count = deletion::execute(&store, &plan)?;
                        writeln!(
                            out,
                            "{}",
                            inbox::message!("已删除 {count} 条灵感", "Deleted {count} notes")
                        )?;
                    } else {
                        writeln!(
                            out,
                            "{}",
                            i18n::text(
                                "已取消，未删除任何灵感",
                                "Cancelled; no notes were deleted"
                            )
                        )?;
                    }
                }
            }
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
            let (files, notes, views) = match Store::open(&root, false)? {
                Some(store) => query::doctor(&store)?,
                None => (0, 0, 0),
            };
            writeln!(
                out,
                "{}",
                inbox::message!(
                    "OK: {files} 个日期文件，{notes} 条记录，{views} 次浏览",
                    "OK: {files} day files, {notes} notes, {views} views"
                )
            )?;
        }
        Command::Help | Command::Version => unreachable!(),
    }
    out.flush()?;
    Ok(())
}

fn confirm_delete(label: &str, count: usize) -> Result<bool> {
    eprint!(
        "{}",
        inbox::message!(
            "将永久删除{label}的 {count} 条灵感及其浏览记录。输入 yes 确认：",
            "Permanently delete {count} notes from {label} and their view history. Type yes to confirm: "
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
