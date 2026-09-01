#!/bin/sh

set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
binary="$repo_root/target/debug/yai"

if [ ! -x "$binary" ]; then
    printf '%s\n' "yai: local product binary is not built: $binary" >&2
    printf '%s\n' "yai: run 'make build-rust' from the repository root" >&2
    exit 127
fi

exec "$binary" "$@"
