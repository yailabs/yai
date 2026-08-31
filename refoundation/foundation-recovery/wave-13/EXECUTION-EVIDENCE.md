# Wave 13 execution evidence

All timestamps, identifiers and output below were emitted by the real product
or test process. Output is intentionally bounded; the complete scenario and
every asserted subcommand are in
`tests/characterization/multi-case-runtime/test_multi_case_runtime.sh`.

## E13-01 — authenticated RuntimeInstance bootstrap

- evidence_id: E13-01
- run_id: `/tmp/yai-multi-case-runtime.NKSHNe`
- execution_order: 1
- pre-state: empty temporary `YAI_HOME`; no RuntimeInstance
- cwd: repository root
- environment: `YAI_EXECUTION_EVIDENCE=1`; local AF_UNIX daemon and loopback providers
- authenticated principal: `principal:72cc156b82060120eac8f7e234dbfcef`, POSIX real/effective UID 1000
- Tenants: `tenant:w13-a`, `tenant:w13-b`
- runtime_instance_id: `runtime-instance:local-default`
- exact command: `env YAI_EXECUTION_EVIDENCE=1 tests/characterization/multi-case-runtime/test_multi_case_runtime.sh`
- actual exit: 0
- invariant: one kernel-authenticated Principal owns both test Tenants; no caller `--as` creates scheduler authority.

Bounded raw output:

```text
[product-command:bootstrap-a]
$ .../target/debug/yai security bootstrap-local --tenant tenant:w13-a --organization organization:characterization
security_bootstrap: created
authentication_kind: local_posix_effective_credential
real_uid: 1000
effective_uid: 1000
principal_id: principal:72cc156b82060120eac8f7e234dbfcef
tenant_id: tenant:w13-a
membership: owner
exit: 0

[product-command:bootstrap-b]
$ .../target/debug/yai security bootstrap-local --tenant tenant:w13-b --organization organization:characterization
security_bootstrap: created
principal_id: principal:72cc156b82060120eac8f7e234dbfcef
tenant_id: tenant:w13-b
membership: owner
exit: 0
```

## E13-02 — real bounded concurrency, same-Case exclusion and fairness

- evidence_id: E13-02
- run_id: `/tmp/yai-multi-case-runtime.NKSHNe`
- execution_order: 2
- pre-state: three Ready+Valid Cases; disjoint roots; no queued work
- cwd/environment/authentication/Tenants/instance: same as E13-01
- exact service command: `yai runtime serve --workers 2 --max-active-per-tenant 1 --max-queued-per-tenant 2 --max-queued-total 3`
- exact submissions: request IDs `request:a1-first`, `request:b1-first`, `request:a2-first`, `request:a1-second`, `request:b1-second`
- actual exits: 0
- produced IDs: `runtime-work:4f46edeb97a1452e`, `runtime-work:97339c40e799b5ae`, `runtime-work:0c6f83bb7c9bb52f`, `runtime-work:f739dbb52ed93bbc`, `runtime-work:26d696ac1200653c`
- invariant: two disjoint Cases overlap; one Case never owns two workers; Tenant active count is bounded; B1 dispatches before A's backlog drains; FIFO holds inside each Tenant.

Bounded raw output from the unchanged scenario rerun:

```text
tenant_a.started_at_unix_ms: 1788174021017
tenant_a.completed_at_unix_ms: 1788174022517
tenant_b.started_at_unix_ms: 1788174021017
tenant_b.completed_at_unix_ms: 1788174022517
dispatch_order: runtime-work:4f46edeb97a1452e runtime-work:97339c40e799b5ae runtime-work:0c6f83bb7c9bb52f runtime-work:26d696ac1200653c runtime-work:f739dbb52ed93bbc
same_case_max_active: 1
tenant_active_limit_observed: 1
workers_max_observed: 2
```

The intervals overlap because both starts precede both completions; both actual
provider calls began in the same millisecond and each remained live for 1500ms.

## E13-03 — split-brain, isolation and backpressure negatives

- evidence_id: E13-03
- run_id: `/tmp/yai-multi-case-runtime.NKSHNe`
- execution_order: 3
- pre-state: first instance Running; queue bounds active
- exact commands:
  - `yai runtime serve --workers 1 --max-active-per-tenant 1 --max-queued-per-tenant 1 --max-queued-total 1`
  - `yai runtime submit --tenant tenant:w13-b --case case:w13-a1 ... --idempotency-key request:cross`
  - bounded overflow submissions from the characterization script
