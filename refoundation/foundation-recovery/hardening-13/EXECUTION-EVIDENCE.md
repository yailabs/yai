# H13 Execution Evidence

All product blocks below come from one fresh isolated run of the dedicated characterization. Internal crash exits are asserted by the script; the overall command exits zero only after every recovery invariant passes. The pre-fix failing run is preserved separately in `FAILURE-EVIDENCE.md`.

## P-H13-01 — terminal acknowledgement crash/restart

```text
evidence_id: P-H13-01
run_id: /tmp/yai-h13-terminal-ack.DpNsox
execution_order: 01
pre-state: fresh YAI_HOME; one Tenant-scoped Case; WorkItem queued; provider invocation count zero
cwd: /home/mothx/computer-science/projects/YAI/yai
environment: YAI_HOME=/tmp/yai-h13-terminal-ack.DpNsox/home; local POSIX Principal; one-shot loopback fixture
exact command: make smoke-multi-case-runtime-hardening
actual exit: 0
produced IDs: work_id=runtime-work:2f7d1518e0994c05; runtime_instance_id=runtime-instance:local-default
invariant: lost scheduler acknowledgement cannot resurrect semantic completion
```

```text
h13_terminal_ack_reproduction: provider_count=1 final_state=Completed crash_exit=122
checkpoint_status: completed
```

The bounded scheduler log additionally records `status=completed`, the exact failpoint, `instance_admission: reclaimed_stale`, and `recovered_items: 1`, with no second dispatch for that WorkItem.

## P-H13-02 — AwaitingReview acknowledgement crash/restart

```text
evidence_id: P-H13-02
run_id: /tmp/yai-h13-terminal-ack.DpNsox
execution_order: 02
pre-state: fresh review Case under tenant:h13-b; no pending provider call; authenticated Principal linked to reviewer Participant
cwd: /home/mothx/computer-science/projects/YAI/yai
environment: same isolated run; review loopback fixture
exact command: target/debug/yai runtime serve --workers 2 --max-active-per-tenant 1 --max-queued-per-tenant 4 --max-queued-total 8 --failpoint after_case_runtime_awaiting_review_before_workitem_state_commit
actual exit: 123 (injected), followed by restart exit 0
produced IDs: work_id=runtime-work:929373e5b1333354; review_id=review:568a190fd7ccf8321c6bbb817f6af3cc
invariant: restart reconstructs WaitingReview without initial provider repeat and resumes exact work after authenticated approval
```

```text
h13_waiting_review_recovery: work_id=runtime-work:929373e5b1333354 review_id=review:568a190fd7ccf8321c6bbb817f6af3cc crash_exit=123 provider_before=1 provider_final=2 final_state=Completed
```

## P-H13-03 — worker panic containment

```text
evidence_id: P-H13-03
run_id: /tmp/yai-h13-terminal-ack.DpNsox
execution_order: 03
pre-state: separate Case and Running RuntimeInstance; zero provider calls
cwd: /home/mothx/computer-science/projects/YAI/yai
environment: same isolated run; workers=2
exact command: target/debug/yai runtime serve --workers 2 --max-active-per-tenant 1 --max-queued-per-tenant 4 --max-queued-total 8 --failpoint worker_panic_before_case_runtime
actual exit: 2, followed by restart exit 0
produced IDs: work_id=runtime-work:76b9a8bfb6f2ce90
invariant: panic is visible/fail-closed; no semantic outcome fabricated; restart recovers exactly once
```

```text
runtime_worker_event: stopped ... work_id=runtime-work:76b9a8bfb6f2ce90 status=worker_panicked
runtime_worker_panic: worker_id=worker:0 work_id=runtime-work:76b9a8bfb6f2ce90 detail=worker_panic_before_case_runtime
runtime_instance_degraded_by_worker_panic: worker_id=worker:0 work_id=runtime-work:76b9a8bfb6f2ce90
h13_worker_panic_recovery: work_id=runtime-work:76b9a8bfb6f2ce90 panic_exit=2 provider_count=1 final_state=Completed
```

## P-H13-04 — restart-stable fairness

