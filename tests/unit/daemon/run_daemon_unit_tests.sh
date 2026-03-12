#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
OUT_DIR="$ROOT/build/test/unit_daemon"
mkdir -p "$OUT_DIR"

cc -Wall -Wextra -std=c11 -O2 \
  -I"$ROOT/include" -I"$ROOT/include/yai" \
  "$ROOT/tests/unit/daemon/test_daemon_minimal.c" \
  "$ROOT/lib/daemon/actions.c" \
  "$ROOT/lib/daemon/runtime_binding.c" \
  "$ROOT/lib/runtime/local/services.c" \
  -o "$OUT_DIR/daemon_unit_tests"

"$OUT_DIR/daemon_unit_tests"
echo "daemon_unit_tests: ok"
