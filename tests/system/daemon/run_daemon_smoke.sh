#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
"$ROOT/tests/legacy/unit/daemon/run_daemon_unit_tests.sh"
echo "daemon_smoke: ok"
