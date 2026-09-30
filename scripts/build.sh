#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
project_dir="$(cd -- "$script_dir/.." && pwd)"
target=""

usage() {
    cat <<'EOF'
Usage: scripts/build.sh [--target <rust-target>]

Build the optimized inbox binary with locked dependencies.
EOF
}

while (($#)); do
    case "$1" in
        --target)
            [[ $# -ge 2 && -n "$2" ]] || { echo "build.sh: --target requires a value" >&2; exit 2; }
            target="$2"
            shift 2
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo "build.sh: unknown argument: $1" >&2
            usage >&2
            exit 2
            ;;
    esac
done

cd "$project_dir"
args=(build --release --locked)
binary="$project_dir/target/release/inbox"
if [[ -n "$target" ]]; then
    args+=(--target "$target")
    binary="$project_dir/target/$target/release/inbox"
    [[ "$target" == *windows* ]] && binary+=".exe"
fi

cargo "${args[@]}"
printf '%s\n' "$binary"
