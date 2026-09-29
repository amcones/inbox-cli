use crate::{
    Result,
    storage::{Store, open_rw, read_tail, sync_dir},
};
use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, Write},
};

pub fn counts(store: &Store) -> Result<HashMap<String, u64>> {
    let path = store.root.join(".inbox/views.log");
    let file = match File::open(&path) {
        Ok(file) => file,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(HashMap::new()),
        Err(e) => return Err(e.into()),
    };
    let mut counts = HashMap::<String, u64>::new();
    let mut reader = BufReader::new(file);
    let mut line = String::new();
    let mut line_no = 0;
    while reader.read_line(&mut line)? != 0 {
        line_no += 1;
        let Some((id, time)) = line.strip_suffix('\n').and_then(|s| s.split_once('\t')) else {
            return Err(crate::message!(
                "{}:{line_no}: 浏览日志不完整，请备份后修复",
                "{}:{line_no}: incomplete view log; back it up before repairing",
                path.display()
            )
            .into());
        };
        let parsed = uuid::Uuid::parse_str(id);
        if !parsed.is_ok_and(|u| u.to_string() == id) || time.parse::<jiff::Timestamp>().is_err() {
            return Err(crate::message!(
                "{}:{line_no}: 无效的浏览日志",
                "{}:{line_no}: invalid view log",
                path.display()
            )
            .into());
        }
        let count = counts.entry(id.to_owned()).or_default();
        *count = count.saturating_add(1);
        line.clear();
    }
    Ok(counts)
}

pub fn record(store: &Store, id: &str) -> Result<()> {
    store.require_exclusive()?;
    let path = store.root.join(".inbox/views.log");
    let (mut file, is_new) = open_rw(&path, true)?;
    if file.metadata()?.len() != 0 && !read_tail(&mut file, 1)?.ends_with(b"\n") {
        return Err(crate::i18n::text("浏览日志尾部不完整，请备份后修复；本次浏览未计数", "The view log tail is incomplete; back it up before repairing; this view was not counted").into());
    }
    writeln!(file, "{id}\t{}", jiff::Timestamp::now())?;
    file.sync_all()?;
    if is_new {
        sync_dir(&store.root.join(".inbox"))?;
    }
    Ok(())
}
