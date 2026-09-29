use inbox::{
    Result,
    cli::{self, Cli, Command},
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
    let cli = match cli::parse(std::env::args_os().skip(1)) {
        Ok(cli) => cli,
        Err(e) => {
            eprintln!("inbox: {e}\n运行 inbox --help 查看用法");
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
            write!(out, "{}", cli::HELP)?;
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
            let store = Store::open(&root, false)?.ok_or("inbox 为空")?;
            let note = query::find(&store, &prefix)?;
            drop(store);
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
                let store = Store::open(&root, true)?.unwrap();
                views::record(&store, &note.meta.id)
                    .map_err(|e| format!("内容已显示，但浏览计数保存失败：{e}"))?;
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
                "OK: {files} 个日期文件，{notes} 条记录，{views} 次浏览"
            )?;
        }
        Command::Help | Command::Version => unreachable!(),
    }
    out.flush()?;
    Ok(())
}
