#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
OUT_DIR="$ROOT/build/test/unit_mesh"
mkdir -p "$OUT_DIR"

cc -Wall -Wextra -std=c11 -O2 \
  -I"$ROOT/include" -I"$ROOT/include/yai" \
  "$ROOT/tests/unit/mesh/test_mesh_minimal.c" \
  "$ROOT/lib/network/identity/identity.c" \
  "$ROOT/lib/network/discovery/peer_registry.c" \
  "$ROOT/lib/network/discovery/membership.c" \
  "$ROOT/lib/network/discovery/discovery.c" \
  "$ROOT/lib/network/topologies/sovereign_overlay/awareness.c" \
  "$ROOT/lib/network/topologies/sovereign_overlay/coordination.c" \
  "$ROOT/lib/network/transport/session.c" \
  "$ROOT/lib/network/transport/replay.c" \
  "$ROOT/lib/network/topologies/sovereign_overlay/conflict.c" \
  "$ROOT/lib/network/overlay/containment.c" \
  "$ROOT/lib/network/identity/enrollment.c" \
  -o "$OUT_DIR/mesh_unit_tests"

"$OUT_DIR/mesh_unit_tests"
echo "mesh_unit_tests: ok"
