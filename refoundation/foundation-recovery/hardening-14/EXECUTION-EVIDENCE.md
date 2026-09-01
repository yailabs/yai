# H14 execution evidence

These are bounded raw excerpts from actual executions on 2026-09-01. The final
blocks are bound to semantic commit
`0b48edee499f7b74b3a529f728af7912a24d0e5a`; outputs are never reconstructed.

## H14-E01 — content-valid forgery and history rebuild

- evidence_id: H14-E01
- run_id: cargo-h14-focused-20260901-01
- execution_order: 1
- pre-state: isolated temporary LMDB; no resource state for the test identities
- cwd: `/home/mothx/computer-science/projects/YAI/yai/engine`
- exact command: `cargo test -p yai-engine h14_ -- --nocapture`
- exit: 0
- invariant: integrity-valid bytes are not authority; v2 history rebuilds exact

```text
h14_fence_forgery: canonical_fence=resource-fence:sha256:fa357c2e1580544356db11a91 forged_fence=resource-fence:sha256:0f12f1ceb7b14b6ce7e5cadb1 result=stale_resource_fence: requested_epoch=1 current_epoch=1 physical_mutations=0
h14_resource_rebuild: resource=resource-control:sha256:9d390cd8df0736531db7f7415 events=1 epoch=1 active=true case_generation_unchanged=11 invalid_append=resource_history_invalid_acquisition
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 122 filtered out
```

## H14-E02 — eight-process contention, reclaim and terminal atomicity

- evidence_id: H14-E02
- run_id: `/tmp/yai-h14-contention.SQJk2C`
- execution_order: 2
- pre-state: eleven Tenant Cases, one shared filesystem root, no resource owner
- cwd: `/home/mothx/computer-science/projects/YAI/yai`
- environment: local Linux; isolated YAI_HOME; eight independent YAI processes
- exact command: `tests/characterization/shared-resource-fencing-hardening/test_shared_resource_fencing_hardening.sh`
- exit: 0
- produced IDs: resource and effect IDs remain in the isolated raw run directory
- invariant: one winner, monotonic epochs, one reclaim, atomic release

```text
h14_multiprocess_contention: pass
test_run_id: /tmp/yai-h14-contention.SQJk2C
contender_processes: 8
acquisition_winners: 1
acquisition_blocked: 7
resource_id: resource-control:sha256:1387d15d7817307b98f985adc
first_epoch: 1
same_effect_reclaim_epoch: 2
next_acquisition_epoch: 3
terminal_commit_epoch: 4
terminal_failpoint_exit: 89
post_terminal_acquisition_epoch: 5
terminal_recovery_posture: already_finalized
reconcile_exit_one: 0
reconcile_exit_two: 2
physical_mutations_per_effect: 1
```

## H14-E03 — resource busy parks and safely retries the same work

- evidence_id: H14-E03
- run_id: `/tmp/yai-wave14-cross-process.A7Zxv3`
- execution_order: 3
- pre-state: direct Case holds shared resource; RuntimeInstance owns a separate WorkItem
- cwd: `/home/mothx/computer-science/projects/YAI/yai`
- environment: workers=1; generic local providers; one Tenant
- exact command: `tests/characterization/shared-resource-fencing/test_cross_process_fencing.sh`
- exit: 0
- invariant: transient physical admission is Blocked, not DENY/Failed; release reuses the same WorkItem only after canonical authority freshness checks

```text
cross_process_resource_fencing: pass
test_run_id: /tmp/yai-wave14-cross-process.A7Zxv3
direct_exit: 85
direct_effect_id: effect:418050cd699f5f339c8eadd71872f90c
resource_id: resource-control:sha256:5af8b45f5eef41c8c082b294a
resource_epoch: 1
resource_fence_id: resource-fence:sha256:28d22fb8dee375c2a337aa172
runtime_work_id: runtime-work:db0152067a521e7d
runtime_work_initial_state: Blocked
runtime_work_final_state: Completed
runtime_block_reason: resource_temporarily_owned
runtime_retry_trigger: terminal_resource_release
direct_peer_exit: 2
direct_peer_block_reason: resource_temporarily_owned
physical_mutations_before_reconcile: 0
physical_mutations_after_reconcile: 1
```

## H14-E04 — uncertain process signal is observation-only

- evidence_id: H14-E04
- run_id: `/tmp/yai-h14-process-uncertainty.hPSLBe`
- execution_order: 4
- pre-state: explicitly spawned test-owned child; exact birth identity attached
- cwd: `/home/mothx/computer-science/projects/YAI/yai`
- exact command: `tests/characterization/shared-resource-fencing-hardening/test_process_uncertainty.sh`
- exit: 0
- invariant: crash after accepted signal never causes a blind second signal

