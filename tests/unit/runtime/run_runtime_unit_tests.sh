#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
OUT_DIR="$ROOT/build/test/unit_runtime"
mkdir -p "$OUT_DIR"

cc -Wall -Wextra -std=c11 -O2 \
  -I"$ROOT/include" -I"$ROOT/include/yai" \
  "$ROOT/tests/unit/runtime/test_runtime_state_closure.c" \
  "$ROOT/lib/runtime/policy/policy_state.c" \
  "$ROOT/lib/runtime/grants/grants_state.c" \
  "$ROOT/lib/runtime/containment/containment_state.c" \
  -o "$OUT_DIR/runtime_unit_tests"

"$OUT_DIR/runtime_unit_tests"
echo "runtime_unit_tests: ok"
