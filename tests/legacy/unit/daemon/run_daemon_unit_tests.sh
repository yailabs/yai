#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
OUT_DIR="$ROOT/build/test/unit_daemon"
mkdir -p "$OUT_DIR"

cc -Wall -Wextra -std=c11 -O2 \
  -I"$ROOT/include" \
  "$ROOT/tests/legacy/unit/daemon/test_daemon_minimal.c" \
  "$ROOT/sys/daemon/bindings/actions.c" \
  "$ROOT/sys/daemon/bindings/runtime_binding.c" \
  "$ROOT/sys/daemon/yai-daemond/runtime/runtime_services.c" \
  -o "$OUT_DIR/daemon_unit_tests"

"$OUT_DIR/daemon_unit_tests"
echo "daemon_unit_tests: ok"
