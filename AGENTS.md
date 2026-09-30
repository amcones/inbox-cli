# Agent instructions

## Required workflow

- Read `CONTRIBUTE.md` and applicable repository instructions before editing.
- Develop on `develop`; open PRs from `develop` to `main`.
- Inspect the working tree and branch state before switching or updating. Preserve unrelated work and avoid unauthorized history rewrites.
- The user manually merges every PR. Do not merge, enable auto-merge, bypass approval, or modify repository rules to unblock yourself.
- Finish by linking the PR, describing validation and any remaining failures, and leaving the PR for the user to merge.
- Create tags or publish releases only when explicitly requested, after the user has merged the release preparation.

## Product constraints

- Keep the CLI small and fast, local Markdown as the source of truth, and capture independent of full-library scans.
- Do not add a daemon, interactive mode, persistent index, or new dependency merely because an assessment discusses it. Obtain a feature request before changing the architecture.
- Preserve file locking, durable writes, trash recovery, IDs, and timestamps. Never test destructive commands against a user's data.
- Keep Chinese and English application messages consistent. Test affected CLI behavior, malformed input, and mutation/recovery boundaries.

## Documentation and verification

- English onboarding belongs in `README.md`; Chinese onboarding in `README.zh-CN.md`; retain reciprocal links.
- Development guidance belongs in `CONTRIBUTE.md`, data contracts in `docs/FORMAT.md`, and measured performance in `docs/BENCHMARK.md`.
- Follow `CONTRIBUTE.md` for checks. For documentation-only changes, verify links, CLI examples, and stated implementation behavior.
- Separate measurements from hypotheses and proposed performance targets. Never claim warm-cache results measure physical disk latency.
