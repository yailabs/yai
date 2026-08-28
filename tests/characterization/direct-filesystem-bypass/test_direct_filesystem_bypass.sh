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

if YAI_HOME="$runtime_home" "$YAI_BIN" carrier fs-write \
  --sandbox "$sandbox" --path "$target" --content removed-bypass >/dev/null 2>&1; then
  printf 'removed direct filesystem bypass remains product-reachable\n' >&2
  exit 1
fi
[ ! -e "$target" ]

printf '%s\n' observed >"$target"

read_output=$(YAI_HOME="$runtime_home" "$YAI_BIN" carrier fs-read \
  --sandbox "$sandbox" --path "$target")
printf '%s\n' "$read_output" | grep -F 'status: observed' >/dev/null
printf '%s\n' "$read_output" | grep -F 'bytes: 9' >/dev/null

[ ! -e "$outside" ]

if find "$runtime_home" -type f -print -quit | grep -q .; then
  printf 'direct filesystem bypass unexpectedly persisted runtime evidence\n' >&2
  exit 1
fi

printf 'direct_filesystem_bypass:product_command_removed ok\n'
printf 'filesystem_read_compatibility:observed_only ok\n'
