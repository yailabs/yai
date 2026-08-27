#!/bin/sh
set -eu

# YAI - source surface documentation cleanliness guard
#
# Purpose: keep environment roots and module-per-noun documentation authority
# out of the active source tree without claiming a future implementation owner.

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)

for dir in venv .venv env ENV; do
  if [ -e "$ROOT/$dir" ]; then
    printf 'local environment root found in repo: %s\n' "$dir" >&2
    exit 1
  fi
done

for dir in system/ingest include/yai/ingest; do
  if [ -e "$ROOT/$dir" ]; then
    printf 'README-only ingest placeholder root must stay absent: %s\n' "$dir" >&2
    exit 1
  fi
done

for file in system/README.md engine/README.md include/yai/README.md cmd/README.md; do
  if [ ! -f "$ROOT/$file" ]; then
    printf 'source map missing: %s\n' "$file" >&2
    exit 1
  fi
  if ! grep -Fq 'Authority:' "$ROOT/$file"; then
    printf 'source map authority missing: %s\n' "$file" >&2
    exit 1
  fi
done

if find "$ROOT/include/yai" "$ROOT/system" -mindepth 2 -name README.md -print -quit | grep -q .; then
  find "$ROOT/include/yai" "$ROOT/system" -mindepth 2 -name README.md -print >&2
  printf 'module-per-noun README found below a source map\n' >&2
  exit 1
fi

printf 'check-source-surface-clean: ok\n'
