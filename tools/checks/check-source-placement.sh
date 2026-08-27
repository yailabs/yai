#!/bin/sh
# YAI - source placement guard
#
# Purpose:
#   Prevent active source from returning to retired bootstrap roots.
#
# Scope:
#   Checks forbidden roots and canonical source placement.
#
# Non-goals:
#   Does not validate implementation semantics.
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)

if [ -e "$ROOT/src" ]; then
  printf 'forbidden src directory found: %s/src\n' "$ROOT" >&2
  exit 1
fi

if [ -e "$ROOT/crates" ]; then
  printf 'forbidden crates directory found: %s/crates\n' "$ROOT" >&2
  exit 1
fi

if [ -e "$ROOT/daemon" ]; then
  printf 'forbidden daemon directory found: %s/daemon\n' "$ROOT" >&2
  exit 1
fi

if [ -e "$ROOT/ctl" ]; then
  printf 'forbidden ctl directory found: %s/ctl\n' "$ROOT" >&2
  exit 1
fi

if [ -e "$ROOT/lib" ]; then
  printf 'forbidden lib directory found: %s/lib\n' "$ROOT" >&2
  exit 1
fi

bad_rs=$(find "$ROOT" -name '*.rs' ! -path "$ROOT/.venv/*" ! -path "$ROOT/build/*" ! -path "$ROOT/target/*" ! -path "$ROOT/engine/*" ! -path "$ROOT/cmd/yai/*" -print)
if [ -n "$bad_rs" ]; then
  printf '%s\n' "$bad_rs" >&2
  printf 'Rust files are only allowed under engine/ or cmd/yai/\n' >&2
  exit 1
fi

bad_c=$(find "$ROOT" -name '*.c' ! -path "$ROOT/.venv/*" ! -path "$ROOT/build/*" ! -path "$ROOT/target/*" ! -path "$ROOT/system/*" ! -path "$ROOT/cmd/yaid/*" ! -path "$ROOT/tests/*" ! -path "$ROOT/vendor/*" -print)
if [ -n "$bad_c" ]; then
  printf '%s\n' "$bad_c" >&2
  printf 'C files are only allowed under system/, cmd/yaid/, tests/ or vendor/\n' >&2
  exit 1
fi

bad_h=$(find "$ROOT" -name '*.h' ! -path "$ROOT/.venv/*" ! -path "$ROOT/build/*" ! -path "$ROOT/target/*" ! -path "$ROOT/include/yai/*" ! -path "$ROOT/vendor/*" -print)
if [ -n "$bad_h" ]; then
  printf '%s\n' "$bad_h" >&2
  printf 'Headers are only allowed under include/yai/ or vendor/\n' >&2
  exit 1
fi

printf 'check-source-placement: ok\n'
