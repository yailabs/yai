#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
"$ROOT/tests/unit/edge/run_edge_unit_tests.sh"
echo "edge_smoke: ok"
