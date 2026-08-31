# Wave 14 execution evidence

All blocks below are bounded transcripts from actual executions on 2026-08-31.
`YAI SHA` is recorded as `7b2312a...+wave14-working-tree` until the isolated
Wave 14 semantic commit is created; the post-commit qualification block will
bind the same implementation to its published SHA.

## W14-E01 — epoch, conflict, release and stale-carrier rejection

- evidence_id: W14-E01
- run_id: cargo-wave14-focused-20260831-01
- execution_order: 1
- pre-state: two open Tenant Cases, same canonical filesystem root, no resource-control state
- cwd: `/home/mothx/computer-science/projects/YAI/yai`
- environment: local Linux; isolated temporary LMDB; no external provider
- exact command: `cargo test --manifest-path engine/Cargo.toml -p yai-engine wave14_ -- --nocapture`
- exit: 0
- produced IDs: printed below
- invariant: only one active fence; release advances the next acquisition; the old carrier cannot mutate

```text
wave14_fencing: resource=resource-control:sha256:280d0af72870bb52bb56a7078 epoch1=1 blocked=resource_temporarily_owned: resource_id=resource-control:sha256:280d0af72870bb52bb56a7078 epoch=1 case_id=case:wave14-a effect_id=effect:c5001422720f69f505e06b4455184ce9 epoch2=2 stale=stale_resource_fence: requested_epoch=1 current_epoch=2 final=epoch-two history_events=4
test store::lmdb::tests::wave14_shared_resource_epoch_blocks_competitor_and_stale_carrier ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 120 filtered out
```

## W14-E02 — real process-signal carrier

- evidence_id: W14-E02
- run_id: make-smoke-second-carrier-20260831-01
- execution_order: 2
- pre-state: test-owned child process; exact Linux birth identity attached; process policy published and bound
- cwd: `/home/mothx/computer-science/projects/YAI/yai`
- environment: Linux boot identity `be25bd71-5483-46f9-b242-2aff947b6e55`; isolated LMDB
- exact command: `make smoke-second-carrier`
- exit: 0
- produced IDs: printed below
- invariant: process proposal uses the same Decision/Grant path; fence is carrier-validated; kernel acceptance and observation are distinct

```text
wave14_process_carrier: fixture_pid=1034 boot_id=be25bd71-5483-46f9-b242-2aff947b6e55 start_ticks=87982433 operation=operation:3551bbaefae17f023557c68153f98d5c decision=decision:2e4bc4a2903bd60c83fe7bbf71e7167f grant=grant:778504375473edf43c9a8770c5e8f6d3 resource=resource-control:sha256:f418c0eb3378c2728c8989b32 epoch=1 fence=resource-fence:sha256:44227621f0ce9eb4a74212ee1 signal=19 syscall_accepted=true observed_state=Running receipt=effect-receipt:bb8f72671cb67b1751c0a708b24af8b9 finalized=true
second_carrier_characterization: pass
```

The child PID belongs only to the test fixture. The carrier never targets YAI,
YVEX, the shell, or another host process.

## W14-E03 — RuntimeInstance versus independent direct process

- evidence_id: W14-E03
- run_id: `/tmp/yai-wave14-cross-process.gFysed`
- execution_order: 3
- pre-state: two Cases in `tenant:wave14`, one exact shared root, valid independent policy/Grant paths
- cwd: `/home/mothx/computer-science/projects/YAI/yai`
- environment: isolated YAI_HOME; one local provider fixture per Case; RuntimeInstance workers=1
- authenticated principal: kernel-authenticated catalog owner (ID retained inside isolated transcript)
- exact command: `tests/characterization/shared-resource-fencing/test_cross_process_fencing.sh`
- exit: 0
- produced IDs: printed below
- invariant: scheduler-local visibility is irrelevant; the shared resource owner rejects the second PREPARE and physical mutation remains zero

