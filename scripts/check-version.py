#!/usr/bin/env python3
"""Check that release version sources agree."""

from __future__ import annotations

import re
import sys
import tomllib
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def fail(message: str) -> None:
    print(f"version check: {message}", file=sys.stderr)
    raise SystemExit(1)


manifest = tomllib.loads((ROOT / "Cargo.toml").read_text(encoding="utf-8"))
version = manifest["package"]["version"]

lock = tomllib.loads((ROOT / "Cargo.lock").read_text(encoding="utf-8"))
lock_versions = {
    package["version"]
    for package in lock["package"]
    if package["name"] == manifest["package"]["name"]
}
if lock_versions != {version}:
    fail(
        f"Cargo.toml is {version}, but Cargo.lock contains "
        f"{', '.join(sorted(lock_versions)) or 'no package version'}"
    )

changelog = (ROOT / "CHANGELOG.md").read_text(encoding="utf-8")
if not re.search(rf"^## {re.escape(version)}(?:\s|$)", changelog, re.MULTILINE):
    fail(f"CHANGELOG.md has no {version} release heading")

for readme_name in ("README.md", "README.zh-CN.md"):
    readme = (ROOT / readme_name).read_text(encoding="utf-8")
    if f"--tag v{version}" not in readme or f"--version v{version}" not in readme:
        fail(f"{readme_name} does not use v{version} in release install/update examples")

print(f"version check: v{version} is consistent")
