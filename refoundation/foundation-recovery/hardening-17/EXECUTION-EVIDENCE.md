# H17 execution evidence

## H17-E01 — focused adaptive hardening

- evidence_id: `P-H17-01..15`
- run_id: `h17-focused-20260902-01`
- order: 1
- pre-state: H17 source candidate; baseline `e58b08996649a30ebe3446afc4ebbfe4ef2aadfd`; historical dirty work preserved
- cwd: `/home/mothx/computer-science/projects/YAI/yai`
- environment: provider variables absent; temporary LMDB stores created by tests
- Principal/Tenant/Cases: deterministic per-test H17 fixtures
- exact command: `tests/characterization/adaptive-workflow-hardening/test_adaptive_workflow_hardening.sh`
- exit: 0
- bounded raw stdout:

```text
h17_patch_bounds: bytes=262144 accepted bytes=262145 rejected operations=32 accepted operations=33 rejected added_nodes=16 accepted added_nodes=17 rejected added_edges_cap=64 dominated_by_operations_cap=32 future_schema=v99_rejected path_separator=in_definition_node_rejected
h17_amendment_lineage: revisions=32 bytes=29097 transitions=66 effective_nodes=33 effective_edges=0 lineage_execution_us=2868256 prefix_replay_us=227916 topology_rebuild_us=17827 resolution_us=20506 corruptions=11 fail_closed=11 digest=sha256:78413b6bb75dea1bdf7dd1a86b9d9bea4274e480b07a43ea5e24a11c83f103f3
h17_amendment_process_race: processes=32 winners=1 stale=31 generation_delta=1 revision=1 digest=sha256:6385e6aab98c5a5887225d406f8471f6828ccdbce52c1b3abf11c80eb45d44a5
h17_handoff_forgery: wrong_handoff=rejected cross_case_evidence=rejected empty_evidence_success=admitted_as_target_report source_effects=0 source_grants=0 tampered_outcome=integrity_rejected
h17_handoff_process_races: result_processes=32 result_semantic_winners=1 identical_observers=16 conflicts=16 result_generation_delta=1 reconcile_processes=32 reconcile_semantic_winners=1 idempotent_observers=32 reconcile_generation_delta=1
h17_handoff_lifecycle_races: cancel_accept_serialized=true acceptance_committed=1 cancel_result_serialized=true result_committed=1 cancel_reconcile_both_preserved=true close_before_settlement=blocked close_after_reconcile=committed
h17_cycle_races: two_way_writers=2 committed=1 refused=1 three_way_writers=3 committed=2 refused=1 final_active_graph=acyclic
h17_active_graph_terminality: original_edge=A->B target_decline=terminal reverse_edge=B->A admitted_before_source_reconcile=true active_cycle=false
h17_multi_case_scale: cases=64 active_handoffs=63 offer_validation_us=87467 cycle_check_us=1421 graph_relations=254 graph_double_rebuild_us=35307 duplicates=0 process_owners=0
h17_cross_definition: depth=4 qualified_leaf=root/s1/s2/s3/s4/leaf corrupt_deep_child=fail_closed exact_restore=resolution_equal topology_digest=sha256:8f9ad181b4b3d785c5808d1654178f37bd6c01eca5ab045710aaecb0f7c1c8d9 simulated_upgrade_drift=fail_closed
h17_deep_model_recovery: depth=4 node=root/s1/s2/s3/s4/deep-model executions=1 provider_invocations=1 provider_results=1 completed_turn_duplicates=0 replay_equal=true
h17_deep_deterministic_recovery: depth=4 node=root/s1/s2/s3/s4/deep-deterministic proposals=1 operations=1 proposal_duplicates=0 operation_duplicates=0 operation_id=operation:f487c8c2ba660b46da789651cfc2650d
h17_handoff_bounds: request_bytes=16384 accepted request_bytes=16385 rejected result_bytes=16384 accepted evidence_refs=32 accepted evidence_refs=33 rejected role_requirements=16 accepted role_requirements=17 rejected duplicate_json_keys=rejected
w17_model_patch: provider_results=3 malformed_candidates=0 forged_origin_candidates=0 valid_candidates=1 duplicate_candidates=0 auto_adoptions=0 owner_adoptions=1 patch_id=workflow-plan-patch:c193714b8936ab30234315cac0607d74
adaptive_workflow_hardening_characterization: pass
semantic_owner_delta: 0
```

