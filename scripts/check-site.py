#!/usr/bin/env python3
"""Validate the dependency-free introducing site without network access."""

from html.parser import HTMLParser
from pathlib import Path
import re
import subprocess
import sys


ROOT = Path(__file__).resolve().parents[1]
SITE = ROOT / "site"


class SiteParser(HTMLParser):
    def __init__(self) -> None:
        super().__init__()
        self.ids: list[str] = []
        self.local_refs: list[str] = []
        self.translation_keys: set[str] = set()

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        values = dict(attrs)
        if values.get("id"):
            self.ids.append(values["id"])
        if values.get("data-i18n"):
            self.translation_keys.add(values["data-i18n"])
        if values.get("data-i18n-aria"):
            self.translation_keys.add(values["data-i18n-aria"])
        for name in ("href", "src"):
            ref = values.get(name)
            if ref and not ref.startswith(("#", "http://", "https://", "mailto:")):
                self.local_refs.append(ref.split("?", 1)[0].split("#", 1)[0])


def fail(message: str) -> None:
    print(f"site check: {message}", file=sys.stderr)
    raise SystemExit(1)


html = (SITE / "index.html").read_text(encoding="utf-8")
script = (SITE / "script.js").read_text(encoding="utf-8")
changelog = (ROOT / "CHANGELOG.md").read_text(encoding="utf-8")
subprocess.run(
    [sys.executable, str(ROOT / "scripts" / "build-site-releases.py"), "--check"],
    check=True,
)
parser = SiteParser()
parser.feed(html)

duplicates = sorted({item for item in parser.ids if parser.ids.count(item) > 1})
if duplicates:
    fail(f"duplicate HTML ids: {', '.join(duplicates)}")

missing_files = sorted(ref for ref in parser.local_refs if not (SITE / ref).is_file())
if missing_files:
    fail(f"missing local files: {', '.join(missing_files)}")

for key in sorted(parser.translation_keys):
    count = len(re.findall(rf"\b{re.escape(key)}\s*:", script))
    if count != 2:
        fail(f"translation key {key!r} occurs {count} times; expected English and Chinese")

for required in ("assets/inbox-icon.svg", "assets/favicon.svg", "releases.js", ".nojekyll"):
    path = SITE / required
    if not path.is_file() or (path.suffix and path.stat().st_size == 0):
        fail(f"required artifact is missing or empty: {required}")

release_headings = re.findall(
    r"^## (\d+\.\d+\.\d+) — (\d{4}-\d{2}-\d{2})$", changelog, re.MULTILINE
)
if len(release_headings) < 4:
    fail("CHANGELOG.md must keep at least four parseable release headings")
if "window.INBOX_RELEASES" not in (SITE / "releases.js").read_text(encoding="utf-8"):
    fail("site release history is missing")

print(
    f"site check: {len(parser.local_refs)} local references, "
    f"{len(parser.translation_keys)} translated strings, {len(parser.ids)} ids, "
    f"and {len(release_headings)} changelog releases are valid"
)
