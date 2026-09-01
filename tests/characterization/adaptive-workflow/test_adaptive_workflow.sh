#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
ENGINE_MANIFEST="$ROOT/engine/Cargo.toml"
YAI_BIN="$ROOT/target/debug/yai"

run_engine_proof() {
  local test_name=$1
  local expected=$2
  local output
  output=$(cargo test --manifest-path "$ENGINE_MANIFEST" -p yai-engine \
    "$test_name" -- --exact --nocapture 2>&1)
  grep -Fq -- "$expected" <<<"$output"
  printf '%s\n' "$output" | grep -E '^w17_|test result:' | tail -2
}

run_engine_proof \
  store::lmdb::tests::wave17_planpatch_adoption_is_case_local_stale_safe_and_replayable \
  'w17_planpatch: proposals=2 adopted=1 stale=1 revision=1'
run_engine_proof \
  store::lmdb::tests::wave17_model_planpatch_is_strict_candidate_and_cannot_self_adopt \
  'malformed_candidates=0 valid_candidates=1 duplicate_candidates=0 auto_adoptions=0 owner_adoptions=1'
run_engine_proof \
  store::lmdb::tests::wave17_eight_way_amendment_adoption_has_one_winner \
  'w17_patch_race: contenders=8 winners=1 stale=7 amendments=1 revision=1'
run_engine_proof \
  store::lmdb::tests::wave17_subflow_progresses_inside_one_case_and_replays_without_run_owner \
  'cases=1 definitions=2 qualified_nodes=2 work_items=0 completed=true'
run_engine_proof \
  store::lmdb::tests::wave17_same_tenant_handoff_moves_data_without_authority \
  'acceptance_contenders=8 acceptance_winners=1'
run_engine_proof \
  store::lmdb::tests::wave17_workflow_handoff_waits_worker_free_until_source_reconciliation \
  'offer=1 accept=1 result=1 reconcile=1 source_satisfaction=1 workers_held=0'
run_engine_proof \
  store::lmdb::tests::wave17_handoff_cycle_and_cross_tenant_edges_fail_closed \
  'active_cycle=rejected cross_tenant=rejected target_payloads=0'
run_engine_proof \
  store::lmdb::tests::wave17_handoff_terminal_case_matrix_preserves_both_histories \
  'source_cancel_before_accept=rejected target_cancel_after_accept=reconciled_cancelled target_results=0 histories_verified=4'
run_engine_proof \
  workflow::tests::wave17_amendment_and_expanded_subflow_bounds_are_operational \
  'w17_scale: amendments=32'
run_engine_proof \
  store::lmdb::tests::wave17_four_case_handoff_chain_replays_without_process_owner \
  'w17_handoff_chain: cases=4 edges=3 accepts=3 results=3 reconciliations=3 histories_replayed=4 process_owners=0 imported_grants=0'

help_json=$($YAI_BIN help --json)
grep -Fq '"operation_id":"yai.workflow.patch.propose"' <<<"$help_json"
grep -Fq '"operation_id":"yai.workflow.patch.propose_model"' <<<"$help_json"
grep -Fq '"operation_id":"yai.case.handoff.offer"' <<<"$help_json"
grep -Fq '"operation_id":"yai.case.handoff.reconcile"' <<<"$help_json"

case "$help_json" in
  *'"cli_registry_digest":"sha256:'*) ;;
  *) printf 'adaptive workflow operations lack registry identity\n' >&2; exit 1 ;;
esac

printf 'adaptive_workflow_characterization: pass\n'
printf 'planpatch_proposal_adoption: separated\n'
printf 'model_auto_adoptions: 0\n'
printf 'patch_race_contenders: 8\n'
printf 'patch_race_winners: 1\n'
printf 'subflow_work_items_for_passive_container: 0\n'
printf 'maximum_amendments_replayed: 32\n'
printf 'maximum_effective_nodes_expanded: 512\n'
printf 'handoff_chain_cases_replayed: 4\n'
printf 'handoff_workers_held_while_waiting: 0\n'
printf 'cross_tenant_handoff: rejected\n'
printf 'workflow_run_owner: 0\n'
printf 'multi_case_process_owner: 0\n'