- invariant: amendment lineage, qualified Subflow identity, Handoff terminal
  facts and active multi-Case graph are deterministic, one-winner and
  reconstructible without a new owner.

## H17-E02 — engine and CLI suite

- evidence_id: `H17-QUAL-ENGINE-CLI`
- run_id: `h17-build-rust-20260902-01`
- order: 2
- pre-state: same formatted candidate
- exact command: `make smoke-adaptive-workflow-hardening` (its `build-rust`
  prerequisite runs both workspaces before the focused script)
- exit: initial target exit 2 only because the new smoke script referenced a
  misspelled test name after both suites passed; corrected script rerun is
  H17-E01. The suite results from the same run were:

```text
test result: ok. 181 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 34.11s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
```

- invariant: complete engine and CLI unit suites were green; the target
  integration typo caused no product-semantic failure.

Additional full qualification evidence is appended after the final candidate
run; outputs from distinct commands are not merged into either run above.

## H17-E03 — full repository check

- evidence_id: `H17-QUAL-CHECK`
- run_id: `h17-make-check-20260902-01`
- order: 3
- pre-state: complete formatted H17 candidate and dossier
- cwd: `/home/mothx/computer-science/projects/YAI/yai`
- environment: normal repository environment; external provider variables absent
- exact command: `make check`
- exit: 0
- bounded raw stdout:

```text
check-no-old-roots: ok
check-required-layout: ok
check-source-placement: ok
check-source-surface-clean: ok
check-doc-links: ok (29 files)
test result: ok. 181 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 33.83s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
case_runtime:agentless_26_turn_provider_model_replacement ok
human_review:crash_r1_r6_recovery ok
governance_hardening_characterization: pass
policy_authority:allow_chain ok
```

- invariant: source/layout/docs, complete engine/CLI suites, runtime endurance,
  review recovery, governance and lower-wave authority smoke remain green.

## H17-E04 — full characterization

- evidence_id: `H17-QUAL-CHARACTERIZATION`
- run_id: `h17-characterization-20260902-02`
- order: 4
- pre-state: identical H17 candidate; first restricted run had already proved
  181 engine and 22 CLI tests but the sandbox denied AF_UNIX bind
- cwd: `/home/mothx/computer-science/projects/YAI/yai`
- environment: same repository environment, run with local Unix-socket
  permission; no external provider variables
- exact command: `make characterization`
- exit: 0
- bounded raw stdout:

```text
test result: ok. 181 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 35.17s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
daemon:started
ipc:status ok
daemon:shutdown ok
provider_model_vertical:real_http_invocation ok
case_runtime:agentless_26_turn_provider_model_replacement ok
cli_product_surface: porcelain_governed_workflow=pass
w17_scale: amendments=32 amendment_bytes=28028 amendment_nodes=33 amendment_rebuild_us=18132 subflow_instances=128 expanded_nodes=512 expanded_edges=765 expanded_rebuild_us=41872
adaptive_workflow_characterization: pass
h17_amendment_lineage: revisions=32 bytes=29097 transitions=66 effective_nodes=33 effective_edges=0 lineage_execution_us=2918397 prefix_replay_us=233055 topology_rebuild_us=18497 resolution_us=20702 corruptions=11 fail_closed=11 digest=sha256:78413b6bb75dea1bdf7dd1a86b9d9bea4274e480b07a43ea5e24a11c83f103f3
h17_multi_case_scale: cases=64 active_handoffs=63 offer_validation_us=84373 cycle_check_us=1368 graph_relations=254 graph_double_rebuild_us=34736 duplicates=0 process_owners=0
adaptive_workflow_hardening_characterization: pass
semantic_owner_delta: 0
```

