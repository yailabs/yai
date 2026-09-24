#!/bin/sh
# Explicit opt-in installation, separate from backend/CLI builds.
set -eu
script=$(readlink -f -- "$0")
repo_root=$(CDPATH= cd -- "$(dirname -- "$script")/../.." && pwd)
if [ ! -x "$repo_root/studio/src-tauri/target/release/yai-studio" ]; then
    printf '%s\n' 'Build Studio first: npm run desktop:build -- -- --locked' >&2
    exit 127
fi
bindir="${PREFIX:-$HOME/.local}/bin"
mkdir -p -- "$bindir"
if [ -d "$bindir/yai-studio" ]; then
    printf '%s\n' "Refusing to replace directory: $bindir/yai-studio" >&2
    exit 1
fi
ln -sfnT -- "$repo_root/tools/shell/yai-studio.sh" "$bindir/yai-studio"
printf 'Installed: %s/yai-studio\nRun: yai-studio\n' "$bindir"
