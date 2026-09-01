# Wave 15 execution evidence

These are bounded, unedited excerpts from single pre-publication runs. A
post-semantic-commit evidence pass replaces/binds them to the published
semantic SHA without mixing run output.

## P15-01/P15-05/P15-06/P15-07/P15-11/P15-12 — definition, input, deterministic effect, two Cases and pinning

- evidence_id: P15-DETERMINISTIC-01
- run_id: w15-prepublish-20260901-kernel-s0xUaU
- execution_order: 01–11
- pre-state: empty temporary YAI_HOME; two new Cases; two disjoint roots
- cwd: `/home/mothx/computer-science/projects/YAI/yai`
- environment: `YAI_EXECUTION_EVIDENCE=1`; local kernel-authenticated Principal
- authenticated principal: `principal:72cc156b82060120eac8f7e234dbfcef`
- Tenant: `tenant:workflow-product`
- Cases: `case:workflow-product`, `case:workflow-product-b`
- WorkflowDefinition: `workflow-definition:f2101d881787f1d0e30cc6c714597932`
- WorkflowBindings: `case-workflow-binding:893e961a842b65bcc3a6a5acca2e9d6a`, `case-workflow-binding:043f3d3589296c2ad48dcf386507e280`
- exact command: `YAI_EXECUTION_EVIDENCE=1 tests/characterization/workflow-kernel/test_workflow_kernel.sh`
- exit: 0

```text
workflow_definition: accepted
workflow_definition_id: workflow-definition:f2101d881787f1d0e30cc6c714597932
workflow_definition_digest: sha256:e8b0238f9fc61a642983ad85713e724799568c77f8200c7a23a99e213eb07e08
tenant_id: tenant:workflow-product
workflow_key: controlled-remediation
declared_version: 1
nodes: 6
edges: 5
workflow_definitions: 2
workflow: workflow-definition:f2101d881787f1d0e30cc6c714597932 key=controlled-remediation version=1 nodes=6 edges=5
workflow: workflow-definition:ec919a82dc509f7bc7afca9fb8eec4b9 key=controlled-remediation version=2 nodes=1 edges=0
workflow_human_input: accepted
case_id: case:workflow-product
node_id: change-ticket
review_action_created: false
completed: true
satisfied: 5
skipped: 1
node: apply-remediation kind=deterministic_work posture=Satisfied reason=canonical_satisfaction_recorded execution=workflow-execution:70f8c5d300e1e005
node: manual-analysis-exception kind=model_work posture=Skipped reason=conditional_branch_not_selected execution=none
runtime_status: Completed
stop_detail: workflow deterministic effect finalized; provider_invocations=0
invocations: 0
operations: 1
last_effect_id: effect:ea6f8d3105889faa308a423fdfd2e002
workflow_work_materialized: 2
runtime_worker_event: started timestamp_unix_ms=1788268703063 worker_id=worker:0 work_id=runtime-work:dcb679bd75ce46f3 tenant_id=tenant:workflow-product case_id=case:workflow-product
runtime_worker_event: started timestamp_unix_ms=1788268703064 worker_id=worker:1 work_id=runtime-work:a9b35a5ef80af83e tenant_id=tenant:workflow-product case_id=case:workflow-product-b
effect_state: finalized
effect_state: finalized
workflow_kernel_characterization: pass
shared_definition_case_count: 2
definition_pinning: v1_case_unchanged_after_v2
provider_invocations: 0
```

Invariant: the same immutable v1 Definition has independent Case bindings and
histories; v2 is a distinct object; HumanInput is not Review; two deterministic
nodes overlap on two bounded workers and both traverse normal authority/fencing
with zero provider calls.

## P15-02 — one-turn ModelWork

- evidence_id: P15-MODELWORK-01
- run_id: w15-prepublish-20260901-modelwork-49968
- execution_order: 01–03
- pre-state: empty temporary store; one loopback generic provider
- cwd: `/home/mothx/computer-science/projects/YAI/yai`
- environment: `YAI_EXECUTION_EVIDENCE=1`; provider model `model:workflow-fixture`
- Tenant: `tenant:workflow-model`
- Case: `case:new12-filesystem`
- WorkflowDefinition: `workflow-definition:6810091aa6ca3a068f34cc2f95693bfd`
- node/execution: `analyze` / `workflow-execution:fb3d75265b490b41`
- exact command: `YAI_EXECUTION_EVIDENCE=1 tests/characterization/workflow-kernel/test_workflow_modelwork.sh`
- exit: 0

```text
completed: true
satisfied: 1
node: analyze kind=model_work posture=Satisfied reason=canonical_satisfaction_recorded execution=workflow-execution:fb3d75265b490b41
runtime_status: Completed
stop_detail: canonical workflow completion predicate satisfied
invocations: 1
operations: 0
last_provider_result_id: provider-result:case:new12-filesystem:model-output-3
```

