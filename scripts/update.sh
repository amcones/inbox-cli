#!/usr/bin/env bash
set -euo pipefail

repository="amcones/inbox-cli"
version="latest"
bin_dir=""
release_base="${INBOX_RELEASE_BASE_URL:-https://github.com/$repository/releases}"

usage() {
    cat <<'EOF'
Usage: inbox-update [--version <vMAJOR.MINOR.PATCH>] [--bin-dir <directory>]

Download a verified GitHub Release and replace the locally installed inbox.
The latest release is used unless --version is supplied.
EOF
}

while (($#)); do
    case "$1" in
        --version)
            [[ $# -ge 2 && "$2" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]] || {
                echo "inbox-update: --version requires vMAJOR.MINOR.PATCH" >&2
                exit 2
            }
            version="$2"
            shift 2
            ;;
        --bin-dir)
            [[ $# -ge 2 && -n "$2" ]] || {
                echo "inbox-update: --bin-dir requires a directory" >&2
                exit 2
            }
            bin_dir="$2"
            shift 2
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo "inbox-update: unknown argument: $1" >&2
            usage >&2
            exit 2
            ;;
    esac
done

case "$(uname -s):$(uname -m)" in
    Darwin:arm64|Darwin:aarch64) asset="inbox-macos-aarch64" ;;
    Darwin:x86_64) asset="inbox-macos-x86_64" ;;
    Linux:arm64|Linux:aarch64) asset="inbox-linux-aarch64" ;;
    Linux:x86_64|Linux:amd64) asset="inbox-linux-x86_64" ;;
    *) echo "inbox-update: this operating system or architecture is not supported" >&2; exit 1 ;;
esac

if [[ -n "$bin_dir" ]]; then
    installed="$bin_dir/inbox"
else
    installed="$(command -v inbox || true)"
    [[ -n "$installed" ]] || {
        echo "inbox-update: inbox is not on PATH; use --bin-dir to identify its installation directory" >&2
        exit 1
    }
    [[ ! -L "$installed" ]] || {
        echo "inbox-update: $installed is a symbolic link; use the original package manager or --bin-dir" >&2
        exit 1
    }
    bin_dir="$(cd -- "$(dirname -- "$installed")" && pwd)"
    installed="$bin_dir/inbox"
fi
[[ -f "$installed" ]] || { echo "inbox-update: $installed is not an installed inbox binary" >&2; exit 1; }

command -v curl >/dev/null || { echo "inbox-update: curl is required" >&2; exit 1; }
command -v tar >/dev/null || { echo "inbox-update: tar is required" >&2; exit 1; }

archive="$asset.tar.gz"
if [[ "$version" == latest ]]; then
    download_base="$release_base/latest/download"
else
    download_base="$release_base/download/$version"
fi
work_dir="$(mktemp -d "${TMPDIR:-/tmp}/inbox-update.XXXXXX")"
staged=""
cleanup() {
    rm -rf -- "$work_dir"
    [[ -z "$staged" ]] || rm -f -- "$staged"
}
trap cleanup EXIT

curl --fail --location --silent --show-error "$download_base/$archive" --output "$work_dir/$archive"
curl --fail --location --silent --show-error "$download_base/$archive.sha256" --output "$work_dir/$archive.sha256"
expected="$(awk 'NR == 1 { print tolower($1) }' "$work_dir/$archive.sha256")"
if command -v sha256sum >/dev/null; then
    actual="$(sha256sum "$work_dir/$archive" | awk '{ print $1 }')"
else
    actual="$(shasum -a 256 "$work_dir/$archive" | awk '{ print $1 }')"
fi
[[ -n "$expected" && "$actual" == "$expected" ]] || {
    echo "inbox-update: checksum verification failed" >&2
    exit 1
}

tar -xzf "$work_dir/$archive" -C "$work_dir"
downloaded="$work_dir/$asset/inbox"
[[ -x "$downloaded" ]] || { echo "inbox-update: release archive does not contain inbox" >&2; exit 1; }
new_version="$($downloaded --version)"
if [[ "$version" != latest && "$new_version" != "inbox ${version#v}" ]]; then
    echo "inbox-update: downloaded $new_version, expected inbox ${version#v}" >&2
    exit 1
fi

staged="$installed.new.$$"
install -m 755 "$downloaded" "$staged"
mv -f -- "$staged" "$installed"
staged=""

for pair in \
    "completion.bash:${XDG_DATA_HOME:-$HOME/.local/share}/bash-completion/completions/inbox" \
    "completion.zsh:$HOME/.zfunc/_inbox" \
    "completion.fish:${XDG_CONFIG_HOME:-$HOME/.config}/fish/completions/inbox.fish"
do
    source_file="$work_dir/$asset/${pair%%:*}"
    destination="${pair#*:}"
    if [[ -f "$source_file" && -f "$destination" ]]; then
        cp -- "$source_file" "$destination"
    fi
done
if [[ -f "$work_dir/$asset/update.sh" && -f "$bin_dir/inbox-update" ]]; then
    install -m 755 "$work_dir/$asset/update.sh" "$bin_dir/inbox-update.new.$$"
    mv -f -- "$bin_dir/inbox-update.new.$$" "$bin_dir/inbox-update"
fi

printf 'Updated %s to %s\n' "$installed" "$new_version"
