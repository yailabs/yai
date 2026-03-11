#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
"$ROOT/tests/unit/runtime/run_runtime_unit_tests.sh"
echo "runtime_state_smoke: ok"