```text
h14_process_uncertainty: pass
test_run_id: /tmp/yai-h14-process-uncertainty.hPSLBe
fixture_pid: 1579851
signal_carrier_exit: 88
effect_id: effect:aafd81b31bc956095020cbbeda324a1d
resource_id: resource-control:sha256:f18c992c37787d0c9e408194b
recovery_mode: observation_only
signal_repeated_during_recovery: false
effect_posture: indeterminate
```

## H14-Y01 — black-box external provider probe

- evidence_id: H14-Y01
- run_id: `yvex-black-box-20260901T104023Z-1605952`
- execution_order: 5
- pre-state: no endpoint/model configuration supplied; default loopback probe only
- cwd: `/home/mothx/computer-science/projects/YAI/yai`
- environment: provider URL and model unset; no secret values printed
- exact command: `tests/integration/yvex/qualification_yvex_provider.sh`
- exit: 3
- invariant: YAI does not inspect/administer YVEX and never fabricates a live pass

```text
qualification_mode: black_box_openai_compatible_provider
yvex_repository_accessed: false
yvex_cli_used: false
provider_endpoint: http://127.0.0.1:8001/v1
yvex_external_qualification_state: blocked_external_dependency
reason: no reachable OpenAI-compatible models endpoint at http://127.0.0.1:8001/v1/models
```

No model ID, ContextFrame ID, invocation ID, DENY/REVIEW/resource-busy model
flow or X/Y/Z epistemic result exists for this run because no external model was
reachable. None is fabricated.

## H14-Q01 — full Rust and repository check

- evidence_id: H14-Q01
- run_id: make-check-h14-20260901-01
- execution_order: 6
- pre-state: complete pre-publication H14 tree
- cwd: `/home/mothx/computer-science/projects/YAI/yai`
- exact command: `make check`
- exit: 0
- invariant: layout/docs, all Rust tests and default deterministic smoke remain green

```text
check-required-layout: ok
check-doc-links: ok (28 files)
test result: ok. 130 passed; 0 failed
test result: ok. 11 passed; 0 failed
case_runtime:agentless_26_turn_provider_model_replacement ok
human_review:crash_r1_r6_recovery ok
policy_authority:unconfigured_pre_provider_stop ok
```

## H14-Q02 — cross-wave smoke matrix

- evidence_id: H14-Q02
- run_id: make-smoke-w10-h14-20260901-01
- execution_order: 7
- pre-state: isolated fixture state per target
- cwd: `/home/mothx/computer-science/projects/YAI/yai`
- exact command: `make smoke-policy-authority-hardening smoke-temporal-governance smoke-tenant-security smoke-multi-case-runtime smoke-multi-case-runtime-hardening smoke-shared-resource-fencing smoke-second-carrier smoke-shared-resource-fencing-hardening`
- exit: 0
- invariant: authority, time/cancel, Tenant, concurrency, fencing and both carriers remain compatible

```text
policy_authority_hardening:canonical_write_rederivation ok
temporal_governance_characterization: pass
tenant_security_characterization: pass
multi_case_runtime_characterization: pass
h13_hardening_characterization: pass
shared_resource_fencing_characterization: pass
second_carrier_characterization: pass
h14_multiprocess_contention: pass
h14_process_uncertainty: pass
```

## H14-Q03 — bounded endurance and static checks

- evidence_id: H14-Q03
- run_id: h14-endurance-static-20260901-01
- execution_order: 8
- pre-state: complete formatted tree
- cwd: `/home/mothx/computer-science/projects/YAI/yai`
- exact commands: `make endurance-agentless-case-runtime`; `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets`; `cargo clippy --all-targets`; `make check-layout check-docs`; `git diff --check`
- exit: 0 for every repository-contract command
- invariant: 26 turns, both bounded endurance bodies, format/docs/layout/diff and existing Clippy contract remain qualified

```text
test residency::tests::hundred_iteration_case_state_memory_context_endurance ... ok
test residency::tests::hundred_iteration_planning_remains_bounded ... ok
case_runtime:agentless_26_turn_provider_model_replacement ok
check-required-layout: ok
check-doc-links: ok (28 files)
```

