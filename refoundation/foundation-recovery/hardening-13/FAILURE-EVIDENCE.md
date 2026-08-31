# H13 Failure Evidence

This file preserves the unchanged Wave-13 recovery failure before its repair. The only production change used for the reproduction was a scheduler crash failpoint after the existing Case runtime returned a terminal checkpoint and before the existing WorkItem update. It did not alter recovery behavior.

## H13-F01 — lost terminal acknowledgement resurrects a completed checkpoint

```text
evidence_id: H13-F01
run_id: /tmp/yai-h13-terminal-ack.8sw1HN
execution_order: 1
pre-state: baseline 5e1ae2fa286cb1052d480c79ed3b84410d881035 plus observation-only crash failpoint; one queued WorkItem; zero provider invocations
cwd: /home/mothx/computer-science/projects/YAI/yai
command: YAI_KEEP_TEST_DIR=1 tests/characterization/multi-case-runtime-hardening/test_terminal_ack_reproduction.sh
actual_exit: 1
produced_ids: work_id=runtime-work:2f7d1518e0994c05 run_id=case-run:85fc15a9f4daf733 provider_result_id=provider-result:case:h13-terminal:model-output-33
invariant_expected: terminal checkpoint repairs Running WorkItem without Case re-entry
invariant_observed: violated
```

Bounded raw output:

```text
runtime_worker_event: stopped timestamp_unix_ms=1788185614875 worker_id=worker:0 work_id=runtime-work:2f7d1518e0994c05 status=completed
runtime_instance_crash_injected: after_case_runtime_terminal_before_workitem_terminal_commit work_id=runtime-work:2f7d1518e0994c05 checkpoint_status=completed
instance_admission: reclaimed_stale
recovered_items: 1
runtime_dispatch: work_id=runtime-work:2f7d1518e0994c05 worker_id=worker:0 reason=tenant_round_robin tenant=tenant:h13 fifo_sequence=1 resource_relation=no_active_conflict
runtime_worker_event: started timestamp_unix_ms=1788185614922 worker_id=worker:0 work_id=runtime-work:2f7d1518e0994c05 tenant_id=tenant:h13 case_id=case:h13-terminal
provider_retry: 1 reason:failed to connect provider: Connection refused (os error 111)
runtime_worker_event: stopped timestamp_unix_ms=1788185614936 worker_id=worker:0 work_id=runtime-work:2f7d1518e0994c05 status=provider_failure_budget_exhausted
h13_terminal_ack_reproduction: provider_count=1 final_state=Failed crash_exit=122
```

The terminal checkpoint initially contained `status=completed`, `invocations=1`, `operations=0`, and the canonical provider result above. Recovery changed `Running → Queued`; `update_resume_budgets` then rewrote `Completed → Running`; the second execution attempt reached the provider boundary and failed because the one-shot fixture had already exited. No second canonical ProviderInvocation/Operation/Decision/Grant/effect was committed in this run, but the duplicate provider-call attempt and terminal-state resurrection were real.

## H13-F01-R — unchanged reproduction after repair

```text
evidence_id: H13-F01-R
run_id: /tmp/yai-h13-terminal-ack.IraqzA
execution_order: 1
pre-state: fresh isolated product home; same failpoint and one-shot provider
cwd: /home/mothx/computer-science/projects/YAI/yai
command: YAI_KEEP_TEST_DIR=1 tests/characterization/multi-case-runtime-hardening/test_terminal_ack_reproduction.sh
actual_exit: 0
produced_ids: work_id=runtime-work:2f7d1518e0994c05
invariant: terminal checkpoint repairs Running WorkItem without Case re-entry
```

```text
runtime_worker_event: stopped timestamp_unix_ms=1788187383459 worker_id=worker:0 work_id=runtime-work:2f7d1518e0994c05 status=completed
runtime_instance_crash_injected: after_case_runtime_terminal_before_workitem_terminal_commit work_id=runtime-work:2f7d1518e0994c05 checkpoint_status=completed
instance_admission: reclaimed_stale
recovered_items: 1
h13_terminal_ack_reproduction: provider_count=1 final_state=Completed crash_exit=122
```

No post-restart dispatch line exists for this WorkItem. Provider, Operation, Decision, Grant and effect counts therefore remain unchanged.

## H13-F02 — Wave-13 operational schema marker rejected before v1 reader

```text
evidence_id: H13-F02
run_id: h13-repository-check-first
execution_order: 2
pre-state: final H13 implementation before metadata predecessor was admitted; existing Wave-13 operational store created with yai.runtime_instance.v1
cwd: /home/mothx/computer-science/projects/YAI/yai
command: make check
actual_exit: 2
produced_ids: none
invariant_expected: known Wave-13 RuntimeInstance v1 operational state remains readable while v2 ownership is introduced
invariant_observed: violated before the v1 record decoder was reached
```

Bounded raw output:

```text
record store import failed after journal write remained at build/tmp/new12/daemon-680885/filesystem/journal.jsonl: unsupported_persisted_schema: meta:runtime_instance_schema expected=yai.runtime_instance.v2 actual=yai.runtime_instance.v1
make: *** [Makefile:539: smoke-controlled-effect] Error 2
```

The fix admits only the exact known predecessor marker through the existing
schema-upgrade transaction. It rewrites the marker to v2, retains v1 record
integrity validation, and continues to reject unknown/future schema markers.

## H13-F02-R — unchanged repository check after repair

```text
evidence_id: H13-F02-R
run_id: h13-repository-final
execution_order: 2
pre-state: same accumulated operational fixtures; exact v1 predecessor admitted
cwd: /home/mothx/computer-science/projects/YAI/yai
command: make check
actual_exit: 0
produced_ids: none
invariant: Wave-13 stores upgrade safely; all repository gates remain green
```

```text
test store::lmdb::tests::h13_wave13_runtime_instance_schema_marker_upgrades_without_rejecting_store ... ok
test result: ok. 118 passed; 0 failed
test result: ok. 9 passed; 0 failed
case_runtime:agentless_26_turn_provider_model_replacement ok
human_review:crash_r1_r6_recovery ok
```
