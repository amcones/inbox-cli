# Changelog

## Unreleased

## 0.6.2 — 2026-10-01

- Add checksum-verified bootstrap installers that detect the platform and download the latest prebuilt release from GitHub on macOS, Linux, and Windows. <!-- zh: 为 macOS、Linux 和 Windows 新增带校验的引导安装脚本，自动识别平台并从 GitHub 下载最新预编译版本。 -->
- Make elapsed-time output and `review` explanations independently configurable through `INBOX_SHOW_ELAPSED` and `INBOX_REVIEW_REASONS`. <!-- zh: 通过 INBOX_SHOW_ELAPSED 和 INBOX_REVIEW_REASONS 分别配置是否显示命令耗时和 review 原因。 -->
- Replace the compact `inbox info` mark with the new high-resolution ASCII artwork. <!-- zh: 使用新的高分辨率 ASCII 字符画替换 inbox info 中的紧凑图标。 -->

## 0.6.1 — 2026-10-01

- Increase the `inbox info` ASCII icon resolution and show `inbox tags` entries as the same colored `#tag` form used in note lists. <!-- zh: 提高 inbox info 的 ASCII 图标分辨率，并让 inbox tags 使用与灵感列表一致的彩色 #tag 形式。 -->
- Report elapsed time after every command on standard error, preserving standard output for scripts and pipelines. <!-- zh: 每条命令执行后在标准错误中显示耗时，同时保持标准输出可用于脚本和管道。 -->
- Replace all committed icon sizes with the latest direct Figma exports and refresh the website icon copies. <!-- zh: 使用 Figma 最新直接导出替换仓库中的全部图标尺寸，并同步更新网站图标副本。 -->
- Generate bilingual website release history from `CHANGELOG.md` at build time, removing the unreliable runtime fetch and its duplicate maintenance burden. <!-- zh: 构建时从 CHANGELOG.md 生成双语网站版本记录，移除不稳定的运行时请求，也无需维护第二份版本数据。 -->
- Add an accessible, responsive back-to-top control and language-specific documentation links to the introducing website. <!-- zh: 为产品主页新增支持无障碍和响应式布局的回到顶部按钮，并按页面语言链接对应的中英文文档。 -->
- Automatically validate and create the version tag after a prepared release is merged into `main`, then invoke the reusable release workflow so publication cannot be skipped by a missing manual tag. <!-- zh: 发布准备合并到 main 后自动校验并创建版本标签，再调用可复用发布工作流，避免因遗漏手动推送标签而跳过发布。 -->

## 0.6.0 — 2026-10-01

- Redesign list, search, review, and trash output with compact relative times, result counts, and terminal-aware colors while keeping redirected note output plain and script-friendly. <!-- zh: 重新设计列表、搜索、回顾和回收站输出，使用简洁的相对时间、结果总数和终端感知配色，同时保持重定向输出为纯文本并兼容脚本。 -->
- Highlight case-insensitive search matches and center long excerpts around the first match. <!-- zh: 高亮不区分大小写的搜索结果，并让长内容摘要围绕首次匹配位置显示。 -->
- Expand `inbox info` with an ASCII icon, version, UTC build time, license, author, storage location, and library counts. <!-- zh: 扩展 inbox info，展示 ASCII 图标、版本、UTC 构建时间、许可证、作者、存储位置和资料库统计。 -->

## 0.5.1 — 2026-09-30

- Add `inbox backup verify <directory>` for read-only validation of backup manifests, notes, view history, and trash. <!-- zh: 新增 inbox backup verify <目录>，以只读方式校验备份清单、灵感、浏览记录和回收站。 -->
- Test that v0.5.0 backup format restores in v0.5.1 and reject unknown backup format versions without modification. <!-- zh: 验证 v0.5.0 备份可在 v0.5.1 中还原，并在不修改数据的前提下拒绝未知备份格式。 -->
- Add `inbox info` with the installed version, resolved data directory, storage state, and library counts. <!-- zh: 新增 inbox info，展示安装版本、解析后的数据目录、存储状态和资料库统计。 -->
- Complete the new command and backup action across Bash, Zsh, Fish, and PowerShell. <!-- zh: 为 Bash、Zsh、Fish 和 PowerShell 补齐新命令与备份操作的补全支持。 -->

## 0.5.0 — 2026-09-30

- Add `inbox backup <directory>` for locked, validated snapshots of Markdown notes, view history, and trash. <!-- zh: 新增 inbox backup <目录>，在锁定并校验后创建 Markdown 灵感、浏览记录和回收站快照。 -->
- Add confirmed full-library restore with `inbox restore --from <directory>`, automatic pre-restore safety backups, and `--yes` for automation. <!-- zh: 新增需确认的整库还原、还原前自动安全备份，以及适用于自动化的 --yes 选项。 -->
- Make committed restore transactions recoverable after interruption while preserving the active lock file. <!-- zh: 已提交的还原事务在中断后可以恢复，同时保留当前锁文件。 -->
- Complete backup paths and restore options in Bash, Zsh, Fish, and PowerShell. <!-- zh: 为 Bash、Zsh、Fish 和 PowerShell 补齐备份路径和还原选项的补全支持。 -->

## 0.4.3 — 2026-09-30

- Replace the `-h` / `--help` flags with the `inbox help` command; `delete range -h` remains the hour-range option. <!-- zh: 用 inbox help 命令替代 -h / --help；delete range -h 继续表示小时范围。 -->
- Add verified update scripts for macOS, Linux, and Windows that replace the installed binary and refresh existing shell completion files. <!-- zh: 为 macOS、Linux 和 Windows 新增经过校验的更新脚本，可替换已安装程序并刷新现有命令补全文件。 -->
- Install the updater beside the binary and include updater and completion scripts in release archives. <!-- zh: 将更新器安装在程序旁，并在发布归档中包含更新器和命令补全脚本。 -->

## 0.4.2 — 2026-09-30

- Confirm bulk deletion and permanent trash emptying with one `y` keypress; any other key cancels.
- Fix dynamic ID completion for `show`, `delete`, and `restore`, using active notes or trash as appropriate.
- Complete command-specific options, tag values, sort and language values, paths, date parts, hours, limits, and delete targets across Bash, Zsh, Fish, and PowerShell.
- Add a stable hidden completion interface so shell scripts no longer parse human-readable list output.

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
