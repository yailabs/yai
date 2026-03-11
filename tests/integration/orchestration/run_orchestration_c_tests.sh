#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
OUT_ROOT="$ROOT/build/test/knowledge"
OBJ_DIR="$OUT_ROOT/obj"
BIN_DIR="$OUT_ROOT/bin"
LMDB_LIBS="$(pkg-config --libs liblmdb 2>/dev/null || true)"
HIREDIS_LIBS="$(pkg-config --libs hiredis 2>/dev/null || true)"
DUCKDB_LIBS=""
if [[ -f /opt/homebrew/opt/duckdb/lib/libduckdb.dylib ]]; then
  DUCKDB_LIBS="-L/opt/homebrew/opt/duckdb/lib -lduckdb"
fi
mkdir -p "$OBJ_DIR" "$BIN_DIR"

if [[ ! -f "$OBJ_DIR/lib/runtime/lifecycle/runtime_capabilities.o" ]]; then
  "$ROOT/tests/unit/knowledge/run_knowledge_unit_tests.sh" >/dev/null
fi
make -C "$ROOT" data graph orchestration >/dev/null

OBJS=()
while IFS= read -r obj; do
  OBJS+=("$obj")
done < <(find "$OBJ_DIR/lib" -type f -name '*.o' | sort)

for t in test_runtime_primary test_orchestration_transport_smoke; do
  cc -Wall -Wextra -std=c11 -O2 -I"$ROOT/include" -I"$ROOT/include/yai" \
    "$ROOT/tests/integration/orchestration/$t.c" "${OBJS[@]}" \
    "$ROOT/build/lib/libyai_orchestration.a" \
    "$ROOT/build/lib/libyai_data.a" \
    "$ROOT/build/lib/libyai_graph.a" \
    "$ROOT/build/lib/libyai_edge.a" \
    "$ROOT/build/lib/libyai_support.a" \
    "$ROOT/build/lib/libyai_platform.a" \
    -o "$BIN_DIR/$t" -lm $LMDB_LIBS $HIREDIS_LIBS $DUCKDB_LIBS
  "$BIN_DIR/$t"
done
