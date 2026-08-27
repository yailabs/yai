#!/bin/sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
YAI_BIN="$ROOT/target/debug/yai"
test_root=$(mktemp -d /tmp/yai-direct-filesystem-characterization.XXXXXX)
trap 'rm -rf "$test_root"' EXIT INT TERM

sandbox="$test_root/sandbox"
target="$sandbox/output.txt"
outside="$test_root/outside.txt"
runtime_home="$test_root/runtime-home"
mkdir -p "$sandbox" "$runtime_home"

write_output=$(YAI_HOME="$runtime_home" "$YAI_BIN" carrier fs-write \
  --sandbox "$sandbox" --path "$target" --content current-bypass)
printf '%s\n' "$write_output" | grep -F 'status: executed' >/dev/null
[ "$(sed -n '1p' "$target")" = "current-bypass" ]

read_output=$(YAI_HOME="$runtime_home" "$YAI_BIN" carrier fs-read \
  --sandbox "$sandbox" --path "$target")
printf '%s\n' "$read_output" | grep -F 'status: observed' >/dev/null
printf '%s\n' "$read_output" | grep -F 'bytes: 14' >/dev/null

if YAI_HOME="$runtime_home" "$YAI_BIN" carrier fs-write \
  --sandbox "$sandbox" --path "$outside" --content rejected >/dev/null 2>&1; then
  printf 'direct filesystem bypass accepted an out-of-sandbox path\n' >&2
  exit 1
fi
[ ! -e "$outside" ]

if find "$runtime_home" -type f -print -quit | grep -q .; then
  printf 'direct filesystem bypass unexpectedly persisted runtime evidence\n' >&2
  exit 1
fi

printf 'direct_filesystem_bypass:current_write_read_behavior ok\n'
printf 'direct_filesystem_bypass:no_admission_or_receipt_residue ok\n'