- actual exits: 2 for each negative
- invariant: a second live scheduler cannot dispatch, Tenant/Case mismatch cannot enter the queue, and capacity rejection occurs before Case mutation.

Bounded raw output:

```text
split_brain_exit: 2
cross_tenant_rejection_count: 1
tenant_queue_rejection_count: 1
global_queue_rejection_count: 1

runtime_instance_active: runtime-instance:local-default ...
runtime_work_security_domain_mismatch
runtime_tenant_queue_capacity_exhausted
runtime_global_queue_capacity_exhausted
```

The engine regression compares Case generation before/after rejected submission
and observes no Transition, provider invocation, Operation, Decision or Grant.

## E13-04 — conservative overlapping-root serialization

- evidence_id: E13-04
- run_id: `/tmp/yai-multi-case-runtime.NKSHNe`
- execution_order: 4
- pre-state: two same-Tenant Cases with parent/child canonical roots; two workers; Tenant active limit 2
- exact service command: `yai runtime serve --workers 2 --max-active-per-tenant 2 --max-queued-per-tenant 4 --max-queued-total 8`
- actual exit: 0
- invariant: only one overlapping Case dispatches at a time; this is explicitly scheduler serialization, not fencing.

Bounded raw output:

```text
runtime_dispatch: ... resource_relation=no_active_conflict
runtime_scheduler_skip: ... serialized_due_to_resource_overlap_or_unknown_relation
```

## E13-05 — Review parks and resumes exact work

- evidence_id: E13-05
- run_id: `/tmp/yai-multi-case-runtime.NKSHNe`
- execution_order: 5
- pre-state: review-required Case plus unrelated Tenant-B Case
- exact human command: `yai review approve <review_id> --case case:w13-review --reason 'authenticated scheduler review qualification'`
- actual exit: 0
- invariant: review WorkItem reaches `WaitingReview` and releases its worker; peer completes; authenticated approval resumes the same Operation/WorkItem without repeating its initial provider call.

Observed state sequence:

```text
runtime_work: Queued -> Running -> WaitingReview
peer_runtime_work: Queued -> Running -> Completed
review_approve: committed
runtime_work: WaitingReview -> Running -> Completed
```

## E13-06 — crash after PREPARE and durable recovery sweep

- evidence_id: E13-06
- run_id: `/tmp/yai-multi-case-runtime.NKSHNe`
- execution_order: 6
- pre-state: `case:w13-recovery` Ready+Valid; WorkItem queued with `after_prepare_before_effect`
- authenticated principal/Tenant/instance: `principal:72cc156b82060120eac8f7e234dbfcef`, `tenant:w13-b`, `runtime-instance:local-default`
- exact submit command: `yai runtime submit --tenant tenant:w13-b --case case:w13-recovery ... --idempotency-key request:recovery --failpoint after_prepare_before_effect`
- exact serve command: `yai runtime serve --workers 2 --max-active-per-tenant 2 --max-queued-per-tenant 4 --max-queued-total 8`
- first instance exit: 85
- restarted instance exit: 0
- produced IDs: WorkItem `runtime-work:2ccaf1494370823f`; Operation `operation:07755c0e7fed5951cd642c24a03d869d`; DecisionBasis `decision-basis:5fdd3666c337b8d4816eea3cbef342d4`; Decision `decision:665c42af53d442ebd714c6658503ee5e`; Grant/effect `43aa7252c6a9fd83b8521f5aaba20939`
- invariant: PREPARE remains canonical truth across total RuntimeInstance death; restart reclaims one stale item, reconciles the same effect and performs one physical write.

Bounded raw output before crash:

```text
instance_admission: reclaimed_stale
runtime_dispatch: work_id=runtime-work:2ccaf1494370823f worker_id=worker:0
runtime_worker_event: started timestamp_unix_ms=1788174074602
operation_id: operation:07755c0e7fed5951cd642c24a03d869d
decision_id: decision:665c42af53d442ebd714c6658503ee5e
decision_basis_id: decision-basis:5fdd3666c337b8d4816eea3cbef342d4
execution_grant_id: grant:43aa7252c6a9fd83b8521f5aaba20939
effect_id: effect:43aa7252c6a9fd83b8521f5aaba20939
effect_state: prepared_durable_before_mutation
controlled_effect_crash_injected: after_prepare_before_effect
exit: 85
```

Bounded raw output after restart:

