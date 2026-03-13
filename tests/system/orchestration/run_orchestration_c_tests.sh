#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
BIN_DIR="$ROOT/build/test/sys-orchestration/bin"
LMDB_LIBS="$(pkg-config --libs liblmdb 2>/dev/null || true)"
HIREDIS_LIBS="$(pkg-config --libs hiredis 2>/dev/null || true)"
DUCKDB_LIBS=""
if [[ -f /opt/homebrew/opt/duckdb/lib/libduckdb.dylib ]]; then
  DUCKDB_LIBS="-L/opt/homebrew/opt/duckdb/lib -lduckdb"
fi
mkdir -p "$BIN_DIR"
make -C "$ROOT" data graph orchestration >/dev/null

for t in test_runtime_primary test_orchestration_transport_smoke; do
  cc -Wall -Wextra -std=c11 -O2 \
    -I"$ROOT/include" \
    "$ROOT/tests/sys/orchestration/$t.c" \
    "$ROOT/build/lib/libyai_core.a" \
    "$ROOT/build/lib/libyai_orchestration.a" \
    "$ROOT/build/lib/libyai_knowledge.a" \
    "$ROOT/build/lib/libyai_providers.a" \
    "$ROOT/build/lib/libyai_data.a" \
    "$ROOT/build/lib/libyai_graph.a" \
    "$ROOT/build/lib/libyai_network.a" \
    "$ROOT/build/lib/libyai_daemon.a" \
    "$ROOT/build/lib/libyai_container.a" \
    "$ROOT/build/lib/libyai_support.a" \
    "$ROOT/build/lib/libyai_platform.a" \
    "$ROOT/build/lib/libyai_protocol.a" \
    -o "$BIN_DIR/$t" -lm $LMDB_LIBS $HIREDIS_LIBS $DUCKDB_LIBS
  "$BIN_DIR/$t"
done