## P15-03 — iterative agentless ModelWork through governed effects

- evidence_id: P15-MODELWORK-ITERATIVE-01
- run_id: w15-prepublish-20260901-modelwork-49968
- execution_order: 04–06
- pre-state: isolated Case and generic provider in adaptive mode
- Tenant: `tenant:workflow-iterative`
- Case: `case:new12-filesystem`
- WorkflowDefinition: `workflow-definition:aab6df58fd11d23d7d6b8f5dd7d73e4c`
- node/execution: `remediate` / `workflow-execution:7172bc4ba962c9d3`
- exact command: same script and run, isolated second scenario
- exit: 0

```text
completed: true
satisfied: 1
node: remediate kind=model_work posture=Satisfied reason=canonical_satisfaction_recorded execution=workflow-execution:7172bc4ba962c9d3
runtime_status: Completed
stop_detail: canonical workflow completion predicate satisfied
invocations: 2
operations: 2
last_provider_result_id: provider-result:case:new12-filesystem:model-output-7
last_effect_id: effect:d29774bea317037377ff78cb4c23a034
last_effect_outcome: Applied
iterative_modelwork_provider_invocations: 2
iterative_modelwork_operations: 2
second_turn_observed_first_effect: true
```

Invariant: one stable workflow execution performs two turns; the second
ContextFrame reports the first finalized `allowed/step-00.txt` consequence,
and only exact finalized `allowed/step-01.txt` satisfies completion. No Agent
record or loop owner exists.

## P15-04 — provider “done” is not effect truth

- evidence_id: P15-FALSE-COMPLETION-01
- run_id: w15-prepublish-20260901-modelwork-49968
- execution_order: 07–09
- pre-state: separate store; provider returns typed completion
- WorkflowDefinition: `workflow-definition:385947905afeb1445dcae8d050080662`
- node: `effect-required`
- exact command: same script and run, isolated third scenario
- exit: 0

```text
completed: false
satisfied: 0
active: 1
node: effect-required kind=model_work posture=Active reason=workflow_execution_active execution=workflow-execution:5aac5b734c399011
runtime_status: InvocationBudgetExhausted
invocations: 1
operations: 0
false_completion_node_satisfied: false
```

Invariant: ProviderResult exists, but no execution-scoped finalized Effect or
Receipt exists, so no NodeSatisfied fact is emitted.

## P15-08 — existing Review park/resume

- evidence_id: P15-REVIEW-01
- run_id: w15-prepublish-20260901-review-11914
- execution_order: 01–05
- pre-state: workflow deterministic proposal governed by REQUIRE_REVIEW
- Tenant: `tenant:wave15-review`
- Case: `case:wave15-review`
- WorkflowDefinition: `workflow-definition:5f7f1c041ea039b62e1a40abc2058355`
- WorkflowBinding: `case-workflow-binding:76d815711807927ae8b0f6252b442c02`
- node/execution: `reviewed-write` / `workflow-execution:11424193f1de7c58`
- exact command: `YAI_EXECUTION_EVIDENCE=1 tests/characterization/workflow-kernel/test_workflow_review.sh`
- exit: 0

```text
state: WaitingReview
worker_id: none
stop_reason: awaiting_review: workflow deterministic operation awaiting existing Review
review_action: committed
review_id: review:c5eaa016139ed0ee909c4833db2cafff
authenticated_principal_id: principal:72cc156b82060120eac8f7e234dbfcef
action: approve
execution_grant: none_review_command_never_executes
external_effect: none
completed: true
state: Completed
attempt_count: 2
worker_released_while_waiting_review: true
provider_invocations: 0
review_owner_reused: yai.review
```

## P15-09 — resource busy parks and safely retries

- evidence_id: P15-RESOURCE-BUSY-01
- run_id: w15-prepublish-20260901-resource-busy-59013
- execution_order: 01–06
- pre-state: holder Effect PREPARED at resource epoch 1
- Tenant: `tenant:wave15-busy`
- Case: `case:wave15-workflow-busy`
- WorkflowDefinition: `workflow-definition:c943270b0abdf77d2a75740ec105fc2f`
- WorkflowBinding: `case-workflow-binding:d1f7102e345cc2b4c2052360919734ed`
- node/execution: `write-after-release` / `workflow-execution:141a3c940415757e`
- exact command: `YAI_EXECUTION_EVIDENCE=1 tests/characterization/workflow-kernel/test_workflow_resource_busy.sh`
- exit: 0