```text
instance_admission: reclaimed_stale
recovered_items: 1
runtime_dispatch: work_id=runtime-work:2ccaf1494370823f worker_id=worker:0
runtime_admission: reclaimed_stale
reconciliation: EffectObserved
effect_id: effect:43aa7252c6a9fd83b8521f5aaba20939
effect_state: Some(Finalized)
operation_id: operation:07755c0e7fed5951cd642c24a03d869d
decision_id: decision:665c42af53d442ebd714c6658503ee5e
execution_grant_id: grant:43aa7252c6a9fd83b8521f5aaba20939
runtime_worker_event: stopped timestamp_unix_ms=1788174074751 ... status=completed
state: stopped
exit: 0
```

## E13-07 — drain/stop/restart

- evidence_id: E13-07
- run_id: `/tmp/yai-multi-case-runtime.NKSHNe`
- execution_order: 7
- pre-state: service Running after each bounded phase
- exact command: `yai runtime stop`
- actual exit: 0
- invariant: stop first commits Draining; the foreground instance dispatches no new work, joins active workers at their safe Case stop, then persists Stopped; a later start reclaims the stopped instance.

```text
runtime_stop: drain_requested
runtime_instance_id: runtime-instance:local-default
state: Draining
exit: 0
...
runtime_instance_id: runtime-instance:local-default
state: stopped
```

## E13-F01 — Case-global Transition identifier collision (fixed)

- evidence_id: E13-F01
- run_id: first multi-Case implementation reproduction
- execution_order: failure discovery 1
- exact command: `tests/characterization/multi-case-runtime/test_multi_case_runtime.sh`
- actual exit: nonzero
- invariant exposed: IDs that were safe only under a single Case collided at equal local generations.

```text
duplicate_transition_id: transition:provider-attachment:subject-llm-provider:30
```

Fix: new provider/resource attachment, prompt/result and interpretation IDs are
Case-qualified. Historical identifiers remain readable. The unchanged full
scenario now exits 0.

## E13-F02 — independent LMDB environments under worker concurrency (fixed)

- evidence_id: E13-F02
- run_id: first concurrent worker reproduction
- execution_order: failure discovery 2
- exact command: `tests/characterization/multi-case-runtime/test_multi_case_runtime.sh`
- actual exit: nonzero (one later reproduction terminated with signal 11)
- invariant exposed: independently opening the same LMDB environment in worker threads is not a safe local store boundary.

```text
failed to start RuntimeInstance read: MDB_BAD_RSLOT: Invalid reuse of reader locktable slot
```

Fix: all stores for the same canonical path in one process share one
`Arc<Environment>` from a weak path cache and use LMDB `NO_TLS`. The dedicated
four-worker repeated-open regression and the unchanged product scenario pass.

## E13-F03 — fixture impurity (fixed)

- evidence_id: E13-F03
- run_id: focused scheduler unit test
- execution_order: failure discovery 3
- exact command: `cargo test --manifest-path cmd/yai/Cargo.toml scheduler -- --nocapture`
- actual exit: nonzero before fix, zero after fix
- invariant exposed: the pure selector accidentally opened the default LMDB to resolve roots.

Fix: the selector accepts an injected root resolver; only its product adapter
reads LMDB. Unit selection is now storage-independent and status remains pure.

## E13-Q01 — focused implementation tests

- evidence_id: E13-Q01
- run_id: repository working tree
- execution_order: qualification 1
- exact command: `cargo test --manifest-path engine/yai-engine/Cargo.toml wave13 -- --nocapture`
- actual exit: 0

```text
running 3 tests
...wave13_runtime_instance_is_exclusive_reclaimable_and_noncanonical ... ok
...wave13_one_process_shares_one_lmdb_environment_across_workers ... ok
...wave13_work_items_are_idempotent_bounded_isolated_and_case_serialized ... ok
test result: ok. 3 passed; 0 failed; 107 filtered out
```

## E13-Q02 — complete Rust/H10/W11/W12 regression

- evidence_id: E13-Q02
- run_id: repository working tree
- execution_order: qualification 2
- exact commands:
  - `cargo test --manifest-path engine/yai-engine/Cargo.toml`
  - `cargo test --manifest-path cmd/yai/Cargo.toml`
- actual exits: 0, 0
- invariant: canonical authority, temporal governance, Tenant isolation and the new operational owner coexist without weakening historical replay.

