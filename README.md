# inbox

[English](README.md) · [简体中文](README.zh-CN.md)

**Minimalist CLI idea-taking tool.**

inbox is a small Rust CLI that keeps your notes in local Markdown files, grouped by day. It runs when you call it, with no account or background service.

- **Quick capture:** write a note with tags, or pipe multiline content from another tool.
- **Easy recall:** search complete note bodies, filter by tags, and sort by recency or a hidden priority that rewards recent and frequently viewed ideas.
- **Editable and recoverable:** update notes, move them to trash, and restore them with their original identity.
- **Your files:** readable daily Markdown, local storage, and a single binary for macOS, Linux, and Windows.
- **Bilingual:** Chinese and English messages, selected from your system language or an explicit setting.

## Install

Download an archive from [GitHub Releases](https://github.com/amcones/inbox-cli/releases/latest). Each archive has a matching `.sha256` checksum file.

| Platform | Archive |
|---|---|
| macOS Apple Silicon | `inbox-macos-aarch64.tar.gz` |
| macOS Intel | `inbox-macos-x86_64.tar.gz` |
| Linux ARM64 | `inbox-linux-aarch64.tar.gz` |
| Linux x86_64 | `inbox-linux-x86_64.tar.gz` |
| Windows x86_64 | `inbox-windows-x86_64.zip` |

On macOS or Linux, extract the archive and put the binary in your `PATH`. For example, on Apple Silicon:

```bash
tar -xzf inbox-macos-aarch64.tar.gz
mkdir -p ~/.local/bin
install -m 755 inbox-macos-aarch64/inbox ~/.local/bin/inbox
inbox --version
```

Ensure `~/.local/bin` is in your `PATH`. On Windows, extract the ZIP, add the folder containing `inbox.exe` to `PATH`, and run `inbox --version` in PowerShell.

With Rust 1.89 or newer, clone the repository and use the build or install script:

```bash
git clone https://github.com/amcones/inbox-cli
cd inbox-cli
./scripts/build.sh
./scripts/install.sh
```

The Unix installer puts the binary in `~/.local/bin` and installs completion for the detected Bash, Zsh, or Fish shell. Override these choices with `--bin-dir` and `--shell`; run `./scripts/install.sh --help` for details. On Windows:

```powershell
git clone https://github.com/amcones/inbox-cli
Set-Location inbox-cli
.\scripts\build.ps1
.\scripts\install.ps1
```

The PowerShell installer defaults to `%LOCALAPPDATA%\Programs\inbox` and writes `inbox-completion.ps1` beside the executable. Both installers report when their binary directory still needs to be added to `PATH`. To install a published source revision directly with Cargo, use `cargo install --git https://github.com/amcones/inbox-cli --tag v0.4.0 --locked`.

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
| Read without counting a view | `inbox show <ID> --no-track` |
| Full usage | `inbox --help` |

ASCII letters in tags are normalized to lowercase, so `Rust`, `RUST`, and `rust` are the same tag. Duplicates and leading `#` are also normalized. A `#word` inside the body is ordinary text. Only successful `show` output counts as a view, including output redirected to a file. Lists, searches, and reviews do not count. IDs can be unique prefixes of at least four characters. Each note supports up to 1 MiB of text and 64 tags (128 bytes each).

## Command completion

The install scripts generate completion automatically. For a manual installation, generate completion for your shell once, then restart the shell. For Zsh:

```bash
mkdir -p ~/.zfunc
inbox completions zsh > ~/.zfunc/_inbox
echo 'fpath=(~/.zfunc $fpath)' >> ~/.zshrc
echo 'autoload -Uz compinit && compinit' >> ~/.zshrc
```

For Bash, source the generated script from your shell configuration. Fish and PowerShell can load the generated script directly:

```bash
inbox completions bash > ~/.inbox-completion.bash
echo 'source ~/.inbox-completion.bash' >> ~/.bashrc
mkdir -p ~/.config/fish/completions
inbox completions fish > ~/.config/fish/completions/inbox.fish
inbox completions powershell > inbox-completion.ps1
```

For PowerShell, dot-source `inbox-completion.ps1` from your profile. Supported shell names are `bash`, `zsh`, `fish`, and `powershell`.

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

For `delete range`, omitted year, month, and day default to the current local date; omitted hours default to `0 24`. Two-digit years mean 2000–2099. Matching uses each note's recorded date and hour. In this command, `-y` means year and `-h` means hours: use `--yes` to skip confirmation and `--help` for help.

Bulk deletion asks for `yes`, `y`, `是`, or `确认`; other input or EOF cancels. `--yes` skips the prompt for automation. Notes added while you confirm are preserved. Deletion clears view history; restoring brings back the body, tags, ID, and original creation time, but not past views. **Emptying trash is permanent.**

## Data, language, and backup

Notes live in `~/inbox/YYYY/MM/YYYY-MM-DD.md`. Change the location with `--dir` or `INBOX_DIR`; the command-line option takes precedence.

```bash
inbox --dir ~/Notes/inbox add 'An idea'
inbox --lang en --help
inbox --lang zh --help
```

Language precedence is `--lang` > `INBOX_LANG` > system language. Choose `auto`, `en`, or `zh`; unsupported system languages fall back to English. Body text and tags are never translated. Since v0.3.0, use `add` instead of the old `-m` capture option. For body text starting with a dash, put options first and use `--`, for example `inbox add -- '--an idea'`.

**Back up the entire data directory, including `.inbox`.** It holds view history and trash. Stop inbox commands before copying a backup. Hidden files are not encrypted. Avoid using older versions that do not support trash on the same library.

If a command reports damaged data, stop writing, make a backup, and run `inbox doctor`. It checks notes, view history, and trash; it does not guess how to repair corrupted content. For manual Markdown edits, preserve metadata and record markers and avoid concurrent writes. See the [storage and recovery guide](docs/FORMAT.md).

## Project

[Development and contributions](CONTRIBUTE.md) · [Performance measurements](docs/BENCHMARK.md) · [Architecture assessment](docs/ARCHITECTURE.md) · [Changelog](CHANGELOG.md) · [MIT license](LICENSE)
