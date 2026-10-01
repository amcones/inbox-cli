#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
project_dir="$(cd -- "$script_dir/.." && pwd)"
case "$(uname -s):$(uname -m)" in
    Darwin:arm64|Darwin:aarch64) asset="inbox-macos-aarch64" ;;
    Darwin:x86_64) asset="inbox-macos-x86_64" ;;
    Linux:arm64|Linux:aarch64) asset="inbox-linux-aarch64" ;;
    Linux:x86_64|Linux:amd64) asset="inbox-linux-x86_64" ;;
    *) echo "unsupported test platform" >&2; exit 1 ;;
esac

test_dir="$(mktemp -d "${TMPDIR:-/tmp}/inbox-update-test.XXXXXX")"
trap 'rm -rf -- "$test_dir"' EXIT
release_dir="$test_dir/releases/download/v0.6.2"
package_dir="$test_dir/package/$asset"
bin_dir="$test_dir/bin"
mkdir -p "$release_dir" "$package_dir" "$bin_dir"
cp "$project_dir/target/release/inbox" "$package_dir/inbox"
cp "$script_dir/update.sh" "$package_dir/update.sh"
cp "$script_dir/install.sh" "$package_dir/install.sh"
cp "$script_dir/completion.bash" "$package_dir/completion.bash"
tar -C "$test_dir/package" -czf "$release_dir/$asset.tar.gz" "$asset"
if command -v sha256sum >/dev/null; then
    (cd "$release_dir" && sha256sum "$asset.tar.gz" > "$asset.tar.gz.sha256")
else
    (cd "$release_dir" && shasum -a 256 "$asset.tar.gz" > "$asset.tar.gz.sha256")
fi
mkdir -p "$test_dir/releases/latest"
ln -s ../download/v0.6.2 "$test_dir/releases/latest/download"
touch "$bin_dir/inbox" "$bin_dir/inbox-update"
mkdir -p "$test_dir/bash-completion/completions"
touch "$test_dir/bash-completion/completions/inbox"

cp "$release_dir/$asset.tar.gz.sha256" "$release_dir/checksum.correct"
awk '{ print "0000000000000000000000000000000000000000000000000000000000000000  " $2 }' \
    "$release_dir/checksum.correct" > "$release_dir/$asset.tar.gz.sha256"
if HOME="$test_dir/home" INBOX_RELEASE_BASE_URL="file://$test_dir/releases" \
    "$script_dir/update.sh" --version v0.6.2 --bin-dir "$bin_dir" 2>/dev/null
then
    echo "updater accepted a damaged checksum" >&2
    exit 1
fi
test ! -s "$bin_dir/inbox"
mv "$release_dir/checksum.correct" "$release_dir/$asset.tar.gz.sha256"

HOME="$test_dir/home" \
XDG_DATA_HOME="$test_dir" \
INBOX_RELEASE_BASE_URL="file://$test_dir/releases" \
    "$script_dir/update.sh" --version v0.6.2 --bin-dir "$bin_dir"

test "$($bin_dir/inbox --version)" = "inbox 0.6.2"
cmp "$bin_dir/inbox-update" "$script_dir/update.sh"
cmp "$test_dir/bash-completion/completions/inbox" "$script_dir/completion.bash"
install_bin="$test_dir/install-bin"
HOME="$test_dir/install-home" INBOX_RELEASE_BASE_URL="file://$test_dir/releases" \
    "$script_dir/install-release.sh" --bin-dir "$install_bin" --shell none
test "$($install_bin/inbox --version)" = "inbox 0.6.2"
printf 'Unix updater test passed\n'
