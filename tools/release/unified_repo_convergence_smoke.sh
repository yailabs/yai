#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

run() {
  echo "[b13-smoke] $*"
  "$@"
}

run make -C "$ROOT" governance-check
run python3 "$ROOT/tools/validate/validate_root_framing.py"
run python3 "$ROOT/tools/validate/validate_tooling_legacy_refs.py"
run python3 "$ROOT/tools/validate/validate_governance_manifests.py"
run python3 "$ROOT/tools/validate/validate_aux_naming.py"
run python3 "$ROOT/tools/validate/validate_final_naming_consistency.py"
run python3 "$ROOT/tools/validate/validate_docs_hierarchy.py"
run python3 "$ROOT/tools/validate/validate_docs_naming.py"
run python3 "$ROOT/tools/validate/validate_docs_canonical_spine.py"
run python3 "$ROOT/tools/validate/validate_docs_section_verticalization.py"
run python3 "$ROOT/tools/validate/validate_docs_archive_reduction.py"
run python3 "$ROOT/tools/validate/validate_docs_editorial_consistency.py"
run python3 "$ROOT/tools/validate/validate_docs_surface_hardening.py"
run python3 "$ROOT/tools/validate/validate_docs_live_set_compression.py"
run python3 "$ROOT/tools/validate/validate_docs_authority_contract.py"
run python3 "$ROOT/tools/validate/validate_program_decision_lifecycle.py"
run python3 "$ROOT/tools/validate/validate_docs_surface_policy.py"
run python3 "$ROOT/tools/validate/validate_docs_freeze_contract.py"
run python3 "$ROOT/tools/validate/validate_governance_contracts_schema.py"
run python3 "$ROOT/tools/validate/validate_governance_ingestion_pipeline.py"
run "$ROOT/tests/unit/governance/run_governance_unit_tests.sh"
run "$ROOT/tests/integration/governance/run_governance_resolution_smoke.sh"

echo "[b13-smoke] unified repo convergence: OK"
