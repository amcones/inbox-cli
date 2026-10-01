#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
project_dir="$(cd -- "$script_dir/.." && pwd)"
bin_dir="${INBOX_BIN_DIR:-${HOME:?HOME is required}/.local/bin}"
shell_name="auto"
skip_build=false
package_dir=""
shell_config=""

usage() {
    cat <<'EOF'
Usage: scripts/install.sh [--bin-dir <directory>] [--shell <auto|bash|zsh|fish|none>] [--no-build]

Build and install inbox for the current user. INBOX_BIN_DIR changes the
default binary directory (~/.local/bin).
EOF
}

while (($#)); do
    case "$1" in
        --bin-dir)
            [[ $# -ge 2 && -n "$2" ]] || { echo "install.sh: --bin-dir requires a value" >&2; exit 2; }
            bin_dir="$2"
            shift 2
            ;;
        --shell)
            [[ $# -ge 2 && -n "$2" ]] || { echo "install.sh: --shell requires a value" >&2; exit 2; }
            shell_name="$2"
            shift 2
            ;;
        --no-build)
            skip_build=true
            shift
            ;;
        --package-dir)
            [[ $# -ge 2 && -n "$2" ]] || { echo "install.sh: --package-dir requires a value" >&2; exit 2; }
            package_dir="$2"
            skip_build=true
            shift 2
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo "install.sh: unknown argument: $1" >&2
            usage >&2
            exit 2
            ;;
    esac
done

if [[ "$shell_name" == auto ]]; then
    shell_name="$(basename -- "${SHELL:-}")"
    case "$shell_name" in
        bash|zsh|fish) ;;
        *) shell_name="none" ;;
    esac
fi
case "$shell_name" in
    bash|zsh|fish|none) ;;
    *) echo "install.sh: shell must be auto, bash, zsh, fish, or none" >&2; exit 2 ;;
esac

if [[ -n "$package_dir" ]]; then
    binary="$package_dir/inbox"
    asset_dir="$package_dir"
else
    binary="$project_dir/target/release/inbox"
    asset_dir="$script_dir"
fi
if [[ "$skip_build" == false ]]; then
    "$script_dir/build.sh" >/dev/null
elif [[ ! -x "$binary" ]]; then
    echo "install.sh: $binary does not exist; run without --no-build" >&2
    exit 1
fi

mkdir -p "$bin_dir"
install -m 755 "$binary" "$bin_dir/inbox"
install -m 755 "$asset_dir/update.sh" "$bin_dir/inbox-update"

completion_path=""
case "$shell_name" in
    bash)
        completion_path="${XDG_DATA_HOME:-$HOME/.local/share}/bash-completion/completions/inbox"
        ;;
    zsh)
        completion_path="$HOME/.zfunc/_inbox"
        ;;
    fish)
        completion_path="${XDG_CONFIG_HOME:-$HOME/.config}/fish/completions/inbox.fish"
        ;;
esac
if [[ -n "$completion_path" ]]; then
    mkdir -p "$(dirname -- "$completion_path")"
    cp "$asset_dir/completion.$shell_name" "$completion_path"
fi

append_once() {
    local line="$1"
    local file="$2"
    mkdir -p "$(dirname -- "$file")"
    touch "$file"
    grep -Fqx -- "$line" "$file" 2>/dev/null || printf '%s\n' "$line" >> "$file"
}

if [[ "$shell_name" != none ]]; then
    printf -v quoted_bin '%q' "$bin_dir"
    case "$shell_name" in
        bash)
            shell_config="${BASH_ENV:-$HOME/.bashrc}"
            append_once "export PATH=$quoted_bin:\$PATH" "$shell_config"
            printf -v quoted_completion '%q' "$completion_path"
            append_once "source $quoted_completion" "$shell_config"
            ;;
        zsh)
            shell_config="${ZDOTDIR:-$HOME}/.zshrc"
            append_once "export PATH=$quoted_bin:\$PATH" "$shell_config"
            append_once 'fpath=(~/.zfunc $fpath)' "$shell_config"
            append_once 'autoload -Uz compinit && compinit' "$shell_config"
            ;;
        fish)
            shell_config="${XDG_CONFIG_HOME:-$HOME/.config}/fish/config.fish"
            append_once "fish_add_path '$bin_dir'" "$shell_config"
            ;;
    esac
fi

printf 'Installed inbox to %s\n' "$bin_dir/inbox"
printf 'Installed updater to %s\n' "$bin_dir/inbox-update"
if [[ -n "$completion_path" ]]; then
    printf 'Installed %s completion to %s\n' "$shell_name" "$completion_path"
fi
if [[ "$shell_name" == zsh ]]; then
    printf 'Updated shell configuration: %s\n' "$shell_config"
elif [[ -n "$shell_config" ]]; then
    printf 'Updated shell configuration: %s\n' "$shell_config"
fi
case ":$PATH:" in
    *":$bin_dir:"*) ;;
    *) printf 'Add %s to PATH before using inbox.\n' "$bin_dir" ;;
esac