```text
test result: ok. 110 passed; 0 failed; 0 ignored
test runtime_instance::tests::overlapping_or_unknown_resources_serialize ... ok
test runtime_instance::tests::tenant_round_robin_and_fifo_are_deterministic ... ok
test result: ok. 2 passed; 0 failed; 0 ignored
```

## E13-Q03 — repository check and historical smoke matrix

- evidence_id: E13-Q03
- run_id: repository working tree
- execution_order: qualification 3
- exact command: `make check`
- actual exit: 0
- invariant: layout/docs/build/full Rust tests, R1–R6 human review recovery, governance intake/materialization/admission, controlled effects, semantic continuity and the 26-turn governed runtime remain green.

Representative raw output:

```text
check-required-layout: ok
check-source-surface-clean: ok
check-doc-links: ok (28 files)
test result: ok. 110 passed; 0 failed
test result: ok. 2 passed; 0 failed
human_review:crash_r1_r6_recovery ok
case_runtime:agentless_26_turn_provider_model_replacement ok
governance_intake_characterization: pass
governance_hardening_characterization: pass
case_policy_materialization_characterization: pass
policy_authority:allow_chain ok
policy_authority:explicit_deny_and_no_match_fail_closed ok
```

## E13-Q04 — H10 through Wave13 product qualification

- evidence_id: E13-Q04
- run_id: repository working tree
- execution_order: qualification 4
- exact command: `make characterization smoke-policy-authority-hardening smoke-temporal-governance smoke-tenant-security smoke-multi-case-runtime`
- actual exit: 0
- invariant: H10 semantic re-derivation, Wave11 time/cancellation/closure, Wave12 Tenant isolation and Wave13 scheduling/recovery all pass in one unchanged tree.

```text
h10_authority_injection: forged_decision=authority_decision_basis_mismatch forged_grant=policy_execution_grant_semantic_mismatch ... grant_committed=false
policy_authority_hardening:canonical_write_rederivation ok
policy_authority_hardening:canonical_evidence_and_review ok
policy_authority_hardening:grant_adjacency_and_historical_replay ok
temporal_governance_characterization: pass
tenant_security_characterization: pass
cross_bind_exit: 2
root_exact_exit: 2
root_overlap_exit: 2
spoof_exit: 2
multi_case_runtime_characterization: pass
workers_max_observed: 2
split_brain_exit: 2
crash_exit: 85
```

## E13-Q05 — bounded 128-iteration and 26-turn endurance

- evidence_id: E13-Q05
- run_id: repository working tree
- execution_order: qualification 5
- exact command: `make endurance-agentless-case-runtime`
- actual exit: 0
- invariant: both 128-iteration residency/planning bodies and the real 26-turn governed runtime remain bounded after enabling concurrent workers.

```text
test residency::tests::hundred_iteration_case_state_memory_context_endurance ... ok
test residency::tests::hundred_iteration_planning_remains_bounded ... ok
test result: ok. 110 passed; 0 failed
case_runtime:agentless_26_turn_provider_model_replacement ok
case_runtime:deny_adaptation_and_bounded_residency ok
case_runtime:grant_effect_and_memory_restart_recovery ok
```

## E13-Q06 — formatting, Clippy delta, docs and whitespace

- evidence_id: E13-Q06
- run_id: repository working tree
- execution_order: qualification 6
- exact commands:
  - `cargo fmt --manifest-path engine/yai-engine/Cargo.toml --check`
  - `cargo fmt --manifest-path cmd/yai/Cargo.toml --check`
  - `cargo clippy --manifest-path engine/yai-engine/Cargo.toml --all-targets -- -D warnings` (baseline audit)
  - `cargo clippy ... -D warnings` with only the enumerated pre-Wave13 lint categories allowed
  - `make check-layout check-docs`
  - `git diff --check`
- actual exits: 0, 0, 101 for the unwaived baseline audit, 0/0 for the Wave13 delta checks, 0, 0
- invariant: Wave13 adds no Clippy diagnostic; formatting/docs/whitespace pass. The unwaived audit reports existing warnings in pre-Wave13 code (`too_many_arguments`, `should_implement_trait`, `needless_borrow`, `manual_pattern_char_comparison`, `unnecessary_map_or`, `ptr_arg`, `type_complexity`, `let_unit_value`). It is retained rather than falsely reported green.

```text
Checking yai-engine ...
Finished `dev` profile ...
Checking yai ...
Finished `dev` profile ...
check-required-layout: ok
check-source-surface-clean: ok
check-doc-links: ok (28 files)
```
