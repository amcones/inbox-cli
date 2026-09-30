# Changelog

## Unreleased

## 0.4.1 — 2026-09-30

- Remove the `inbox completions` subcommand; installers now deploy dynamic shell completion scripts and persist shell configuration.
- Suggest recent note IDs after `show`, `restore`, and `delete` in Bash, Zsh, Fish, and PowerShell.
- Remove selected tags from an individual note with repeated `-t` options without moving the note to trash.
- Reorganize installation documentation by method and operating system.

## 0.4.0 — 2026-09-30

- Normalize ASCII letters in tags to lowercase and match tag filters without regard to English letter case, while continuing to read older mixed-case tags.
- Add `inbox review` with an optional result limit, five candidates by default, priority ordering, and short recency/view-frequency reasons.
- Add installer-managed shell completion for Bash, Zsh, Fish, and PowerShell.
- Add release build and user-local installation scripts for Unix shells and PowerShell, including optional completion installation.

## 0.3.0 — 2026-09-30

- Add case-insensitive full-text body search with tag filters, time/priority sorting, and result limits.
- Replace `-m` with the clearer `inbox add <content>` command, including stdin support via `inbox add -`.
- Add `inbox edit` for changing note content and replacing or clearing tags while preserving identity and view history.
- Move deleted notes into a recoverable trash, with `inbox trash`, `inbox restore`, and confirmed `inbox trash empty` commands.
- Commit Markdown, view-log, and trash changes in the same recoverable deletion transaction.
- Add confirmed deletion for an hour range on one day, with compact year/month/day overrides and defaults for today.

## 0.2.2 — 2026-09-29

- Bundle the IANA time-zone database on Windows so named zones and local-time operations work without an external zoneinfo installation.
- Publish native binaries for macOS ARM64/x86_64, Linux ARM64/x86_64, and Windows x86_64 with SHA-256 checksums.
- Test every pull request on all five release platforms and verify the Rust 1.89 minimum supported version.
- Validate release tags against `Cargo.toml` and this changelog before publishing.
- Publish releases as drafts and make them public only after every binary has uploaded successfully.
- Document binary installation and current platform support.

## 0.2.1 — 2026-09-29

- Add `inbox delete today` with an explicit confirmation prompt.
- Add `inbox delete all` with an explicit confirmation prompt.
- Add `--yes` / `-y` for explicitly confirmed automation.
- Make multi-day deletion recoverable as one committed transaction.
- Preserve notes added after the confirmation snapshot.
- Keep recovery compatibility with v0.2 single-note deletion journals.

## 0.2.0 — 2026-09-29

- Add Chinese and English interfaces with automatic system-language detection.
- Add deletion by unique ID prefix and clean associated view history.

## 0.1.0 — 2026-09-29

- Initial Markdown inbox with tags, view tracking, and priority sorting.
