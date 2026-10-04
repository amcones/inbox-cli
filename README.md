# inbox

[Website](https://amcones.cn/inbox-cli/) · [English](README.md) · [简体中文](README.zh-CN.md)

**Minimalist CLI idea-taking tool.**

inbox is a small Rust CLI that keeps your notes in local Markdown files, grouped by day. It runs when you call it, with no account or background service.

- **Quick capture:** write a note with tags, or pipe multiline content from another tool.
- **Easy recall:** search complete note bodies, filter by tags, and sort by recency or a hidden priority that rewards recent and frequently viewed ideas.
- **Editable and recoverable:** update notes, use trash for individual mistakes, and create verified full-library backups.
- **Your files:** readable daily Markdown, local storage, and a single binary for macOS, Linux, and Windows.
- **Bilingual:** Chinese and English messages, selected from your system language or an explicit setting.

## Install

### Prebuilt release

The installer detects the current platform, downloads the latest archive from [GitHub Releases](https://github.com/amcones/inbox-cli/releases/latest), verifies its SHA-256 checksum, and configures `PATH` and completion.

No Rust toolchain is needed. Run the command in your terminal (PowerShell on Windows), then open a new terminal to use `inbox` and completion. macOS/Linux support ARM64 and x86_64; Windows currently supports x86_64.

| Platform | Archive |
|---|---|
| macOS Apple Silicon | `inbox-macos-aarch64.tar.gz` |
| macOS Intel | `inbox-macos-x86_64.tar.gz` |
| Linux ARM64 | `inbox-linux-aarch64.tar.gz` |
| Linux x86_64 | `inbox-linux-x86_64.tar.gz` |
| Windows x86_64 | `inbox-windows-x86_64.zip` |

#### macOS

```bash
curl -fsSL https://amcones.cn/inbox-cli/install.sh | bash
```

#### Linux

Use the same command as macOS; ARM64 and x86_64 are detected automatically.

#### Windows

```powershell
irm https://amcones.cn/inbox-cli/install.ps1 | iex
```

### Build and install from source

#### macOS and Linux

With Rust 1.89 or newer:

```bash
git clone https://github.com/amcones/inbox-cli
cd inbox-cli
./scripts/build.sh
./scripts/install.sh
```

The installer writes PATH and completion setup to the detected shell configuration and installs `inbox-update` beside the binary. Override the destination with `--bin-dir` and `--shell`; run `./scripts/install.sh --help` for details.

#### Windows

```powershell
git clone https://github.com/amcones/inbox-cli
Set-Location inbox-cli
.\scripts\build.ps1
.\scripts\install.ps1
```

The PowerShell installer defaults to `%LOCALAPPDATA%\Programs\inbox`, persists that directory in the user PATH, loads completion from the PowerShell profile, and installs `inbox-update.ps1` beside the binary.

### Cargo

#### macOS, Linux, and Windows

With Rust 1.89 or newer, install the published source revision directly:

```bash
cargo install --git ssh://git@github.com/amcones/inbox-cli.git --tag v0.6.2 --locked
```

## Update

Release and source-script installations include an updater that verifies the archive checksum before replacing the installed binary. Existing completion files are refreshed when the release contains them.

### macOS and Linux

```bash
inbox-update                    # latest release
inbox-update --version v0.6.2  # a specific release
```

If `inbox` is outside `PATH`, pass its directory with `--bin-dir`.

### Windows

```powershell
inbox-update.ps1
inbox-update.ps1 -Version v0.6.2
```

Use `-BinDir` when `inbox.exe` is outside `PATH`. Cargo installations remain managed by Cargo; update those with the corresponding `cargo install` command.

## Capture, find, and refine

```bash
inbox add 'Add a reading mode to the blog' -t product -t blog
printf 'First line\nSecond line\n' | inbox add - -t idea
inbox                              # Recent 20 notes
inbox search 'reading mode'         # Case-insensitive body search
inbox list -t product -n 5          # Five recent notes with this tag
inbox list --sort priority          # Recent and frequently viewed ideas
inbox review                        # Five priority candidates with reasons
inbox review -n 3                   # Limit the review to three candidates
```

List, search, review, and trash views use compact relative times and report the result count. Full timestamps remain available in `inbox show <ID>`. On a terminal, times, IDs, tags, and search matches use distinct colors; redirected output stays plain, and the count is written to standard error so pipelines over note rows keep working. Set [`NO_COLOR`](https://no-color.org/) to disable color explicitly.

`add` prints a short ID. Use that ID in the following examples:

```bash
inbox show a83f912b
inbox edit a83f912b 'Let readers adjust the font size'
inbox edit a83f912b -t product -t next
inbox edit a83f912b --clear-tags
```

Editing preserves the ID, creation time, and view count. Omit tag options to keep existing tags; `-t` replaces the complete tag set. Omit content to edit tags only, or use `edit <ID> -` to read content from stdin.

| Need | Command or behavior |
|---|---|
| Match all supplied tags | `inbox list -t product -t blog` |
| Match any supplied tag | Add `--any` |
| Search and filter together | `inbox search 'reading' -t product --sort priority -n 5` |
| See tags and counts | `inbox tags` |
| See build identity, data path, and library counts | `inbox info` |
| Read without counting a view | `inbox show <ID> --no-track` |
| Full usage | `inbox help` |

ASCII letters in tags are normalized to lowercase, so `Rust`, `RUST`, and `rust` are the same tag. Duplicates and leading `#` are also normalized. A `#word` inside the body is ordinary text. `inbox tags` uses the same colored `#tag` form as note lists. Only successful `show` output counts as a view, including output redirected to a file. Lists, searches, and reviews do not count. IDs can be unique prefixes of at least four characters. Each note supports up to 1 MiB of text and 64 tags (128 bytes each).

Every command reports its elapsed time on standard error. Note IDs, note rows, tag data, and other machine-readable results remain on standard output, so existing pipelines can continue to consume them independently.

## Display configuration

Set `INBOX_SHOW_ELAPSED=off` to hide command timings, or `INBOX_REVIEW_REASONS=off` to hide the explanatory line below each `review` candidate. Both settings default to `on` and accept `1/0`, `true/false`, `yes/no`, or `on/off` (case-insensitive).

## Command completion

The install scripts deploy dynamic completion for Bash, Zsh, Fish, and PowerShell. Completion covers commands, command-specific options, tag values, sort and language values, common limits, dates and hours, active note IDs for `show`, `edit`, and `delete`, and trashed note IDs for `restore`. The Unix installer writes the required setup to the detected shell configuration; the PowerShell installer updates the user profile.

## Delete and recover

```bash
inbox delete a83f912b               # Move one note to trash
inbox delete today                  # Confirm moving today's notes
inbox delete range -h 8 12          # Today: 08:00 inclusive to 12:00 exclusive
inbox delete range -y 26 -m 9 -d 29 -h 0 24
inbox delete all                    # Confirm moving all notes
inbox trash
inbox restore a83f912b
inbox trash empty                   # Confirm permanent deletion
```

For `delete range`, omitted year, month, and day default to the current local date; omitted hours default to `0 24`. Two-digit years mean 2000–2099. Matching uses each note's recorded date and hour. In this command, `-y` means year and `-h` means hours; use `--yes` to skip confirmation and `inbox help` for help.

Bulk deletion asks you to press `y`; any other key or EOF cancels immediately. `--yes` skips the prompt for automation. Notes added while you confirm are preserved. Deletion clears view history; restoring brings back the body, tags, ID, and original creation time, but not past views. **Emptying trash is permanent.**

## Backup and restore

Create a complete snapshot of the Markdown files, view history, and trash:

```bash
inbox backup ~/Backups/inbox-2026-09-30
inbox backup verify ~/Backups/inbox-2026-09-30
inbox restore --from ~/Backups/inbox-2026-09-30
```

The backup command holds the inbox lock, validates all notes, view history, and trash, then writes a self-contained directory with a manifest. `backup verify` repeats the complete validation without locking or modifying the snapshot, including on read-only media. The destination must not already exist and cannot be inside the data directory. Backups preserve readable files and are not encrypted.

Full restore validates the snapshot before asking you to press `y`; any other key cancels. `--yes` skips the prompt for automation. Before replacement, inbox automatically creates a sibling snapshot named `inbox.before-restore-…` and reports its path. A committed restore interrupted by a crash continues the next time inbox opens the library. Keep backups on separate storage if you need protection from disk loss.

## Data and language

Notes live in `~/inbox/YYYY/MM/YYYY-MM-DD.md`. Change the location with `--dir` or `INBOX_DIR`; the command-line option takes precedence.

```bash
inbox --dir ~/Notes/inbox add 'An idea'
inbox help --lang en
inbox help --lang zh
```

Language precedence is `--lang` > `INBOX_LANG` > system language. Choose `auto`, `en`, or `zh`; unsupported system languages fall back to English. Body text and tags are never translated. Since v0.3.0, use `add` instead of the old `-m` capture option. For body text starting with a dash, put options first and use `--`, for example `inbox add -- '--an idea'`.

Run `inbox info` to see the ASCII application mark, installed version, UTC build time, license, author, resolved data directory, storage state, and counts for day files, active notes, views, and trash. It validates the same library data as `doctor` while producing a concise operational summary.

If a command reports damaged data, stop writing, make a backup, and run `inbox doctor`. It checks notes, view history, and trash; it does not guess how to repair corrupted content. For manual Markdown edits, preserve metadata and record markers and avoid concurrent writes. See the [storage and recovery guide](docs/FORMAT.md).

## Project

[Introducing website](https://amcones.cn/inbox-cli/) · [Brand assets](art/README.md) · [Development and contributions](CONTRIBUTE.md) · [Performance measurements](docs/BENCHMARK.md) · [Architecture assessment](docs/ARCHITECTURE.md) · [Changelog](CHANGELOG.md) · [MIT license](LICENSE)
