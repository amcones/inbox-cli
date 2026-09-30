# Contributing to inbox

[User guide](README.md) · [中文使用指南](README.zh-CN.md) · [Agent instructions](AGENTS.md)

## Workflow

`develop` integrates work for the next version; `main` is the stable release branch and the repository's default branch. Inspect the working tree first and preserve unrelated changes. Start topic branches from the current `origin/develop`.

| PR target | Source | Merge method |
|---|---|---|
| `develop` | `feat/*`, `fix/*`, `docs/*`, `refactor/*`, `chore/*`, `ci/*`, `test/*`, `perf/*`, `build/*`, `revert/*` (including forks) | Squash for ordinary changes |
| `main` | This repository's `develop` only | Merge commit |
| `develop` | This repository's `main`, or a topic branch merging `main` into `develop` | Merge commit for history synchronization |

After each release merge, synchronize `main` back into `develop` before the next release PR. A direct `main` → `develop` PR is suitable when it can pass the required up-to-date check. If GitHub reports it behind, create a `chore/*` branch from the latest `develop`, merge `origin/main` into that branch, and open a PR to `develop`. Resolve conflicts there and merge the synchronization PR with **Create a merge commit**. Squash or rebase would discard the ancestry being restored. Do not reset or force-push either long-lived branch.

Both branch rulesets require a PR, resolved conversations, and the existing `CI gate` with the up-to-date requirement. Both block deletion and force pushes, have no bypass actors, and currently require zero approvals for solo maintenance. `main` permits only merge commits; `develop` permits squash and merge commits, with merge reserved for synchronization. Neither requires linear history. Rulesets select methods by target branch, so maintainers must select squash for ordinary topic PRs and merge for synchronization PRs.

**The maintainer merges manually.** Stop after submitting the PR and reporting its checks. Do not merge, enable auto-merge, bypass reviews, or change branch protection. Do not create tags or publish releases without an explicit release request. Do not force-push shared history without authorization.

## Build and check

The project uses Rust edition 2024 and supports Rust 1.89 or newer. Runtime dependencies are limited to `jiff` (time zones), `lexopt` (arguments), `serde` / `serde_json` (metadata), and `uuid` (IDs), plus platform bindings (`libc` on Unix and `windows-sys` on Windows) for immediate single-key confirmation.

```bash
cargo build --release --locked
./target/release/inbox --help
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
cargo +1.89.0 check --all-targets --locked
```

The supported local wrappers are `scripts/build.sh` and `scripts/install.sh` on macOS/Linux, with `scripts/build.ps1` and `scripts/install.ps1` on Windows. CI exercises both the build and no-rebuild installation paths. Keep their builds equivalent to `cargo build --release --locked`.

The last command requires the 1.89.0 toolchain; CI also runs it. Use temporary data directories for manual checks, never a user's real inbox. Documentation-only work needs link and example verification, not extra behavior tests. Runtime changes need relevant regression tests and the Rust checks above.

## Code and data contracts

| Area | Files |
|---|---|
| Arguments, output, language | `src/cli.rs`, `src/main.rs`, `src/i18n.rs` |
| Note validation and Markdown | `src/model.rs`, `src/markdown.rs` |
| Files and locks | `src/storage.rs` |
| Queries and ranking | `src/query.rs`, `src/ranking.rs`, `src/views.rs` |
| Mutations and recovery | `src/editing.rs`, `src/deletion.rs`, `src/trash.rs` |
| End-to-end regression tests | `tests/cli.rs` |

Keep capture independent of library size: adding reads at most 256 trailing bytes of the current day file, appends, and synchronizes the write. Recent lists use Top-N selection and stop when older files cannot affect results; directory discovery still enumerates the library. ID lookup, tags, priority sorting, and searches with insufficient matches scan the full library.

The priority formula is `0.65 × 2^(-age_days / 14) + 0.35 × views / (views + 5)`. Future age is clamped to zero; ties use creation time then full ID, descending. Only successful `show` output records a view.

Respect exclusive write/shared read locks. Default `show` holds an exclusive lock through output and tracking. Editing replaces a complete day file; deletion journals replacements and trash copies for recovery. Never discard damaged bytes or delete committed recovery manifests. Preserve recovery compatibility and document changes in [FORMAT.md](docs/FORMAT.md).

Append failures can leave incomplete records; fail explicitly on subsequent reads/writes instead of silently repairing them. `sync_all` and Unix parent-directory syncs are part of durability behavior, not benchmark options to remove. New Unix files/directories use `0600`/`0700`; existing permissions are not rewritten. External editors and sync tools may ignore locks.

Application exit codes: `0` success, `1` runtime/content/storage failure, `2` argument failure. A broken output pipe is treated as normal termination and does not count an unsuccessful view. A write may have succeeded before output or sync reports an error; retrying blindly can duplicate content.

Language precedence is `--lang` > `INBOX_LANG` > `LC_ALL`, `LC_MESSAGES`, `LANG` (first nonempty value). macOS falls back to its preferred language when those variables are absent. Translate application messages, not note data or underlying OS/library diagnostics.

## Performance work

```bash
cargo build --release --locked
python3 scripts/benchmark.py --samples 30 --output docs/BENCHMARK.md
```

The benchmark creates disposable libraries and includes process launch, operation, and exit. It uses warm OS file caches and excludes terminal rendering; do not describe it as cold-disk latency. Record the binary version, platform, sample count, median, P95, memory, and workload alongside conclusions.

`release` uses size optimization, LTO, one codegen unit, stripping, and abort-on-panic. Compare `cargo build --profile speed` and `cargo build --profile small` when justified. Evaluate startup, parsing, scanning, directory discovery, log replay, and synchronization separately before blaming disk I/O. See the [interactive-process assessment](docs/ARCHITECTURE.md).

## CI and releases

PRs targeting `main` or `develop` run a source-branch policy, formatting, Clippy, tests, release builds, and script checks on macOS ARM64/x86_64, Linux ARM64/x86_64, and Windows x86_64. A separate `Rust 1.89` job checks the MSRV. A PR targeting `main` must come from this repository's `develop`, not a fork with a same-named branch. Topic branches target `develop`; this repository's `main` is also allowed there for release synchronization. The policy reruns when a PR is retargeted. The platform jobs remain parallel for coverage, but the repository rulesets require only the final `CI gate` check; it fails unless the source policy, MSRV, and every platform job succeed. The PR workflow builds binaries but does not upload PR build artifacts.

After the maintainer merges release preparation, an explicitly authorized `vMAJOR.MINOR.PATCH` tag triggers the release workflow. It verifies the Cargo version, changelog entry, and membership in `main`, then builds five native archives with SHA-256 files. Publication creates a draft, uploads all ten assets, then makes the release public. Read-only permissions are the default; only publication receives contents write access.

Release preparation must keep `Cargo.toml`, `Cargo.lock`, `CHANGELOG.md`, and both README installation examples consistent. A new feature PR alone is not authorization to tag or release.

## Documentation

Keep [README.md](README.md) in English and [docs/README.zh-CN.md](README.zh-CN.md) in Chinese, with reciprocal links and equivalent command semantics. Keep onboarding and everyday use in those files; place implementation details here or in `docs/`. Describe breaking CLI changes and recovery limitations explicitly. Agent-specific workflow rules live in [AGENT.md](AGENTS.md).
