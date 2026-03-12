#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

run() {
  echo "[final-matrix] $*"
  "$@"
}

run make -C "$ROOT" -j4
run make -C "$ROOT" governance-check
run tests/legacy/unit/governance/run_governance_unit_tests.sh
run tests/sys/governance/run_governance_resolution_smoke.sh
run "$ROOT"/tests/user/shell/integration/container_output_guardrail.sh

if [[ "${RUN_SOCKET_SCENARIOS:-0}" == "1" ]]; then
  run "$ROOT"/tests/legacy/workspace/workspace_session_binding_contract.sh
  run "$ROOT"/tests/legacy/workspace/workspace_inspect_surfaces.sh
  run "$ROOT"/tests/legacy/workspace/workspace_real_flow.sh
  run "$ROOT"/tests/legacy/workspace/workspace_scientific_flow.sh
  run "$ROOT"/tests/legacy/workspace/workspace_digital_flow.sh
  run "$ROOT"/tests/legacy/workspace/workspace_hostile_path_baseline.sh
  run "$ROOT"/tests/legacy/workspace/workspace_isolation_guards.sh
  run "$ROOT"/tests/legacy/workspace/workspace_negative_paths.sh
else
  echo "[final-matrix] socket scenarios skipped (set RUN_SOCKET_SCENARIOS=1 to enable)"
fi

echo "[final-matrix] ok"