```text
cross_process_resource_fencing: pass
test_run_id: /tmp/yai-wave14-cross-process.gFysed
direct_exit: 85
direct_effect_id: effect:3fe6fc5fa9102228eb2baec190f85791
resource_id: resource-control:sha256:03729cc8969652eae1101e10f
resource_epoch: 1
resource_fence_id: resource-fence:sha256:19839026d4721e38af02c28e9
runtime_work_id: runtime-work:db0152067a521e7d
runtime_work_state: Failed
runtime_block_reason: resource_temporarily_owned
physical_mutations_before_reconcile: 0
physical_mutations_after_reconcile: 1
```

The operational WorkItem is terminal `Failed`, not a policy DENY. Its Case
Decision remained ALLOW; physical admission failed at PREPARE. A future
Workflow resolver may model this as retryable, but Wave 14 does not implement
Workflow state.

## W14-E04 — full engine and CLI unit suites during smoke

- evidence_id: W14-E04
- run_id: make-wave14-smokes-20260831-01
- execution_order: 4
- pre-state: Wave 14 working tree
- cwd: `/home/mothx/computer-science/projects/YAI/yai`
- exact command: `make smoke-shared-resource-fencing && make smoke-second-carrier`
- exit: 0
- invariant: all old authority, temporal, Tenant and RuntimeInstance unit contracts remain green

```text
running 122 tests
test result: ok. 122 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
running 9 tests
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
shared_resource_fencing_characterization: pass
second_carrier_characterization: pass
```

## W14-Y01 — external YVEX dependency probe

- evidence_id: W14-Y01
- run_id: yvex-probe-20260831-01
- execution_order: 5
- pre-state: exact read-only external checkout as found
- cwd: `/home/mothx/computer-science/projects/YAI/yai`
- environment: `YVEX_REPO=/tmp/yvex-research.bZQXAZ/repo`, `YVEX_BASE_URL=http://127.0.0.1:8001/v1`; no credentials printed
- exact command: `YVEX_REPO=/tmp/yvex-research.bZQXAZ/repo tests/integration/yvex/qualification_yvex_provider.sh`
- exit: 3
- invariant: a missing live external dependency cannot be reported as a semantic pass

```text
yvex_repository: /tmp/yvex-research.bZQXAZ/repo
yvex_branch: main
yvex_sha: 2df3b84cc840dfca8b38f6fc387a833169b5598e
yvex_origin: 2df3b84cc840dfca8b38f6fc387a833169b5598e
yvex_dirty: clean
yvex_endpoint: http://127.0.0.1:8001/v1
yvex_health: curl: (7) Failed to connect to 127.0.0.1 port 8001
yvex_external_qualification_state: blocked_external_dependency
reason: no reachable OpenAI-compatible /v1/models endpoint
models_probe: curl: (7) Failed to connect to 127.0.0.1 port 8001
```

The read-only executable separately reported `yvex 0.1.0 protocol=11`, no
private runtime socket and `MODELS count=0`. No YVEX live Case, invocation or
introspection ID exists, and none is fabricated.

## W14-F01 — live-process fence ownership audit defect

- evidence_id: W14-F01
- run_id: code-audit-wave14-owner-binding-20260831-01
- execution_order: 6
- pre-state: first Wave 14 carrier-validation implementation
- exact reproduction: construct a structurally valid fence whose owner PID and birth marker identify a different live local process, then call the lowest carrier fence validator from the current process
- observed before fix: validator established that the named process was live but did not establish that it was the executing carrier process
- physical mutation in discovery: not attempted
- fix: prepare/reclaim require `owner_pid == current_process`; carrier validation requires current PID and exact current birth identity in addition to canonical fence/state equality
- unchanged negative after fix: wrong live process returns `resource_fence_owner_process_mismatch`
- invariant: fence bytes plus knowledge of another live PID are not bearer authority

This defect was found by direct source-boundary audit before a destructive
carrier reproduction; therefore there is no fabricated raw failing syscall
transcript.

## Post-commit qualification

This section is populated by the evidence-only publication correction after
the semantic Wave 14 commit is fixed and the full qualification is rerun at an
exact SHA.