```text
evidence_id: P-H13-04
run_id: /tmp/yai-h13-terminal-ack.DpNsox
execution_order: 04
pre-state: A1 A2 under tenant:h13; B1 B2 under tenant:h13-b; two disjoint Cases
cwd: /home/mothx/computer-science/projects/YAI/yai
environment: same isolated run; workers=2; max_active_per_tenant=1
exact command: target/debug/yai runtime serve ... --failpoint after_work_running_before_case_admission (twice), then target/debug/yai runtime serve ...
actual exit: 121, 121, 0
produced IDs: runtime-work:59624b61969c0e74 runtime-work:59624e61969c138d runtime-work:e9cbc1fe76f57bd8 runtime-work:e9cbc4fe76f580f1
invariant: cursor A survives first crash and selects B after restart; all work later completes
```

```text
h13_restart_fairness: first=tenant:h13 second=tenant:h13-b crash_exits=121,121 provider_counts=2,2 final=all_completed
fairness_work_ids: runtime-work:59624b61969c0e74 runtime-work:59624e61969c138d runtime-work:e9cbc1fe76f57bd8 runtime-work:e9cbc4fe76f580f1
```

## P-H13-05 / 06 — live-owner refusal and dead-owner reclaim

```text
evidence_id: P-H13-05-06
run_id: /tmp/yai-h13-terminal-ack.DpNsox
execution_order: 05
pre-state: RuntimeInstance owned by a live process, then the same process is crash-exited
cwd: /home/mothx/computer-science/projects/YAI/yai
environment: same isolated run; owner process fingerprint includes boot ID and /proc starttime
exact command: target/debug/yai runtime serve --workers 1 --max-active-per-tenant 1 --max-queued-per-tenant 4 --max-queued-total 4
actual exit: 2 while owner live; 0 after dead-owner recovery
produced IDs: runtime_instance_id=runtime-instance:local-default
invariant: token/Principal/lease cannot split-brain a live process; actual death remains reclaimable
```

```text
live_owner_split_exit: 2
live_owner_split_error: runtime_instance_active: principal_id=principal:72cc156b82060120eac8f7e234dbfcef owner_pid=722305 owner_process_identity=linux-proc-v1:be25bd71-5483-46f9-b242-2aff947b6e55:86858909 lease_expires_at_unix_ms=1788189320321
dead_owner_reclaim: instance_admission: reclaimed_stale
```

## P-H13-07 — wrong journal isolation

```text
evidence_id: P-H13-07
run_id: /tmp/yai-h13-terminal-ack.DpNsox
execution_order: 06
pre-state: Case A live; explicit journal contains only Case B refs
cwd: /home/mothx/computer-science/projects/YAI/yai
environment: same isolated run; YAI_JOURNAL points to wrong-journal.jsonl
exact command: target/debug/yai runtime submit --tenant tenant:h13 --case case:h13-terminal --subject subject:llm-provider --attachment workspace --prompt forged-journal --idempotency-key request:h13-wrong-journal
actual exit: 2
produced IDs: none
invariant: compatibility journal cannot cross-contaminate Case context
```

```text
wrong_journal_exit: 2
wrong_journal_error: journal_case_identity_mismatch: expected=case:h13-terminal observed=case:h13-wrong record_id=rec:h13-terminal-case
```

## P-H13-08 — shared LMDB and heartbeat stress

The exact focused qualification command and its bounded raw output are recorded in the final qualification block below. It uses worker counts 1/2/4/8, 480 shared-environment writes/read summaries, 512 eight-worker contention writes, repeated open/drop/open, a map-size mismatch negative, and a live RuntimeInstance heartbeat.

## P-H13-09 — normal Wave-13 concurrency

The unchanged `make smoke-multi-case-runtime` transcript is recorded in the final qualification block. It retains the real two-worker overlap, same-Case serialization, Tenant fairness/quota/backpressure, authenticated review park/resume, PREPARE crash/reconciliation, split-brain refusal and cross-Tenant rejection.

## Q-H13-01 — focused H13 engine and CLI contracts

```text
evidence_id: Q-H13-01
run_id: h13-focused-final
execution_order: 07
pre-state: final H13 source tree; no product fixture
cwd: /home/mothx/computer-science/projects/YAI/yai
exact commands: cargo test --manifest-path engine/yai-engine/Cargo.toml h13_ -- --nocapture; cargo test --manifest-path cmd/yai/Cargo.toml h13_ -- --nocapture
actual exits: 0; 0
produced IDs: none
invariant: FSM, process identity, schema compatibility, fairness, checkpoint and LMDB stress contracts are mechanically green
```

