#!/usr/bin/env python3
"""Generate the website's bilingual release data from CHANGELOG.md."""

import argparse
import json
from pathlib import Path
import re
import sys


ROOT = Path(__file__).resolve().parents[1]
CHANGELOG = ROOT / "CHANGELOG.md"
OUTPUT = ROOT / "site" / "releases.js"
LIMIT = 4
HEADING = re.compile(r"^## (\d+\.\d+\.\d+) — (\d{4}-\d{2}-\d{2})$")
BULLET = re.compile(r"^- (.*?)\s*<!-- zh: (.*?) -->$")


def plain(text: str) -> str:
    return text.replace("`", "")


def parse_changelog(markdown: str) -> list[dict[str, object]]:
    releases: list[dict[str, object]] = []
    current: dict[str, object] | None = None
    for line in markdown.splitlines():
        heading = HEADING.match(line)
        if heading:
            if len(releases) == LIMIT:
                break
            current = {
                "version": heading.group(1),
                "date": heading.group(2),
                "changes": {"en": [], "zh": []},
            }
            releases.append(current)
            continue
        if current is None or not line.startswith("- "):
            continue
        bullet = BULLET.match(line)
        if bullet is None:
            raise ValueError(
                f"{current['version']} website bullet needs an inline Chinese translation"
            )
        changes = current["changes"]
        assert isinstance(changes, dict)
        changes["en"].append(plain(bullet.group(1)))
        changes["zh"].append(plain(bullet.group(2)))
    if len(releases) != LIMIT:
        raise ValueError(f"expected {LIMIT} releases, found {len(releases)}")
    return releases


def render(releases: list[dict[str, object]]) -> str:
    data = json.dumps(releases, ensure_ascii=False, indent=2)
    return f"// Generated from CHANGELOG.md. Do not edit by hand.\nwindow.INBOX_RELEASES = {data};\n"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    expected = render(parse_changelog(CHANGELOG.read_text(encoding="utf-8")))
    if args.check:
        if not OUTPUT.is_file() or OUTPUT.read_text(encoding="utf-8") != expected:
            print("site release data is stale; run scripts/build-site-releases.py", file=sys.stderr)
            return 1
        print("site release data matches CHANGELOG.md")
        return 0
    OUTPUT.write_text(expected, encoding="utf-8")
    print(f"generated {OUTPUT.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
