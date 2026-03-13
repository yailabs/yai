#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
"$ROOT/tests/legacy/unit/network/mesh/run_mesh_unit_tests.sh"
echo "mesh_smoke: ok"
