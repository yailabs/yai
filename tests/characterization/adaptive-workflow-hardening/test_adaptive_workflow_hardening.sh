#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
ENGINE_MANIFEST="$ROOT/engine/Cargo.toml"

run_engine_proof() {
  local test_name=$1
  local expected=$2
  local output
  output=$(cargo test --manifest-path "$ENGINE_MANIFEST" -p yai-engine \
    "$test_name" -- --exact --nocapture 2>&1)
  grep -Fq -- "$expected" <<<"$output"
  printf '%s\n' "$output" | grep -E '^h17_|^w17_|test result:' | tail -2
}

run_engine_proof \
  store::lmdb::tests::hardening17_tests::hardening17_planpatch_limits_and_future_schema_refusal_have_no_off_by_one \
  'bytes=262144 accepted bytes=262145 rejected operations=32 accepted operations=33 rejected added_nodes=16 accepted added_nodes=17 rejected'
run_engine_proof \
  store::lmdb::tests::hardening17_tests::hardening17_long_amendment_lineage_replays_and_corruption_fails_closed \
  'h17_amendment_lineage: revisions=32'
run_engine_proof \
  store::lmdb::tests::hardening17_tests::hardening17_thirty_two_process_amendment_race_has_one_winner \
  'processes=32 winners=1 stale=31 generation_delta=1 revision=1'
run_engine_proof \
  store::lmdb::tests::hardening17_tests::hardening17_adjacent_revision_late_writers_never_fork_lineage \
  'rounds=8 winning_amendments=8 late_stale_writers=8 forks=0 final_revision=8'
run_engine_proof \
  store::lmdb::tests::hardening17_tests::hardening17_amendment_races_with_human_and_passive_progress_are_serializable \
  'human_input_or_amendment_one_winner=true passive_satisfaction_or_amendment_one_truth=true write_skew=0'
run_engine_proof \
  store::lmdb::tests::hardening17_tests::hardening17_handoff_forgery_and_empty_success_boundaries_are_exact \
  'wrong_handoff=rejected cross_case_evidence=rejected empty_evidence_success=admitted_as_target_report'
run_engine_proof \
  store::lmdb::tests::hardening17_tests::hardening17_result_and_reconciliation_process_races_have_one_truth \
  'result_processes=32 result_semantic_winners=1 identical_observers=16 conflicts=16'
run_engine_proof \
  store::lmdb::tests::hardening17_tests::hardening17_cancellation_close_and_handoff_writes_are_serializable \
  'cancel_accept_serialized=true'
run_engine_proof \
  store::lmdb::tests::hardening17_tests::hardening17_concurrent_cycle_creation_never_commits_a_cycle \
  'two_way_writers=2 committed=1 refused=1 three_way_writers=3 committed=2 refused=1 final_active_graph=acyclic'
run_engine_proof \
  store::lmdb::tests::hardening17_tests::hardening17_terminal_target_disposition_leaves_active_wait_graph \
  'target_decline=terminal reverse_edge=B->A admitted_before_source_reconcile=true active_cycle=false'
run_engine_proof \
  store::lmdb::tests::hardening17_tests::hardening17_large_handoff_graph_and_derived_relations_rebuild_exactly \
  'cases=64 active_handoffs=63'
run_engine_proof \
  store::lmdb::tests::hardening17_tests::hardening17_transitive_child_corruption_fails_closed_and_exact_restore_recovers \
  'corrupt_deep_child=fail_closed exact_restore=resolution_equal'
run_engine_proof \
  store::lmdb::tests::hardening17_tests::hardening17_depth_four_modelwork_recovery_preserves_qualified_identity \
  'depth=4 node=root/s1/s2/s3/s4/deep-model executions=1 provider_invocations=1 provider_results=1 completed_turn_duplicates=0 replay_equal=true'
run_engine_proof \
  store::lmdb::tests::hardening17_tests::hardening17_depth_four_deterministic_recovery_does_not_duplicate_proposal_or_operation \
  'depth=4 node=root/s1/s2/s3/s4/deep-deterministic proposals=1 operations=1 proposal_duplicates=0 operation_duplicates=0'
run_engine_proof \
  handoff::tests::hardening17_payload_role_evidence_and_json_bounds_are_exact \
  'request_bytes=16384 accepted request_bytes=16385 rejected'
run_engine_proof \
  store::lmdb::tests::wave17_model_planpatch_is_strict_candidate_and_cannot_self_adopt \
  'forged_origin_candidates=0 valid_candidates=1 duplicate_candidates=0 auto_adoptions=0 owner_adoptions=1'

printf 'adaptive_workflow_hardening_characterization: pass\n'
printf 'amendment_process_contenders: 32\n'
printf 'handoff_result_process_contenders: 32\n'
printf 'handoff_reconcile_process_contenders: 32\n'
printf 'maximum_amendments_replayed: 32\n'
printf 'maximum_subflow_depth_recovered: 4\n'
printf 'multi_case_graph_cases: 64\n'
printf 'semantic_owner_delta: 0\n'
