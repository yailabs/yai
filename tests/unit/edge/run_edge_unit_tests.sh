#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
OUT_DIR="$ROOT/build/test/unit_edge"
mkdir -p "$OUT_DIR"

cc -Wall -Wextra -std=c11 -O2 \
  -I"$ROOT/include" -I"$ROOT/include/yai" \
  "$ROOT/tests/unit/edge/test_edge_minimal.c" \
  "$ROOT/lib/edge/action_point.c" \
  "$ROOT/lib/edge/edge_binding.c" \
  "$ROOT/lib/edge/edge_services.c" \
  -o "$OUT_DIR/edge_unit_tests"

"$OUT_DIR/edge_unit_tests"
echo "edge_unit_tests: ok"