- invariant: all characterization through W17 plus H17, including local IPC,
  provider fixture, 512-node expansion and 64-Case derived graph, passes.

The preceding restricted run exited 2 only at `smoke-new11` with raw stderr
`failed to start ipc server: invalid`; it is classified as a qualification
environment limitation and caused no source change.

## H17-E05 — registry, format, lint and hygiene

- evidence_id: `H17-QUAL-HYGIENE`
- run_id: `h17-hygiene-20260902-01`
- order: 5
- pre-state: same fully characterized candidate
- exact commands: engine and CLI `cargo fmt --check`; engine and CLI
  `cargo clippy --all-targets` under the repository warning contract;
  `python3 tests/characterization/cli-product-surface/audit_registry.py
  --binary ./yai`; `git diff --check`; `make check-docs`
- exit: 0 for every command
- bounded raw stdout:

```text
{"handler_failures": 0, "help_failures": 0, "operation_count": 134, "registry_digest": "sha256:c219161abca72268008b9326d4e43a050d8001cde95b0cbe8d6f35c32ebf85a1", "schema": "yai.cli.command_discovery.v1", "visibility_counts": {"advanced": 9, "compatibility": 16, "plumbing": 45, "product": 63, "removed": 1}}
check-doc-links: ok (29 files)
check-repository-identity: ok
```

- stderr: Clippy reported only the repository's pre-existing admitted warning
  classes (`too_many_arguments`, `needless_borrow`, `should_implement_trait`,
  `ptr_arg`) and exited 0; format and diff checks emitted no output.
- invariant: W16 has one registry with zero handler/help failures; H17 adds no
  parser bypass, formatting error or whitespace defect.

## H17-E06 — final transactional race and engine closure

- evidence_id: `H17-01,H17-02,H17-09,H17-10,H17-45`
- run_id: `h17-final-races-20260902-01`
- order: 6
- pre-state: final H17 source candidate after adding adjacent-revision and
  passive/HumanInput adoption races; baseline and historical dirty state
  unchanged
- cwd: `/home/mothx/computer-science/projects/YAI/yai`
- environment: local independent LMDB writers/processes; external provider
  variables absent
- exact commands:
  `tests/characterization/adaptive-workflow-hardening/test_adaptive_workflow_hardening.sh`;
  `cargo test --manifest-path engine/Cargo.toml --workspace`
- exit: 0 for both commands
- bounded raw stdout:

```text
h17_adjacent_revision_races: rounds=8 winning_amendments=8 late_stale_writers=8 forks=0 final_revision=8
h17_progression_adoption_races: human_input_or_amendment_one_winner=true passive_satisfaction_or_amendment_one_truth=true write_skew=0
h17_amendment_process_race: processes=32 winners=1 stale=31 generation_delta=1 revision=1 digest=sha256:6385e6aab98c5a5887225d406f8471f6828ccdbce52c1b3abf11c80eb45d44a5
h17_handoff_process_races: result_processes=32 result_semantic_winners=1 identical_observers=16 conflicts=16 result_generation_delta=1 reconcile_processes=32 reconcile_semantic_winners=1 idempotent_observers=32 reconcile_generation_delta=1
h17_cycle_races: two_way_writers=2 committed=1 refused=1 three_way_writers=3 committed=2 refused=1 final_active_graph=acyclic
adaptive_workflow_hardening_characterization: pass
test result: ok. 183 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 34.26s
```

- invariant: a stale adjacent writer never forks amendment lineage; HumanInput
  and passive canonical progression serialize against adoption; the final
  engine suite including all H17 process contenders is green.
