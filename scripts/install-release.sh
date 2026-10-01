#!/usr/bin/env bash
set -euo pipefail

release_base="${INBOX_RELEASE_BASE_URL:-https://github.com/amcones/inbox-cli/releases}"
case "$(uname -s):$(uname -m)" in
    Darwin:arm64|Darwin:aarch64) asset="inbox-macos-aarch64" ;;
    Darwin:x86_64) asset="inbox-macos-x86_64" ;;
    Linux:arm64|Linux:aarch64) asset="inbox-linux-aarch64" ;;
    Linux:x86_64|Linux:amd64) asset="inbox-linux-x86_64" ;;
    *) echo "inbox-install: this operating system or architecture is not supported" >&2; exit 1 ;;
esac
command -v curl >/dev/null || { echo "inbox-install: curl is required" >&2; exit 1; }
command -v tar >/dev/null || { echo "inbox-install: tar is required" >&2; exit 1; }

work_dir="$(mktemp -d "${TMPDIR:-/tmp}/inbox-install.XXXXXX")"
trap 'rm -rf -- "$work_dir"' EXIT
archive="$asset.tar.gz"
download_base="$release_base/latest/download"
curl --fail --location --silent --show-error "$download_base/$archive" --output "$work_dir/$archive"
curl --fail --location --silent --show-error "$download_base/$archive.sha256" --output "$work_dir/$archive.sha256"
expected="$(awk 'NR == 1 { print tolower($1) }' "$work_dir/$archive.sha256")"
if command -v sha256sum >/dev/null; then
    actual="$(sha256sum "$work_dir/$archive" | awk '{ print $1 }')"
else
    actual="$(shasum -a 256 "$work_dir/$archive" | awk '{ print $1 }')"
fi
[[ -n "$expected" && "$actual" == "$expected" ]] || { echo "inbox-install: checksum verification failed" >&2; exit 1; }
tar -xzf "$work_dir/$archive" -C "$work_dir"
[[ -x "$work_dir/$asset/inbox" && -f "$work_dir/$asset/install.sh" ]] || {
    echo "inbox-install: release archive is incomplete" >&2
    exit 1
}
bash "$work_dir/$asset/install.sh" --package-dir "$work_dir/$asset" "$@"
