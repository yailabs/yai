#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
OUT_DIR="$ROOT/build/test/unit_mesh"
mkdir -p "$OUT_DIR"

cc -Wall -Wextra -std=c11 -O2 \
  -I"$ROOT/include" -I"$ROOT/include/yai" \
  "$ROOT/tests/unit/network/mesh/test_mesh_minimal.c" \
  "$ROOT/lib/network/authority/identity.c" \
  "$ROOT/lib/network/topology/peer_registry.c" \
  "$ROOT/lib/network/topology/membership.c" \
  "$ROOT/lib/network/discovery/discovery.c" \
  "$ROOT/lib/network/topologies/mesh/mesh_topology.c" \
  "$ROOT/lib/network/topologies/mesh/mesh_peering.c" \
  "$ROOT/lib/network/routing/coordination.c" \
  "$ROOT/lib/network/transport/transport_client.c" \
  "$ROOT/lib/network/routing/replay_state.c" \
  "$ROOT/lib/network/routing/conflict_state.c" \
  "$ROOT/lib/network/authority/containment_state.c" \
  "$ROOT/lib/network/discovery/enrollment.c" \
  -o "$OUT_DIR/mesh_unit_tests"

"$OUT_DIR/mesh_unit_tests"
echo "mesh_unit_tests: ok"
