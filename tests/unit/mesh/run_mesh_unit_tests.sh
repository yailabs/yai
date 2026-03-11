#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
OUT_DIR="$ROOT/build/test/unit_mesh"
mkdir -p "$OUT_DIR"

cc -Wall -Wextra -std=c11 -O2 \
  -I"$ROOT/include" -I"$ROOT/include/yai" \
  "$ROOT/tests/unit/mesh/test_mesh_minimal.c" \
  "$ROOT/lib/mesh/identity/identity.c" \
  "$ROOT/lib/mesh/peer_registry/peer_registry.c" \
  "$ROOT/lib/mesh/membership/membership.c" \
  "$ROOT/lib/mesh/discovery/discovery.c" \
  "$ROOT/lib/mesh/awareness/awareness.c" \
  "$ROOT/lib/mesh/coordination/coordination.c" \
  "$ROOT/lib/mesh/transport/transport_state.c" \
  "$ROOT/lib/mesh/replay/replay_state.c" \
  "$ROOT/lib/mesh/conflict/conflict_state.c" \
  "$ROOT/lib/mesh/containment/containment_state.c" \
  "$ROOT/lib/mesh/enrollment/enrollment_state.c" \
  -o "$OUT_DIR/mesh_unit_tests"

"$OUT_DIR/mesh_unit_tests"
echo "mesh_unit_tests: ok"