```text
decision: allow
effect_id: effect:49733fd8879b7817dc5854b359777ca4
resource_epoch: 1
effect_state: prepared_durable_before_mutation
controlled_effect_crash_injected: after_prepare_before_effect
state: Blocked
worker_id: none
stop_reason: worker_error: resource_temporarily_owned
reconciliation: EffectObserved
effect_state: Some(Finalized)
completed: true
state: Completed
attempt_count: 2
blocked_work_posture: Blocked
blocked_decision_posture: allow
retry_trigger: terminal_resource_release
same_workflow_execution_completed: true
provider_invocations_for_workflow: 0
```

Invariant: physical admission does not rewrite ALLOW as DENY or lose semantic
work identity; no busy worker or polling loop remains.

## P15-10 — crash after ProviderResult, before NodeSatisfied

- evidence_id: P15-PROVIDER-ACK-CRASH-01
- run_id: w15-prepublish-20260901-modelwork-49968
- execution_order: 10–15
- pre-state: exact workflow WorkItem running; one provider request expected
- Tenant: `tenant:workflow-model`
- Case: `case:new12-filesystem`
- WorkflowDefinition: `workflow-definition:6810091aa6ca3a068f34cc2f95693bfd`
- WorkflowBinding: `case-workflow-binding:2a4a24402fc312c6df0061cecc3f959c`
- node/execution: `analyze` / `workflow-execution:dd79f7f2d870f6b5`
- exact command: same ModelWork script; runtime includes `--workflow-work-failpoint runtime_after_provider_result`
- first exit: 91; recovery exit: 0

```text
case_runtime_crash_injected: runtime_after_provider_result
exit: 91
completed: false
node: analyze kind=model_work posture=Active reason=completion_proven_pending_canonical_satisfaction execution=workflow-execution:dd79f7f2d870f6b5
completed: true
node: analyze kind=model_work posture=Satisfied reason=canonical_satisfaction_recorded execution=workflow-execution:dd79f7f2d870f6b5
runtime_status: Completed
stop_detail: canonical workflow satisfaction recovered without Case re-execution
invocations: 1
operations: 0
runtime_admission_status: none
state: Completed
stop_reason: canonical_workflow_satisfaction_recovered
provider_result_recovery_duplicate_calls: 0
```

## Preserved pre-fix failure evidence

The first recovery implementation produced the following mismatch after the
same ProviderResult crash: semantic workflow completion was repaired, but its
operational checkpoint was not.

```text
completed: true
satisfied: 1
runtime_status: InvocationBudgetExhausted
invocations: 1
operations: 0
```

After the fix, the unchanged failpoint run is the P15-10 transcript above:
workflow, WorkItem, checkpoint and Case admission converge without a second
provider call.

The first iterative fixture run also preserved a correct fail-closed result:

```text
decision: deny
decision_reason: no_applicable_allow_rule
external_effect: none
runtime_status: ProviderFailureBudgetExhausted
invocations: 2
operations: 0
```

Cause was test configuration (the fixture omitted an ALLOW rule), not an
authority bypass. The unchanged product script passes after the fixture policy
is configured to allow the controlled paths.

## Qualification evidence

- run_id: w15-prepublish-20260901-qualification
- cwd: `/home/mothx/computer-science/projects/YAI/yai`

```text
$ make check
test result: ok. 138 passed; 0 failed
test result: ok. 11 passed; 0 failed
case_runtime:agentless_26_turn_provider_model_replacement ok
human_review:crash_r1_r6_recovery ok
exit: 0

$ make characterization
provider_model_vertical:real_http_invocation ok
controlled_effect:prepare_crash_reconciliation ok
case_runtime:agentless_26_turn_provider_model_replacement ok
human_review:crash_r1_r6_recovery ok
exit: 0

$ make smoke-policy-authority-hardening smoke-temporal-governance smoke-tenant-security smoke-multi-case-runtime smoke-multi-case-runtime-hardening smoke-shared-resource-fencing smoke-shared-resource-fencing-hardening smoke-second-carrier smoke-workflow-kernel
policy_authority_hardening:canonical_write_rederivation ok
temporal_governance_characterization: pass
tenant_security_characterization: pass
multi_case_runtime_characterization: pass
h13_hardening_characterization: pass
shared_resource_fencing_characterization: pass
h14_multiprocess_contention: pass
second_carrier_characterization: pass
workflow_kernel_characterization: pass
workflow_modelwork_characterization: pass
workflow_resource_busy_characterization: pass
workflow_review_characterization: pass
exit: 0
```

Formatting, docs/layout, TSV shape and `git diff --check` passed. Clippy exited
0 under the repository contract and reported only its pre-existing warning
baseline.

## External YVEX

- state: `blocked_external_dependency`
- exact reason: neither `YAI_EXTERNAL_PROVIDER_BASE_URL` nor
  `YAI_EXTERNAL_PROVIDER_MODEL` was supplied.
- action taken: none; YVEX remained an uninspected, unadministered black box.