```text
test result: ok. 8 passed; 0 failed; 110 filtered out
test result: ok. 7 passed; 0 failed; 2 filtered out
h13_runtime_work_list_scale: terminal_items=100 elapsed_us=4711
h13_runtime_work_list_scale: terminal_items=1000 elapsed_us=46073
h13_runtime_work_list_scale: terminal_items=5000 elapsed_us=229537
h13_scheduler_scale: terminal_items=100 total_items=102 selector_elapsed_us=13 idle_tick_ms=50
h13_scheduler_scale: terminal_items=1000 total_items=1002 selector_elapsed_us=63 idle_tick_ms=50
h13_scheduler_scale: terminal_items=5000 total_items=5002 selector_elapsed_us=157 idle_tick_ms=50
h13_heartbeat_stress: workers=8 writes=512 max_heartbeat_ms=1 lease_margin_ms=5000
```

## Q-H13-02 — repository and Rust qualification

```text
evidence_id: Q-H13-02
run_id: h13-repository-final
execution_order: 08
pre-state: known Wave-13 v1 metadata compatibility failure fixed with exact predecessor admission
cwd: /home/mothx/computer-science/projects/YAI/yai
exact command: make check
actual exit: 0
produced IDs: none
invariant: layout/docs, 118 engine tests, 9 CLI tests, R1-R6, canonical authority, historical smokes and 26-turn runtime remain green
```

```text
check-required-layout: ok
check-source-surface-clean: ok
check-doc-links: ok (28 files)
test result: ok. 118 passed; 0 failed
test result: ok. 9 passed; 0 failed
human_review:crash_r1_r6_recovery ok
case_runtime:agentless_26_turn_provider_model_replacement ok
```

## Q-H13-03 — H10 through H13 product matrix

```text
evidence_id: Q-H13-03
run_id: h13-cross-wave-final
execution_order: 09
pre-state: same final tree; fresh isolated fixture per smoke
cwd: /home/mothx/computer-science/projects/YAI/yai
exact command: make characterization smoke-policy-authority-hardening smoke-temporal-governance smoke-tenant-security smoke-multi-case-runtime smoke-multi-case-runtime-hardening
actual exit: 0
produced IDs: normal_run=/tmp/yai-multi-case-runtime.lRADJQ; h13_run=/tmp/yai-h13-terminal-ack.SzhqjA
invariant: H10 semantic authority, Wave11 time/cancellation, Wave12 isolation, Wave13 concurrency and H13 recovery all coexist
```

```text
policy_authority_hardening:canonical_write_rederivation ok
policy_authority_hardening:canonical_evidence_and_review ok
policy_authority_hardening:grant_adjacency_and_historical_replay ok
temporal_governance_characterization: pass
tenant_security_characterization: pass
multi_case_runtime_characterization: pass
h13_hardening_characterization: pass
```

## Q-H13-04 — bounded endurance

```text
evidence_id: Q-H13-04
run_id: h13-endurance-final
execution_order: 10
pre-state: final tree
cwd: /home/mothx/computer-science/projects/YAI/yai
exact command: make endurance-agentless-case-runtime
actual exit: 0
produced IDs: none
invariant: both source bodies execute 128 iterations and the governed 26-turn runtime remains bounded
```

```text
test residency::tests::hundred_iteration_case_state_memory_context_endurance ... ok
test residency::tests::hundred_iteration_planning_remains_bounded ... ok
case_runtime:agentless_26_turn_provider_model_replacement ok
```

## Q-H13-05 — hygiene and Clippy delta

```text
evidence_id: Q-H13-05
run_id: h13-hygiene-final
execution_order: 11
pre-state: complete dossier and source tree
cwd: /home/mothx/computer-science/projects/YAI/yai
exact commands: cargo fmt --manifest-path engine/yai-engine/Cargo.toml --check; cargo fmt --manifest-path cmd/yai/Cargo.toml --check; cargo clippy --manifest-path engine/Cargo.toml --workspace --all-targets; cargo clippy --manifest-path cmd/yai/Cargo.toml --all-targets; make check-layout check-docs; git diff --check
actual exits: 0 for every command
produced IDs: none
invariant: no new H13 Clippy diagnostic; formatting, documentation, layout and whitespace pass
```

The unchanged repository baseline remains 14 engine-library warnings and 17
CLI-binary warnings. H13-added test/helper diagnostics were removed before this
final audit.