Clippy reported only the repository's pre-existing baseline warning families
(14 engine, 16 CLI after removing H14's new response tuple warning) and exited
0 under the existing contract. `-D warnings` is not the repository baseline and
was recorded only as a stricter diagnostic, not misreported as a gate.

## H14-P01 — publication-bound focused authority proof

- evidence_id: H14-P01
- run_id: cargo-h14-focused-20260901-published
- execution_order: 9
- pre-state: clean H14 semantic commit plus the 13 preserved historical entries
- cwd: `/home/mothx/computer-science/projects/YAI/yai/engine`
- exact YAI SHA: `0b48edee499f7b74b3a529f728af7912a24d0e5a`
- exact command: `cargo test -p yai-engine h14_ -- --nocapture`
- exit: 0
- invariant: exact semantic commit rejects content-valid forgery and rebuilds exact state

```text
h14_fence_forgery: canonical_fence=resource-fence:sha256:f5b35e32473c6df829ac7821b forged_fence=resource-fence:sha256:ad1ac67b30b7477be045e8083 result=stale_resource_fence: requested_epoch=1 current_epoch=1 physical_mutations=0
h14_resource_rebuild: resource=resource-control:sha256:3bb4cc8cc40348e8207ccd84b events=1 epoch=1 active=true case_generation_unchanged=11 invalid_append=resource_history_invalid_acquisition
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 122 filtered out
```

## H14-P02 — publication-bound contention and uncertainty

- evidence_id: H14-P02
- run_id: `/tmp/yai-h14-contention.GENZY0`; `/tmp/yai-h14-process-uncertainty.RmXH36`
- execution_order: 10
- pre-state: isolated fixture stores and test-owned process
- cwd: `/home/mothx/computer-science/projects/YAI/yai`
- exact YAI SHA: `0b48edee499f7b74b3a529f728af7912a24d0e5a`
- exact command: `make smoke-shared-resource-fencing-hardening`
- exit: 0
- invariant: one of eight acquisition winners; one reclaim; no blind process retry

```text
h14_multiprocess_contention: pass
test_run_id: /tmp/yai-h14-contention.GENZY0
contender_processes: 8
acquisition_winners: 1
acquisition_blocked: 7
resource_id: resource-control:sha256:9e3c129cdb46de55427245243
first_epoch: 1
same_effect_reclaim_epoch: 2
next_acquisition_epoch: 3
terminal_commit_epoch: 4
post_terminal_acquisition_epoch: 5
terminal_recovery_posture: already_finalized
physical_mutations_per_effect: 1
h14_process_uncertainty: pass
test_run_id: /tmp/yai-h14-process-uncertainty.RmXH36
fixture_pid: 1654372
signal_carrier_exit: 88
effect_id: effect:4163c7a908b0d91000601abf8d2284cf
resource_id: resource-control:sha256:15e9dc6cdd13eb8c8f378c9af
recovery_mode: observation_only
signal_repeated_during_recovery: false
effect_posture: indeterminate
```

## H14-P03 — publication-bound retryable cross-process admission

- evidence_id: H14-P03
- run_id: `/tmp/yai-wave14-cross-process.LrKi04`
- execution_order: 11
- pre-state: one direct Prepared effect and one RuntimeInstance WorkItem share a root
- cwd: `/home/mothx/computer-science/projects/YAI/yai`
- exact YAI SHA: `0b48edee499f7b74b3a529f728af7912a24d0e5a`
- exact command: `tests/characterization/shared-resource-fencing/test_cross_process_fencing.sh`
- exit: 0
- invariant: external direct run cannot bypass shared authority; same WorkItem parks and resumes

```text
cross_process_resource_fencing: pass
test_run_id: /tmp/yai-wave14-cross-process.LrKi04
resource_id: resource-control:sha256:778fd6ae487741b7878af1a09
resource_epoch: 1
resource_fence_id: resource-fence:sha256:e4c41bf544af3aaa6efdf3ccf
runtime_work_id: runtime-work:db0152067a521e7d
runtime_work_initial_state: Blocked
runtime_work_final_state: Completed
runtime_block_reason: resource_temporarily_owned
runtime_retry_trigger: terminal_resource_release
direct_peer_exit: 2
physical_mutations_before_reconcile: 0
physical_mutations_after_reconcile: 1
```

## H14-Y02 — publication-bound black-box provider block

- evidence_id: H14-Y02
- run_id: `yvex-black-box-20260901T110023Z-1655196`
- execution_order: 12
- pre-state: no provider endpoint/model supplied or reachable
- cwd: `/home/mothx/computer-science/projects/YAI/yai`
- exact YAI SHA: `0b48edee499f7b74b3a529f728af7912a24d0e5a`
- exact command: `tests/integration/yvex/qualification_yvex_provider.sh`
- exit: 3
- invariant: black-box-only qualification does not inspect provider plumbing or claim a pass

```text
qualification_mode: black_box_openai_compatible_provider
yvex_repository_accessed: false
yvex_cli_used: false
provider_endpoint: http://127.0.0.1:8001/v1
yvex_external_qualification_state: blocked_external_dependency
reason: no reachable YVEX OpenAI-compatible models endpoint at http://127.0.0.1:8001/v1/models
```
