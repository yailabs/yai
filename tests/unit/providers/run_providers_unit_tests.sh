#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
OUT_DIR="$ROOT/build/test/unit_providers"
mkdir -p "$OUT_DIR"

cc -Wall -Wextra -std=c11 -O2 \
  -I"$ROOT/include" -I"$ROOT/include/yai" \
  "$ROOT/tests/unit/providers/test_registry_selection.c" \
  "$ROOT/lib/network/providers/registry.c" \
  "$ROOT/lib/network/providers/policy.c" \
  "$ROOT/lib/network/providers/selection.c" \
  -o "$OUT_DIR/providers_unit_tests"

"$OUT_DIR/providers_unit_tests"
echo "providers_unit_tests: ok"
