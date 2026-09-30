use inbox::{cli::Sort, model::Note, query, storage::Store, views};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
};

struct Inbox(PathBuf);

impl Inbox {
    fn new() -> Self {
        Self(std::env::temp_dir().join(format!("inbox-test-{}", uuid::Uuid::new_v4())))
    }
    fn command(&self) -> Command {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_inbox"));
        cmd.arg("--dir")
            .arg(&self.0)
            .env("TZ", "Asia/Shanghai")
            .env("LC_ALL", "zh_CN.UTF-8")
            .env_remove("INBOX_LANG");
        cmd
    }
    fn run(&self, args: &[&str]) -> Output {
        self.command().args(args).output().unwrap()
    }
    fn ok(&self, args: &[&str]) -> String {
        let out = self.run(args);
        assert!(
            out.status.success(),
            "{args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8(out.stdout).unwrap()
    }
    fn run_input(&self, args: &[&str], input: &str) -> Output {
        let mut child = self
            .command()
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(input.as_bytes())
            .unwrap();
        child.wait_with_output().unwrap()
    }
    fn add_at(&self, text: &str, tags: &[&str], at: &str) -> Note {
        let note = Note::new(
            text.into(),
            tags.iter().map(|s| s.to_string()).collect(),
            &at.parse().unwrap(),
        )
        .unwrap();
        Store::open(&self.0, true)
            .unwrap()
            .unwrap()
            .add(&note)
            .unwrap();
        note
    }
    fn files(&self) -> Vec<PathBuf> {
        Store::open(&self.0, false)
            .unwrap()
            .unwrap()
            .day_files()
            .unwrap()
    }
}

impl Drop for Inbox {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn empty_inbox_does_not_create_files() {
    let inbox = Inbox::new();
    assert_eq!(inbox.ok(&[]), "");
    assert_eq!(inbox.ok(&["search", "anything"]), "");
    assert_eq!(inbox.ok(&["tags"]), "");
    assert!(inbox.ok(&["doctor"]).contains("0 条记录"));
    assert!(!inbox.0.exists());
}

#[test]
fn full_text_search_supports_case_tags_sorting_and_limits_without_tracking_views() {
    let inbox = Inbox::new();
    let older = inbox.add_at(
        "Rust CLI\n第二行包含阅读模式",
        &["产品", "开发"],
        "2026-09-28T09:00:00+08:00[Asia/Shanghai]",
    );
    inbox.add_at(
        "用 RUST 写另一个工具",
        &["开发"],
        "2026-09-29T09:00:00+08:00[Asia/Shanghai]",
    );
    inbox.add_at(
        "不相关记录",
        &["产品"],
        "2026-09-29T10:00:00+08:00[Asia/Shanghai]",
    );

    let rust = inbox.ok(&["search", "rust"]);
    assert_eq!(rust.lines().count(), 2);
    assert!(rust.contains("Rust CLI"));
    assert!(rust.contains("用 RUST"));
    assert_eq!(
        inbox.ok(&["search", "rust", "-t", "产品"]).lines().count(),
        1
    );
    assert!(
        inbox
            .ok(&["search", "阅读模式", "-t", "产品"])
            .contains(older.short_id())
    );
    assert_eq!(
        inbox
            .ok(&["search", "rust", "-t", "产品", "-t", "missing", "--any"])
            .lines()
            .count(),
        1
    );
    assert_eq!(inbox.ok(&["search", "rust", "-n", "1"]).lines().count(), 1);
    assert_eq!(
        inbox
            .ok(&["search", "rust", "--sort", "priority", "-n", "1"])
            .lines()
            .count(),
        1
    );
    assert!(!inbox.0.join(".inbox/views.log").exists());

    for args in [
        &["search"][..],
        &["search", "   "][..],
        &["search", "rust", "--any"][..],
        &["search", "rust", "--no-track"][..],
    ] {
        assert_eq!(inbox.run(args).status.code(), Some(2));
    }
}

#[test]
fn add_show_and_tags_are_persisted_and_lists_do_not_count_views() {
    let inbox = Inbox::new();
    let id = inbox.ok(&[
        "add",
        "记住这个想法 🦀",
        "-t",
        "产品",
        "-t",
        "#产品",
        "-t",
        "Rust",
    ]);
    assert_eq!(id.trim().len(), 8);
    assert!(inbox.ok(&[]).contains("记住这个想法"));
    assert_eq!(inbox.ok(&["tags"]), "rust\t1\n产品\t1\n");
    let files = inbox.files();
    assert_eq!(files.len(), 1);
    let before = fs::read(&files[0]).unwrap();
    assert!(
        inbox
            .ok(&["show", id.trim(), "--no-track"])
            .contains("记住这个想法 🦀")
    );
    assert!(!inbox.0.join(".inbox/views.log").exists());
    inbox.ok(&["show", id.trim()]);
    inbox.ok(&["show", id.trim()]);
    inbox.ok(&["list", "--sort", "priority"]);
    assert!(inbox.ok(&["doctor"]).contains("2 次浏览"));
    assert_eq!(fs::read(&files[0]).unwrap(), before);
}

#[test]
fn tags_are_ascii_case_insensitive_and_stored_lowercase() {
    let inbox = Inbox::new();
    inbox.ok(&["add", "one", "-t", "A", "-t", "b", "-t", "a"]);
    inbox.ok(&["add", "two", "-t", "a"]);
    inbox.ok(&["add", "three", "-t", "B"]);
    let both = inbox.ok(&["list", "-t", "A", "-t", "B"]);
    assert_eq!(both.lines().count(), 1);
    assert!(both.contains("one"));
    assert_eq!(
        inbox
            .ok(&["list", "-t", "b", "-t", "B", "--any"])
            .lines()
            .count(),
        2
    );
    assert_eq!(inbox.ok(&["tags"]), "a\t2\nb\t2\n");
    assert!(
        String::from_utf8(fs::read(&inbox.files()[0]).unwrap())
            .unwrap()
            .contains(r#""tags":["a","b"]"#)
    );
    assert!(inbox.ok(&["list", "-t", "missing"]).is_empty());
}

#[test]
fn legacy_mixed_case_tags_are_read_case_insensitively() {
    let inbox = Inbox::new();
    inbox.add_at(
        "legacy",
        &["rust"],
        "2026-09-29T09:00:00+08:00[Asia/Shanghai]",
    );
    let path = &inbox.files()[0];
    let old = fs::read_to_string(path)
        .unwrap()
        .replace(r#""tags":["rust"]"#, r#""tags":["Rust","RUST"]"#);
    fs::write(path, old).unwrap();

    assert_eq!(inbox.ok(&["tags"]), "rust\t1\n");
    assert!(inbox.ok(&["list", "-t", "RUST"]).contains("legacy"));
    assert!(inbox.ok(&["doctor"]).contains("1 条记录"));
}

#[test]
fn legacy_mixed_case_tags_in_trash_can_be_listed_and_restored() {
    let inbox = Inbox::new();
    let id = inbox.ok(&["add", "legacy trash", "-t", "rust"]);
    inbox.ok(&["delete", id.trim()]);
    let path = fs::read_dir(inbox.0.join(".inbox/trash"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let old = fs::read_to_string(&path)
        .unwrap()
        .replace(r#""tags": ["rust"]"#, r#""tags": ["Rust","RUST"]"#);
    fs::write(&path, old).unwrap();

    assert!(inbox.ok(&["trash"]).contains("#rust"));
    inbox.ok(&["restore", id.trim()]);
    assert!(
        inbox
            .ok(&["show", id.trim(), "--no-track"])
            .contains("#rust")
    );
}

#[test]
fn review_defaults_to_five_explained_candidates_and_honors_limit() {
    let inbox = Inbox::new();
    let mut ids = Vec::new();
    for i in 0..6 {
        ids.push(inbox.ok(&["add", &format!("idea {i}")]));
    }
    inbox.ok(&["show", ids[0].trim()]);
    inbox.ok(&["show", ids[0].trim()]);

    let default = inbox.ok(&["review"]);
    assert_eq!(default.matches("原因：").count(), 5);
    let one = inbox.ok(&["review", "-n", "1"]);
    assert!(one.contains("idea 0"));
    assert!(one.contains("近期"));
    assert!(one.contains("常看（2 次）"));
    assert!(inbox.ok(&["doctor"]).contains("2 次浏览"));

    for args in [
        &["review", "-n", "0"][..],
        &["review", "--sort", "time"][..],
        &["review", "-t", "idea"][..],
    ] {
        assert_eq!(inbox.run(args).status.code(), Some(2));
    }
}

#[test]
fn stdin_multiline_and_reserved_markers_round_trip() {
    let inbox = Inbox::new();
    let text = "第一行\r\n\r\n- 第二行\r\n<!-- inbox:note {} -->\r\n<!-- inbox:end fake -->\r\n";
    let mut child = inbox
        .command()
        .args(["add", "-", "-t", "a-->b"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(text.as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let id = String::from_utf8(out.stdout).unwrap();
    let store = Store::open(&inbox.0, false).unwrap().unwrap();
    assert_eq!(
        query::find(&store, id.trim()).unwrap().content,
        text.replace("\r\n", "\n")
    );
    assert_eq!(query::find(&store, id.trim()).unwrap().meta.tags, ["a-->b"]);
}

#[test]
fn local_midnight_creates_a_new_file() {
    let inbox = Inbox::new();
    inbox.add_at("昨天", &[], "2026-09-28T23:59:59+08:00[Asia/Shanghai]");
    inbox.add_at("今天", &[], "2026-09-29T00:00:00+08:00[Asia/Shanghai]");
    inbox.add_at(
        "今天第二条",
        &[],
        "2026-09-29T00:00:01+08:00[Asia/Shanghai]",
    );
    assert_eq!(inbox.files().len(), 2);
    let latest = inbox.ok(&["list", "-n", "1"]);
    assert!(latest.contains("今天第二条"));
    assert!(inbox.ok(&["doctor"]).contains("3 条记录"));
}

#[test]
fn time_sort_handles_clock_rollback_and_overlapping_timezone_days() {
    let inbox = Inbox::new();
    inbox.add_at("later-utc", &[], "2026-09-28T23:50:00-10:00[-10:00]");
    inbox.add_at("earlier-utc", &[], "2026-09-29T00:10:00+14:00[+14:00]");
    inbox.add_at("rollback", &[], "2026-09-28T20:00:00-10:00[-10:00]");
    assert!(inbox.ok(&["list", "-n", "1"]).contains("later-utc"));
    let all = inbox.ok(&["list"]);
    let lines: Vec<_> = all.lines().collect();
    assert!(lines[0].contains("later-utc"));
    assert!(lines[1].contains("rollback"));
    assert!(lines[2].contains("earlier-utc"));
}

#[test]
fn views_promote_records_and_priority_top_k_matches_full_order() {
    let inbox = Inbox::new();
    let older = inbox.add_at("older", &["a"], "2026-09-28T09:00:00+08:00[Asia/Shanghai]");
    inbox.add_at("newer", &["a"], "2026-09-29T09:00:00+08:00[Asia/Shanghai]");
    inbox.add_at(
        "excluded",
        &["b"],
        "2026-09-29T10:00:00+08:00[Asia/Shanghai]",
    );
    let now = "2026-09-29T12:00:00+08:00".parse().unwrap();
    {
        let store = Store::open(&inbox.0, false).unwrap().unwrap();
        assert_eq!(
            query::list(&store, &["a".into()], false, Sort::Priority, 1, now).unwrap()[0].content,
            "newer"
        );
    }
    for _ in 0..5 {
        inbox.ok(&["show", older.short_id()]);
    }
    let store = Store::open(&inbox.0, false).unwrap().unwrap();
    assert_eq!(
        query::list(&store, &["a".into()], false, Sort::Priority, 1, now).unwrap()[0].content,
        "older"
    );
    let all = query::list(&store, &[], false, Sort::Priority, 20, now).unwrap();
    let first = query::list(&store, &[], false, Sort::Priority, 1, now).unwrap();
    assert_eq!(first[0].meta.id, all[0].meta.id);
}

#[test]
fn concurrent_adds_and_views_do_not_lose_or_interleave_data() {
    let inbox = Inbox::new();
    let children: Vec<_> = (0..24)
        .map(|i| {
            inbox
                .command()
                .args(["add", &format!("并发 {i}")])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap()
        })
        .collect();
    let mut ids = Vec::new();
    for child in children {
        let out = child.wait_with_output().unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        ids.push(String::from_utf8(out.stdout).unwrap());
    }
    assert_eq!(inbox.files().len(), 1);
    assert!(inbox.ok(&["doctor"]).contains("24 条记录"));
    let children: Vec<_> = (0..16)
        .map(|_| {
            inbox
                .command()
                .args(["show", ids[0].trim()])
                .stdout(Stdio::null())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap()
        })
        .collect();
    for child in children {
        assert!(child.wait_with_output().unwrap().status.success());
    }
    assert!(inbox.ok(&["doctor"]).contains("16 次浏览"));
}

#[test]
fn partial_markdown_is_reported_and_never_overwritten() {
    let inbox = Inbox::new();
    inbox.ok(&["add", "完整记录"]);
    let file = inbox.files().pop().unwrap();
    let mut damaged = fs::read(&file).unwrap();
    damaged.extend_from_slice(b"<!-- inbox:note {\"id\":");
    fs::write(&file, &damaged).unwrap();
    assert!(!inbox.run(&["doctor"]).status.success());
    assert!(!inbox.run(&["list"]).status.success());
    assert!(!inbox.run(&["add", "不能追加"]).status.success());
    assert_eq!(fs::read(file).unwrap(), damaged);
}

#[test]
fn partial_view_log_does_not_prevent_recording_or_untracked_reading() {
    let inbox = Inbox::new();
    let id = inbox.ok(&["add", "original"]);
    inbox.ok(&["show", id.trim()]);
    let log = inbox.0.join(".inbox/views.log");
    let mut damaged = fs::read(&log).unwrap();
    damaged.extend_from_slice(b"partial");
    fs::write(&log, &damaged).unwrap();
    inbox.ok(&["add", "still writable"]);
    inbox.ok(&["show", id.trim(), "--no-track"]);
    inbox.ok(&["list"]);
    let out = inbox.run(&["show", id.trim()]);
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).contains("original"));
    assert!(String::from_utf8_lossy(&out.stderr).contains("内容已显示"));
    assert!(!inbox.run(&["list", "--sort", "priority"]).status.success());
    assert!(!inbox.run(&["doctor"]).status.success());
    assert_eq!(fs::read(log).unwrap(), damaged);
}

#[test]
fn ambiguous_id_and_duplicate_ids_fail_explicitly() {
    let inbox = Inbox::new();
    let a = inbox.add_at("a", &[], "2026-09-29T09:00:00+08:00[Asia/Shanghai]");
    let file = inbox.files().pop().unwrap();
    let mut b = a.clone();
    b.meta.id = format!(
        "{}{}",
        &a.meta.id[..8],
        &uuid::Uuid::new_v4().to_string()[8..]
    );
    Store::open(&inbox.0, true)
        .unwrap()
        .unwrap()
        .add(&b)
        .unwrap();
    let out = inbox.run(&["show", a.short_id()]);
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("匹配多条"));
    inbox.ok(&["show", &a.meta.id, "--no-track"]);
    let text = fs::read_to_string(&file)
        .unwrap()
        .replace(&b.meta.id, &a.meta.id);
    fs::write(&file, text).unwrap();
    assert!(!inbox.run(&["doctor"]).status.success());
}

#[test]
fn invalid_input_does_not_create_a_store() {
    let inbox = Inbox::new();
    for args in [
        vec!["add", "  \n"],
        vec!["add", "a", "-t", "bad tag"],
        vec!["list", "--limit", "0"],
        vec!["show"],
        vec!["add", "a", "--sort", "time"],
        vec!["--any"],
        vec!["tags", "-t", "a"],
        vec!["--dir", ""],
        vec!["list", "--wat"],
    ] {
        assert!(!inbox.run(&args).status.success(), "{args:?}");
        assert!(!inbox.0.exists(), "{args:?}");
    }
}

#[test]
fn command_line_dir_overrides_environment_and_home_is_default() {
    let inbox = Inbox::new();
    let other = Inbox::new();
    let out = inbox
        .command()
        .env("INBOX_DIR", &other.0)
        .args(["add", "explicit"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert!(!other.0.exists());
    let output = Command::new(env!("CARGO_BIN_EXE_inbox"))
        .env("INBOX_DIR", &other.0)
        .args(["add", "environment"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let home = Inbox::new();
    let output = Command::new(env!("CARGO_BIN_EXE_inbox"))
        .env_remove("INBOX_DIR")
        .env("HOME", &home.0)
        .args(["add", "home"])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(home.0.join("inbox/.inbox/format-version").exists());
}

#[test]
fn unsupported_format_is_not_modified() {
    let inbox = Inbox::new();
    inbox.ok(&["add", "original"]);
    let version = inbox.0.join(".inbox/format-version");
    fs::write(&version, "99\n").unwrap();
    assert!(!inbox.run(&["add", "another"]).status.success());
    assert!(!inbox.run(&["list"]).status.success());
    assert_eq!(fs::read_to_string(version).unwrap(), "99\n");
}

#[test]
fn external_body_edits_are_visible_without_reindexing() {
    let inbox = Inbox::new();
    let id = inbox.ok(&["add", "before"]);
    let file = inbox.files().pop().unwrap();
    let text = fs::read_to_string(&file)
        .unwrap()
        .replace("before", "after");
    fs::write(&file, text).unwrap();
    assert!(
        inbox
            .ok(&["show", id.trim(), "--no-track"])
            .contains("after")
    );
}

#[test]
fn weird_filenames_do_not_panic_or_become_notes() {
    let inbox = Inbox::new();
    inbox.add_at("normal", &[], "2026-09-29T09:00:00+08:00[Asia/Shanghai]");
    let dir = inbox.0.join("2026/09");
    fs::write(dir.join("2026-09-é.md"), "ignored").unwrap();
    fs::write(dir.join("2026-09-31.md"), "ignored").unwrap();
    assert!(inbox.ok(&["doctor"]).contains("1 条记录"));
}

#[cfg(unix)]
#[test]
fn read_only_day_file_is_not_reported_as_success() {
    use std::os::unix::fs::PermissionsExt;
    let inbox = Inbox::new();
    inbox.ok(&["add", "original"]);
    let file = inbox.files().pop().unwrap();
    let before = fs::read(&file).unwrap();
    fs::set_permissions(&file, fs::Permissions::from_mode(0o400)).unwrap();
    let result = inbox.run(&["add", "should fail"]);
    fs::set_permissions(&file, fs::Permissions::from_mode(0o600)).unwrap();
    assert!(!result.status.success());
    assert_eq!(fs::read(&file).unwrap(), before);
}

#[test]
fn view_counts_are_cumulative_without_rewriting_markdown() {
    let inbox = Inbox::new();
    let note = inbox.add_at("a", &[], "2026-09-29T09:00:00+08:00[Asia/Shanghai]");
    let store = Store::open(&inbox.0, true).unwrap().unwrap();
    for _ in 0..100 {
        views::record(&store, &note.meta.id).unwrap();
    }
    assert_eq!(views::counts(&store).unwrap()[&note.meta.id], 100);
}

#[test]
fn metadata_errors_report_file_location() {
    let inbox = Inbox::new();
    inbox.ok(&["add", "original"]);
    let path = inbox.files().pop().unwrap();
    let text = fs::read_to_string(&path)
        .unwrap()
        .replace("\"tags\":[]", "\"tags\":[\"bad tag\"]");
    fs::write(&path, text).unwrap();
    let out = inbox.run(&["doctor"]);
    assert!(!out.status.success());
    assert!(
        String::from_utf8_lossy(&out.stderr)
            .contains(Path::new(&path).file_name().unwrap().to_str().unwrap())
    );
}

#[test]
fn missing_footer_newline_is_reported_by_doctor_and_add() {
    let inbox = Inbox::new();
    inbox.ok(&["add", "original"]);
    let path = inbox.files().pop().unwrap();
    let text = fs::read_to_string(&path).unwrap();
    fs::write(&path, text.trim_end_matches('\n')).unwrap();
    assert!(!inbox.run(&["doctor"]).status.success());
    assert!(!inbox.run(&["add", "another"]).status.success());
}

#[test]
fn oversized_stdin_is_rejected_without_creating_data() {
    let inbox = Inbox::new();
    let mut child = inbox
        .command()
        .args(["add", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(&vec![b'x'; inbox::model::MAX_CONTENT_BYTES + 1])
        .unwrap();
    assert!(!child.wait_with_output().unwrap().status.success());
    assert!(!inbox.0.exists());
}

#[test]
fn broken_output_pipe_does_not_increment_views() {
    let inbox = Inbox::new();
    let note = inbox.add_at(
        &"x".repeat(1024 * 1024),
        &[],
        "2026-09-29T09:00:00+08:00[Asia/Shanghai]",
    );
    let mut child = inbox
        .command()
        .args(["show", &note.meta.id])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    drop(child.stdout.take());
    assert!(child.wait_with_output().unwrap().status.success());
    assert!(!inbox.0.join(".inbox/views.log").exists());
}

#[cfg(unix)]
#[test]
fn symlink_day_file_is_not_written_through() {
    use std::os::unix::fs::symlink;
    let inbox = Inbox::new();
    inbox.ok(&["add", "original"]);
    let path = inbox.files().pop().unwrap();
    let backup = path.with_extension("backup");
    fs::rename(&path, &backup).unwrap();
    let before = fs::read(&backup).unwrap();
    symlink(&backup, &path).unwrap();
    assert!(
        !inbox
            .run(&["add", "must not reach backup"])
            .status
            .success()
    );
    assert_eq!(fs::read(&backup).unwrap(), before);
}

#[test]
fn killed_process_releases_the_write_lock() {
    use std::io::BufRead;
    const HELPER: &str = "INBOX_TEST_LOCK_HOLDER";
    if let Some(root) = std::env::var_os(HELPER) {
        let _store = Store::open(Path::new(&root), true).unwrap().unwrap();
        println!("LOCKED");
        std::io::stdout().flush().unwrap();
        loop {
            std::thread::park();
        }
    }
    let inbox = Inbox::new();
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "killed_process_releases_the_write_lock",
            "--nocapture",
        ])
        .env(HELPER, &inbox.0)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut output = std::io::BufReader::new(child.stdout.take().unwrap());
    let mut line = String::new();
    loop {
        line.clear();
        assert!(output.read_line(&mut line).unwrap() > 0);
        if line.trim() == "LOCKED" {
            break;
        }
    }
    child.kill().unwrap();
    child.wait().unwrap();
    inbox.ok(&["add", "after process death"]);
    assert!(inbox.ok(&["doctor"]).contains("1 条记录"));
}

#[test]
fn subsecond_creation_order_survives_reload() {
    let inbox = Inbox::new();
    inbox.add_at(
        "first",
        &[],
        "2026-09-29T09:00:00.123456789+08:00[Asia/Shanghai]",
    );
    inbox.add_at(
        "second",
        &[],
        "2026-09-29T09:00:00.123456790+08:00[Asia/Shanghai]",
    );
    assert!(inbox.ok(&["list", "-n", "1"]).contains("second"));
    assert!(
        inbox
            .ok(&["list", "--sort", "priority", "-n", "1"])
            .contains("second")
    );
}

#[test]
fn oversized_crlf_input_cannot_be_silently_truncated_by_normalization() {
    let inbox = Inbox::new();
    let text = "x\r\n".repeat(inbox::model::MAX_CONTENT_BYTES / 3 + 1);
    let mut child = inbox
        .command()
        .args(["add", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(text.as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("1 MiB"));
    assert!(!inbox.0.exists());
}

#[test]
fn leading_hash_tags_normalize_consistently() {
    let inbox = Inbox::new();
    inbox.ok(&["add", "tagged", "-t", "##产品", "-t", "#产品"]);
    assert_eq!(inbox.ok(&["tags"]), "产品\t1\n");
    assert!(inbox.ok(&["list", "-t", "##产品"]).contains("tagged"));
}

#[cfg(unix)]
#[test]
fn symlink_date_directory_cannot_hide_successful_writes() {
    use std::os::unix::fs::symlink;
    let inbox = Inbox::new();
    let elsewhere = Inbox::new();
    fs::create_dir_all(&inbox.0).unwrap();
    fs::create_dir_all(&elsewhere.0).unwrap();
    symlink(&elsewhere.0, inbox.0.join("2026")).unwrap();
    let note = Note::new(
        "must fail".into(),
        vec![],
        &"2026-09-29T09:00:00+08:00[Asia/Shanghai]".parse().unwrap(),
    )
    .unwrap();
    let store = Store::open(&inbox.0, true).unwrap().unwrap();
    assert!(store.add(&note).is_err());
    assert_eq!(fs::read_dir(&elsewhere.0).unwrap().count(), 0);
}

#[test]
fn language_options_translate_help_status_and_errors_without_touching_content() {
    let inbox = Inbox::new();
    assert!(inbox.ok(&["--help", "--lang", "en"]).contains("Usage:"));
    assert!(inbox.ok(&["--lang=zh", "--help"]).contains("用法:"));
    assert!(inbox.ok(&["--lang", "en", "doctor"]).contains("0 notes"));
    for (lang, expected) in [("en", "Content must not be empty"), ("zh", "内容不能为空")] {
        let out = inbox.run(&["--lang", lang, "add", " "]);
        assert!(!out.status.success());
        assert!(String::from_utf8_lossy(&out.stderr).contains(expected));
    }
    let id = inbox.ok(&["--lang", "en", "add", "中文原文 English", "-t", "标签"]);
    let output = inbox.ok(&["show", id.trim(), "--lang", "en", "--no-track"]);
    assert!(output.contains("中文原文 English"));
    assert!(output.contains("#标签"));
    // A language-looking message is content, not a global option.
    inbox.ok(&["add", "--", "--lang=en"]);
    assert!(inbox.ok(&["doctor"]).contains("2 条记录"));
    for args in [
        vec!["--lang", "fr"],
        vec!["--lang"],
        vec!["--lang", "zh", "--lang", "en"],
        vec!["--lang", "en", "--limit", "0"],
    ] {
        assert_eq!(inbox.run(&args).status.code(), Some(2));
    }
}

#[test]
fn language_environment_precedence_and_unsupported_locale_fallback() {
    let inbox = Inbox::new();
    let help = |vars: &[(&str, &str)], args: &[&str]| {
        let mut cmd = inbox.command();
        for key in ["LC_ALL", "LC_MESSAGES", "LANG", "INBOX_LANG"] {
            cmd.env_remove(key);
        }
        for (key, value) in vars {
            cmd.env(key, value);
        }
        let out = cmd.args(args).output().unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8(out.stdout).unwrap()
    };
    assert!(help(&[("LANG", "zh_TW.UTF-8")], &["--help"]).contains("用法:"));
    assert!(
        help(
            &[("LANG", "zh_CN.UTF-8"), ("LC_MESSAGES", "en_GB.UTF-8")],
            &["--help"]
        )
        .contains("Usage:")
    );
    assert!(
        help(
            &[("LANG", "en_US.UTF-8"), ("LC_ALL", "zh-Hans")],
            &["--help"]
        )
        .contains("用法:")
    );
    for lang in ["C", "POSIX", "fr_FR.UTF-8"] {
        assert!(help(&[("LANG", lang)], &["--help"]).contains("Usage:"));
    }
    assert!(help(&[("LANG", "C"), ("INBOX_LANG", "zh")], &["--help"]).contains("用法:"));
    assert!(
        help(
            &[("LANG", "zh_CN"), ("INBOX_LANG", "zh")],
            &["--help", "--lang=en"]
        )
        .contains("Usage:")
    );
    assert!(
        help(
            &[("LANG", "C"), ("INBOX_LANG", "zh")],
            &["--help", "--lang=auto"]
        )
        .contains("Usage:")
    );
    assert!(
        help(
            &[("LANG", "C"), ("INBOX_LANG", "invalid")],
            &["--help", "--lang=en"]
        )
        .contains("Usage:")
    );
}

#[test]
fn deleting_middle_note_preserves_neighbors_exactly_and_cleans_views() {
    let inbox = Inbox::new();
    let a = inbox.add_at(
        "first\n多行\n",
        &["keep"],
        "2026-09-29T09:00:00+08:00[Asia/Shanghai]",
    );
    let b = inbox.add_at(
        "delete me",
        &["removed"],
        "2026-09-29T10:00:00+08:00[Asia/Shanghai]",
    );
    let c = inbox.add_at(
        "last",
        &["keep"],
        "2026-09-29T11:00:00+08:00[Asia/Shanghai]",
    );
    for note in [&a, &b, &b, &c] {
        inbox.ok(&["show", &note.meta.id]);
    }
    let file = inbox.files().pop().unwrap();
    let before = fs::read_to_string(&file).unwrap();
    let expected = before.replace(&inbox::markdown::encode(&b).unwrap(), "");
    assert!(
        inbox
            .ok(&["delete", b.short_id(), "--lang", "en"])
            .contains("Moved to trash")
    );
    assert_eq!(fs::read_to_string(&file).unwrap(), expected);
    let log = fs::read_to_string(inbox.0.join(".inbox/views.log")).unwrap();
    assert!(!log.contains(&b.meta.id));
    assert!(log.contains(&a.meta.id) && log.contains(&c.meta.id));
    assert!(!inbox.run(&["show", b.short_id()]).status.success());
    assert_eq!(inbox.ok(&["tags"]), "keep\t2\n");
    assert_eq!(inbox.ok(&["list", "--sort", "priority"]).lines().count(), 2);
    assert!(inbox.ok(&["doctor"]).contains("2 条记录，2 次浏览"));
    assert!(!inbox.0.join(".inbox/delete-pending").exists());
}

#[test]
fn deleting_first_and_last_notes_keeps_a_valid_empty_day() {
    let inbox = Inbox::new();
    let a = inbox.ok(&["add", "first"]);
    let b = inbox.ok(&["add", "last"]);
    let file = inbox.files().pop().unwrap();
    inbox.ok(&["delete", a.trim()]);
    assert!(inbox.ok(&["list"]).contains("last"));
    inbox.ok(&["delete", b.trim()]);
    assert_eq!(
        fs::read_to_string(&file).unwrap(),
        format!("# {}\n\n", file.file_stem().unwrap().to_str().unwrap())
    );
    assert!(inbox.ok(&["list"]).is_empty());
    inbox.ok(&["add", "new after deletion"]);
    assert!(inbox.ok(&["doctor"]).contains("1 条记录，0 次浏览"));
}

#[test]
fn invalid_ambiguous_or_damaged_deletions_leave_data_unchanged() {
    let inbox = Inbox::new();
    let mut a = Note::new(
        "a".into(),
        vec![],
        &"2026-09-29T09:00:00+08:00[Asia/Shanghai]".parse().unwrap(),
    )
    .unwrap();
    a.meta.id = "aaaaaaaa-0000-4000-8000-000000000001".into();
    let mut b = a.clone();
    b.meta.id = "aaaaaaaa-0000-4000-8000-000000000002".into();
    {
        let store = Store::open(&inbox.0, true).unwrap().unwrap();
        store.add(&a).unwrap();
        store.add(&b).unwrap();
    }
    let path = inbox.files().pop().unwrap();
    let before = fs::read(&path).unwrap();
    for args in [
        vec!["delete"],
        vec!["delete", "aaaa"],
        vec!["delete", "bbbb"],
        vec!["delete", &a.meta.id, "--no-track"],
    ] {
        assert!(!inbox.run(&args).status.success());
        assert_eq!(fs::read(&path).unwrap(), before);
    }
    let log = inbox.0.join(".inbox/views.log");
    fs::write(&log, "incomplete").unwrap();
    assert!(!inbox.run(&["delete", &a.meta.id]).status.success());
    assert_eq!(fs::read(&path).unwrap(), before);
    assert_eq!(fs::read_to_string(&log).unwrap(), "incomplete");
    fs::remove_file(log).unwrap();
    let mut damaged = before.clone();
    damaged.extend_from_slice(b"<!-- inbox:note {");
    fs::write(&path, &damaged).unwrap();
    assert!(!inbox.run(&["delete", &a.meta.id]).status.success());
    assert_eq!(fs::read(&path).unwrap(), damaged);
}

#[test]
fn deleting_selected_tags_preserves_the_note_and_other_tags() {
    let inbox = Inbox::new();
    let id = inbox.ok(&["add", "idea", "-t", "Keep", "-t", "Remove", "-t", "Other"]);
    let output = inbox.ok(&["delete", id.trim(), "-t", "REMOVE", "-t", "missing"]);
    assert!(output.contains("删除 1 个标签"));
    let shown = inbox.ok(&["show", id.trim(), "--no-track"]);
    assert!(shown.contains("#keep #other"));
    assert!(!shown.contains("#remove"));
    assert!(inbox.ok(&["list"]).contains("idea"));
}

#[test]
fn concurrent_show_delete_and_add_keep_views_consistent() {
    let inbox = Inbox::new();
    let id = inbox.ok(&["add", "to remove"]);
    let mut children = Vec::new();
    for _ in 0..12 {
        children.push(
            inbox
                .command()
                .args(["show", id.trim()])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .unwrap(),
        );
    }
    let mut delete = inbox
        .command()
        .args(["delete", id.trim()])
        .stdout(Stdio::null())
        .spawn()
        .unwrap();
    let mut add = inbox
        .command()
        .args(["add", "survivor"])
        .stdout(Stdio::null())
        .spawn()
        .unwrap();
    for mut child in children {
        let _ = child.wait().unwrap();
    }
    assert!(delete.wait().unwrap().success());
    assert!(add.wait().unwrap().success());
    assert!(inbox.ok(&["doctor"]).contains("1 条记录，0 次浏览"));
    assert!(inbox.ok(&["list"]).contains("survivor"));
}

#[test]
fn committed_deletions_recover_at_every_replacement_boundary() {
    for applied in 0..=2 {
        let inbox = Inbox::new();
        let note = inbox.add_at("remove", &[], "2026-09-29T09:00:00+08:00[Asia/Shanghai]");
        inbox.ok(&["show", &note.meta.id]);
        let stage = inbox.0.join(".inbox/delete-pending");
        fs::create_dir(&stage).unwrap();
        fs::write(stage.join("day.next"), "# 2026-09-29\n\n").unwrap();
        fs::write(stage.join("views.next"), "").unwrap();
        fs::write(
            stage.join("manifest.json"),
            serde_json::to_vec(&serde_json::json!({"date":"2026-09-29", "id":note.meta.id}))
                .unwrap(),
        )
        .unwrap();
        if applied >= 1 {
            fs::rename(
                stage.join("day.next"),
                inbox.0.join("2026/09/2026-09-29.md"),
            )
            .unwrap();
        }
        if applied >= 2 {
            fs::rename(stage.join("views.next"), inbox.0.join(".inbox/views.log")).unwrap();
        }
        // A normal read must finish recovery before exposing any state.
        assert!(inbox.ok(&["list"]).is_empty());
        assert!(inbox.ok(&["doctor"]).contains("0 条记录，0 次浏览"));
        assert!(!stage.exists());
    }
}

#[test]
fn committed_trash_deletions_recover_at_every_replacement_boundary() {
    for applied in 0..=3 {
        let inbox = Inbox::new();
        let note = inbox.add_at(
            "recover to trash",
            &["safe"],
            "2026-09-29T09:00:00+08:00[Asia/Shanghai]",
        );
        inbox.ok(&["show", &note.meta.id]);
        let stage = inbox.0.join(".inbox/delete-pending");
        fs::create_dir(&stage).unwrap();
        fs::write(stage.join("day-2026-09-29.next"), "# 2026-09-29\n\n").unwrap();
        fs::write(stage.join("views.next"), "").unwrap();
        let trash_name = format!("trash-{}.json.next", note.meta.id);
        fs::write(
            stage.join(&trash_name),
            serde_json::to_vec_pretty(&serde_json::json!({
                "version": 1,
                "deleted_at": "2026-09-29T04:00:00Z",
                "note": { "meta": note.meta, "content": note.content }
            }))
            .unwrap(),
        )
        .unwrap();
        fs::write(
            stage.join("manifest.json"),
            serde_json::to_vec(&serde_json::json!({
                "dates": ["2026-09-29"], "ids": [note.meta.id], "trash": true
            }))
            .unwrap(),
        )
        .unwrap();
        let trash_dir = inbox.0.join(".inbox/trash");
        fs::create_dir(&trash_dir).unwrap();
        if applied >= 1 {
            fs::rename(
                stage.join(&trash_name),
                trash_dir.join(format!("{}.json", note.meta.id)),
            )
            .unwrap();
        }
        if applied >= 2 {
            fs::rename(
                stage.join("day-2026-09-29.next"),
                inbox.0.join("2026/09/2026-09-29.md"),
            )
            .unwrap();
        }
        if applied >= 3 {
            fs::rename(stage.join("views.next"), inbox.0.join(".inbox/views.log")).unwrap();
        }
        assert!(inbox.ok(&["list"]).is_empty());
        let trashed = inbox.ok(&["trash"]);
        assert!(trashed.contains(note.short_id()) && trashed.contains("recover to trash"));
        assert!(!stage.exists());
    }
}

#[test]
fn uncommitted_staging_is_ignored_and_can_be_replaced_by_next_delete() {
    let inbox = Inbox::new();
    let id = inbox.ok(&["add", "still here"]);
    let stage = inbox.0.join(".inbox/delete-pending");
    fs::create_dir(&stage).unwrap();
    fs::write(stage.join("day.next"), "partial").unwrap();
    fs::write(stage.join("manifest.tmp"), "partial").unwrap();
    assert!(inbox.ok(&["list"]).contains("still here"));
    inbox.ok(&["delete", id.trim()]);
    assert!(inbox.ok(&["doctor"]).contains("0 条记录"));
}

#[test]
fn delete_missing_inbox_does_not_create_it() {
    let inbox = Inbox::new();
    assert!(!inbox.run(&["delete", "abcd"]).status.success());
    assert!(!inbox.0.exists());
}

#[test]
fn delete_today_requires_confirmation_and_preserves_other_days() {
    let inbox = Inbox::new();
    let old = inbox.add_at("yesterday", &[], "2026-09-28T10:00:00+08:00[Asia/Shanghai]");
    let today_a = inbox.ok(&["add", "today a"]);
    let today_b = inbox.ok(&["add", "today b"]);
    for id in [&old.meta.id, today_a.trim(), today_b.trim()] {
        inbox.ok(&["show", id]);
    }

    for answer in ["no\n", "\n", "YES PLEASE\n"] {
        let out = inbox.run_input(&["delete", "today"], answer);
        assert!(out.status.success());
        assert!(String::from_utf8_lossy(&out.stdout).contains("已取消"));
        assert!(String::from_utf8_lossy(&out.stderr).contains("2 条灵感"));
        assert_eq!(inbox.ok(&["list"]).lines().count(), 3);
    }
    let out = inbox.run_input(&["delete", "today"], "确认\n");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(String::from_utf8_lossy(&out.stdout).contains("已将 2 条灵感移入回收站"));
    let list = inbox.ok(&["list"]);
    assert_eq!(list.lines().count(), 1);
    assert!(list.contains("yesterday"));
    assert!(inbox.ok(&["doctor"]).contains("1 条记录，1 次浏览"));
}

#[test]
fn delete_range_uses_a_left_closed_hour_window_and_preserves_other_notes() {
    let inbox = Inbox::new();
    let before = inbox.add_at("before", &[], "2026-09-29T07:59:00+08:00[Asia/Shanghai]");
    let start = inbox.add_at("start", &[], "2026-09-29T08:00:00+08:00[Asia/Shanghai]");
    let inside = inbox.add_at("inside", &[], "2026-09-29T11:59:00+08:00[Asia/Shanghai]");
    let end = inbox.add_at("end", &[], "2026-09-29T12:00:00+08:00[Asia/Shanghai]");
    let other_day = inbox.add_at("other day", &[], "2026-09-28T09:00:00+08:00[Asia/Shanghai]");

    let cancelled = inbox.run_input(
        &[
            "delete", "range", "-y", "26", "-m", "9", "-d", "29", "-h", "8", "12", "--lang", "en",
        ],
        "no\n",
    );
    assert!(cancelled.status.success());
    assert!(String::from_utf8_lossy(&cancelled.stderr).contains("2 notes"));
    assert_eq!(inbox.ok(&["list"]).lines().count(), 5);

    let moved = inbox.ok(&[
        "delete", "range", "-y", "26", "-m", "9", "-d", "29", "-h", "8", "12", "--yes", "--lang",
        "en",
    ]);
    assert_eq!(moved, "Moved 2 notes to trash\n");
    let active = inbox.ok(&["list"]);
    assert!(active.contains(before.short_id()));
    assert!(active.contains(end.short_id()));
    assert!(active.contains(other_day.short_id()));
    assert!(!active.contains(start.short_id()) && !active.contains(inside.short_id()));
    let trash = inbox.ok(&["trash"]);
    assert!(trash.contains(start.short_id()) && trash.contains(inside.short_id()));
}

#[test]
fn delete_range_rejects_invalid_date_or_hour_options_without_changes() {
    let inbox = Inbox::new();
    inbox.add_at("keep", &[], "2026-09-15T10:00:00+08:00[Asia/Shanghai]");
    for args in [
        vec!["delete", "range", "-y", "26", "-m", "9", "-d", "31"],
        vec!["delete", "range", "-m", "13"],
        vec!["delete", "range", "-d", "0"],
        vec!["delete", "range", "-h", "12", "12"],
        vec!["delete", "range", "-h", "23", "25"],
        vec!["delete", "range", "-y", "26", "-y", "27"],
    ] {
        assert_eq!(inbox.run(&args).status.code(), Some(2));
    }
    assert_eq!(inbox.ok(&["list"]).lines().count(), 1);
    assert!(inbox.ok(&["trash"]).is_empty());
}

#[test]
fn delete_range_defaults_to_today_and_the_full_day() {
    let inbox = Inbox::new();
    let id = inbox.ok(&["add", "today in default range"]);
    assert_eq!(
        inbox.ok(&["delete", "range", "--yes", "--lang", "en"]),
        "Moved 1 notes to trash\n"
    );
    assert!(inbox.ok(&["list"]).is_empty());
    assert!(inbox.ok(&["trash"]).contains(id.trim()));
}

#[test]
fn delete_all_confirmation_and_yes_flag_work_in_both_languages() {
    let inbox = Inbox::new();
    inbox.add_at("old", &[], "2026-09-28T10:00:00+08:00[Asia/Shanghai]");
    inbox.ok(&["add", "new"]);
    let cancelled = inbox.run_input(&["delete", "all", "--lang", "en"], "n\n");
    assert!(cancelled.status.success());
    assert!(String::from_utf8_lossy(&cancelled.stderr).contains("Move 2 notes"));
    assert!(String::from_utf8_lossy(&cancelled.stdout).contains("Cancelled"));
    let deleted = inbox.ok(&["delete", "all", "--yes", "--lang", "en"]);
    assert_eq!(deleted, "Moved 2 notes to trash\n");
    assert!(inbox.ok(&["list"]).is_empty());
    assert!(inbox.ok(&["doctor"]).contains("0 条记录，0 次浏览"));

    inbox.ok(&["add", "one more"]);
    assert_eq!(
        inbox.ok(&["delete", "all", "-y"]),
        "已将 1 条灵感移入回收站\n"
    );
    assert_eq!(inbox.ok(&["delete", "all", "-y"]), "没有可移动的灵感\n");
}

#[test]
fn bulk_delete_option_validation_is_strict() {
    let inbox = Inbox::new();
    inbox.ok(&["add", "keep"]);
    for args in [
        vec!["delete", "all", "--no-track"],
        vec!["delete", "today", "-t", "x"],
        vec!["delete", "all", "--limit", "1"],
        vec!["delete", "abcd", "--yes"],
        vec!["show", "abcd", "--yes"],
        vec!["list", "--yes"],
    ] {
        assert_eq!(inbox.run(&args).status.code(), Some(2), "{args:?}");
    }
    assert!(inbox.ok(&["doctor"]).contains("1 条记录"));
}

#[test]
fn add_is_a_subcommand_and_message_flag_is_rejected() {
    let inbox = Inbox::new();
    let id = inbox.ok(&["add", "new command"]);
    assert!(
        inbox
            .ok(&["show", id.trim(), "--no-track"])
            .contains("new command")
    );
    assert_eq!(inbox.run(&["-m", "old command"]).status.code(), Some(2));
    assert!(inbox.ok(&["doctor"]).contains("1 条记录"));
}

#[test]
fn edit_updates_content_and_explicit_tags_while_preserving_identity_and_views() {
    let inbox = Inbox::new();
    let id = inbox.ok(&["add", "before", "-t", "old"]);
    inbox.ok(&["show", id.trim()]);
    assert!(
        inbox
            .ok(&["edit", id.trim(), "after\nsecond line"])
            .contains(id.trim())
    );
    let shown = inbox.ok(&["show", id.trim(), "--no-track"]);
    assert!(shown.contains("after\nsecond line"));
    assert!(shown.contains("#old"));
    assert!(
        inbox
            .ok(&["edit", id.trim(), "-t", "new", "-t", "two"])
            .contains(id.trim())
    );
    let shown = inbox.ok(&["show", id.trim(), "--no-track"]);
    assert!(shown.contains("after\nsecond line"));
    assert!(shown.contains("#new #two"));
    assert!(!shown.contains("#old"));
    inbox.ok(&["edit", id.trim(), "--clear-tags"]);
    assert_eq!(inbox.ok(&["tags"]), "");
    assert!(inbox.ok(&["doctor"]).contains("1 条记录，1 次浏览"));
}

#[test]
fn edit_reads_stdin_and_rejects_invalid_or_ambiguous_requests_without_changes() {
    let mut a = Note::new(
        "first".into(),
        vec!["a".into()],
        &"2026-09-29T09:00:00+08:00[Asia/Shanghai]".parse().unwrap(),
    )
    .unwrap();
    a.meta.id = "aaaaaaaa-0000-4000-8000-000000000001".into();
    let mut b = a.clone();
    b.meta.id = "aaaaaaaa-0000-4000-8000-000000000002".into();
    b.content = "second".into();
    let inbox = Inbox::new();
    {
        let store = Store::open(&inbox.0, true).unwrap().unwrap();
        store.add(&a).unwrap();
        store.add(&b).unwrap();
    }
    let before = fs::read(inbox.files().pop().unwrap()).unwrap();
    assert!(!inbox.run(&["edit", "aaaa", "changed"]).status.success());
    assert!(!inbox.run(&["edit", &a.meta.id, " "]).status.success());
    assert_eq!(fs::read(inbox.files().pop().unwrap()).unwrap(), before);
    let out = inbox.run_input(&["edit", &a.meta.id, "-"], "stdin\ncontent");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        inbox
            .ok(&["show", &a.meta.id, "--no-track"])
            .contains("stdin\ncontent")
    );
}

#[test]
fn delete_moves_to_trash_restore_preserves_note_and_clears_view_history() {
    let inbox = Inbox::new();
    let id = inbox.ok(&["add", "recover me", "-t", "saved"]);
    inbox.ok(&["show", id.trim()]);
    inbox.ok(&["delete", id.trim()]);
    assert!(inbox.ok(&["list"]).is_empty());
    let trash = inbox.ok(&["trash"]);
    assert!(trash.contains(id.trim()) && trash.contains("recover me") && trash.contains("#saved"));
    assert!(inbox.ok(&["restore", id.trim()]).contains(id.trim()));
    assert!(inbox.ok(&["trash"]).is_empty());
    let shown = inbox.ok(&["show", id.trim(), "--no-track"]);
    assert!(shown.contains("recover me") && shown.contains("#saved"));
    assert!(inbox.ok(&["doctor"]).contains("1 条记录，0 次浏览"));
}

#[test]
fn trash_empty_requires_confirmation_and_yes_permanently_removes_entries() {
    let inbox = Inbox::new();
    let first = inbox.ok(&["add", "first"]);
    let second = inbox.ok(&["add", "second"]);
    inbox.ok(&["delete", first.trim()]);
    inbox.ok(&["delete", second.trim()]);
    let cancelled = inbox.run_input(&["trash", "empty", "--lang", "en"], "no\n");
    assert!(cancelled.status.success());
    assert!(String::from_utf8_lossy(&cancelled.stderr).contains("Permanently delete 2 notes"));
    assert_eq!(inbox.ok(&["trash"]).lines().count(), 2);
    assert_eq!(
        inbox.ok(&["trash", "empty", "--yes", "--lang", "en"]),
        "Permanently deleted 2 notes\n"
    );
    assert!(inbox.ok(&["trash"]).is_empty());
    assert!(!inbox.run(&["restore", first.trim()]).status.success());
}

#[test]
fn doctor_and_restore_reject_damaged_trash_entries() {
    let inbox = Inbox::new();
    let id = inbox.ok(&["add", "damaged later"]);
    inbox.ok(&["delete", id.trim()]);
    let file = fs::read_dir(inbox.0.join(".inbox/trash"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&file).unwrap()).unwrap();
    value["version"] = 99.into();
    fs::write(&file, serde_json::to_vec(&value).unwrap()).unwrap();
    assert!(!inbox.run(&["doctor"]).status.success());
    assert!(!inbox.run(&["restore", id.trim()]).status.success());
    assert!(inbox.ok(&["list"]).is_empty());
}

#[test]
fn multi_day_committed_bulk_delete_recovers_at_each_boundary() {
    for applied in 0..=3 {
        let inbox = Inbox::new();
        let a = inbox.add_at("a", &[], "2026-09-28T10:00:00+08:00[Asia/Shanghai]");
        let b = inbox.add_at("b", &[], "2026-09-29T10:00:00+08:00[Asia/Shanghai]");
        inbox.ok(&["show", &a.meta.id]);
        inbox.ok(&["show", &b.meta.id]);
        let stage = inbox.0.join(".inbox/delete-pending");
        fs::create_dir(&stage).unwrap();
        for date in ["2026-09-28", "2026-09-29"] {
            fs::write(
                stage.join(format!("day-{date}.next")),
                format!("# {date}\n\n"),
            )
            .unwrap();
        }
        fs::write(stage.join("views.next"), "").unwrap();
        fs::write(
            stage.join("manifest.json"),
            serde_json::to_vec(&serde_json::json!({
                "dates":["2026-09-28", "2026-09-29"], "ids":[a.meta.id, b.meta.id]
            }))
            .unwrap(),
        )
        .unwrap();
        if applied >= 1 {
            fs::rename(
                stage.join("day-2026-09-28.next"),
                inbox.0.join("2026/09/2026-09-28.md"),
            )
            .unwrap();
        }
        if applied >= 2 {
            fs::rename(
                stage.join("day-2026-09-29.next"),
                inbox.0.join("2026/09/2026-09-29.md"),
            )
            .unwrap();
        }
        if applied >= 3 {
            fs::rename(stage.join("views.next"), inbox.0.join(".inbox/views.log")).unwrap();
        }
        assert!(inbox.ok(&["list"]).is_empty());
        assert!(inbox.ok(&["doctor"]).contains("0 条记录，0 次浏览"));
        assert!(!stage.exists());
    }
}
