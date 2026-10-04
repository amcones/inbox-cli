#!/usr/bin/env python3
"""Build GitHub Pages with installer endpoints from their canonical sources."""

from pathlib import Path
import shutil
import subprocess
import sys


ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "dist" / "site"

subprocess.run(
    [sys.executable, str(ROOT / "scripts" / "build-site-releases.py")],
    check=True,
)
if OUTPUT.exists():
    shutil.rmtree(OUTPUT)
shutil.copytree(ROOT / "site", OUTPUT)
for extension in ("sh", "ps1"):
    shutil.copyfile(
        ROOT / "scripts" / f"install-release.{extension}",
        OUTPUT / f"install.{extension}",
    )
print(f"built {OUTPUT.relative_to(ROOT)} with release installers")
