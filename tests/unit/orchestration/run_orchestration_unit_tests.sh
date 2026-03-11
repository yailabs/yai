#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
OUT_DIR="$ROOT/build/test/unit_exec"
mkdir -p "$OUT_DIR"

cc -Wall -Wextra -std=c11 -O2 \
  -I"$ROOT/include" -I"$ROOT/include/yai" \
  "$ROOT/tests/unit/orchestration/cortex_harness.c" \
  "$ROOT/lib/orchestration/runtime/runtime_model.c" \
  -o "$OUT_DIR/cortex_harness"

"$OUT_DIR/cortex_harness"
