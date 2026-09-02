# H17 failure evidence

## H17-F01 — nonexistent target evidence was accepted

- run_id: `h17-prefx-evidence-20260902`
- pre-state: W17 result writer with no target-local evidence resolution
- cwd: `/home/mothx/computer-science/projects/YAI/yai/engine`
- exact command: `cargo test -p yai-engine wave17_same_tenant_handoff_moves_data_without_authority -- --nocapture`
- attack: the result used `transition:nonexistent-target-evidence`
- exit: 0 (defect reproduced: the forged ref committed)
- raw stdout:

```text
running 1 test
w17_handoff: handoff_id=handoff:fcb70527d2f5faeeae41f10b7af3e385 acceptance_contenders=8 acceptance_winners=1 source_facts=offer,reconciliation target_facts=acceptance,result source_grants=0 source_effects=0
test store::lmdb::tests::wave17_same_tenant_handoff_moves_data_without_authority ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 165 filtered out; finished in 0.01s
```

Correction: every ref is now resolved against the exact target Case
Transition/fact inside the result transaction. The identical ref returns
`handoff_result_evidence_not_target_local` before mutation; H17-E01 records the
post-fix matrix.

## H17-F02 — success could follow target cancellation

- run_id: `h17-prefx-terminal-result-20260902`
- pre-state: accepted Handoff; target Case then cancelled
- cwd: `/home/mothx/computer-science/projects/YAI/yai/engine`
- exact command: `cargo test -p yai-engine wave17_handoff_terminal_case_matrix_preserves_both_histories -- --nocapture`
- exit: 101
- raw stderr:

```text
running 1 test
test store::lmdb::tests::wave17_handoff_terminal_case_matrix_preserves_both_histories ... FAILED
thread 'store::lmdb::tests::wave17_handoff_terminal_case_matrix_preserves_both_histories' panicked at yai-engine/src/store/lmdb.rs:22734:9:
assertion `left == right` failed
  left: Succeeded
 right: Cancelled
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 165 filtered out
```

Correction: cancellation and result now serialize on target Case state;
cancel-first refuses `handoff_result_target_case_terminal`, result-first
remains historical truth. The updated identical test and H17 lifecycle matrix
pass.

## H17-F03 — deep executable Subflow used root-local lookup

- run_id: `h17-prefx-deep-subflow-20260902`
- pre-state: exact depth-4 Subflow, deepest node executable ModelWork
- cwd: `/home/mothx/computer-science/projects/YAI/yai/engine`
- exact command: `cargo test -p yai-engine hardening17_depth_four_modelwork_recovery_preserves_qualified_identity -- --nocapture`
- exit: 101
- raw stderr:

```text
running 1 test
thread 'store::lmdb::tests::hardening17_tests::hardening17_depth_four_modelwork_recovery_preserves_qualified_identity' panicked at yai-engine/src/store/tests/hardening17_tests.rs:1393:10:
called `Result::unwrap()` on an `Err` value: "workflow_node_not_found"
test store::lmdb::tests::hardening17_tests::hardening17_depth_four_modelwork_recovery_preserves_qualified_identity ... FAILED
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 177 filtered out
```

The deterministic reproduction failed similarly at proposal creation with
`workflow_node_not_found`. Correction: ReadyWork, execution identity,
RuntimeWorkflowContext and deterministic proposal use the exact qualified
effective node ID. Post-fix H17-E01 records one execution/result and one
proposal/Operation at depth four.

## H17-F04 — smoke descriptor typo

The first `make smoke-adaptive-workflow-hardening` ran 181 engine and 22 CLI
tests successfully, then exited 2 because the shell smoke named a test
`hardening17_exact_handoff_bounds...` while the executable test is
`hardening17_payload_role_evidence_and_json_bounds_are_exact`. This was test
integration only. The corrected executable script exits 0 in H17-E01.
