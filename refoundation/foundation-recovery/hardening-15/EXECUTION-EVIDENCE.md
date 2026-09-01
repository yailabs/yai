# H15 Execution Evidence

This file retains bounded, unedited excerpts from actual runs. The report is
created before publication; a post-semantic-commit evidence pass will append
the exact published semantic SHA rather than reconstructing output.

## H15-LOCAL-20260901-01 — focused adversarial engine suite

- execution order: 1
- pre-state: baseline `e8b9974b...` plus H15 worktree implementation
- cwd: `/home/mothx/computer-science/projects/YAI/yai/engine`
- environment: local test LMDBs under `/tmp`; no external provider variables
- command: `cargo test --lib h15_ -- --nocapture`
- exit: 0

```text
h15_deterministic_recovery: proposal_id=workflow-proposal:55b2cbeeee4c1a0f operation_id=operation:ccccba32db6072749fef5fdd81ff12bb proposal_count=1 operation_count=1 provider_invocations=0
h15_definition_integrity: concurrent_exact_publishers=8 stored=1 version_collision_winners=1 missing=fail_closed corrupt=fail_closed exact_restore=equal
h15_process_same_node_start: processes=8 canonical_starts=1 work_items=1 provider_invocations=0
h15_same_node_start: contenders=8 canonical_starts=1 work_items=1 unique_work_ids=1 status_writes=0
h15_human_condition_race: contenders=8 accepted_inputs=1 conflicting_inputs_rejected=7 condition_results=1 review_actions=0
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 138 filtered out
```

Invariant: independent-process ownership, same-node writes, deterministic
recovery, Definition publication and passive/human progression converge to one
canonical fact.

## H15-LOCAL-20260901-02 — ModelWork crash/repetition product run

- execution order: 2
- pre-state: isolated empty YAI homes and fixture HTTP providers
- cwd: `/home/mothx/computer-science/projects/YAI/yai`
- environment: `YAI_EXECUTION_EVIDENCE=1`; fixture providers only; no secrets
- command: `bash tests/characterization/workflow-kernel/test_workflow_modelwork.sh`
- exit: 0

```text
case_runtime_crash_injected: runtime_after_provider_result
exit: 91
workflow_modelwork_characterization: pass
modelwork_provider_invocations: 1
iterative_modelwork_provider_invocations: 2
iterative_modelwork_operations: 2
second_turn_observed_first_effect: true
unsatisfying_provider_result_crash_exit: 91
unsatisfying_provider_result_recovery_invocations: 2
unsatisfying_provider_result_duplicate_prior_turns: 0
repeated_identical_output_invocations: 2
repeated_identical_output_distinct_results: 2
repeated_identical_output_node_satisfied: false
false_completion_provider_invocations: 1
false_completion_node_satisfied: false
provider_result_recovery_duplicate_calls: 0
```

Invariant: a canonical unsatisfying result is not reinvoked; recovery advances
the next bounded turn. Identical text is not invocation identity or completion.

## H15-LOCAL-20260901-03 — deterministic crash and maximum scale

- execution order: 3
- pre-state: isolated empty YAI home; governed local filesystem fixture
- cwd: `/home/mothx/computer-science/projects/YAI/yai`
- environment: `YAI_EXECUTION_EVIDENCE=1`; no provider endpoint
- command: `bash tests/characterization/workflow-kernel-hardening/test_workflow_kernel_hardening.sh`
- exit: 0
- produced IDs: Definition `workflow-definition:f2101d881787f1d0e30cc6c714597932`,
  binding `case-workflow-binding:b952842323af0c3d4ef703e4f00ed345`,
  execution `workflow-execution:267769e0ebdff423`, proposal
  `workflow-proposal:01b690a809f9ad95`, Operation
  `operation:2a4cf5aa0c169054dbc7ab1e4fb962fc`, Effect
  `effect:7d5aaff08837f46d705f29f9472d422b`.

```text
workflow_cycle_rejected
exit: 2
workflow_deterministic_proposal_id: workflow-proposal:01b690a809f9ad95
controlled_effect_crash_injected: after_workflow_deterministic_proposal
exit: 90
instance_admission: reclaimed_stale
recovered_items: 1
operation_id: operation:2a4cf5aa0c169054dbc7ab1e4fb962fc
effect_id: effect:7d5aaff08837f46d705f29f9472d422b
effect_state: finalized
workflow_kernel_hardening_characterization: pass
deterministic_provider_invocations: 0
deterministic_operations: 1
deterministic_work_items: 1
scale_nodes: 128
scale_edges: 127
scale_definition_bytes: 31704
scale_case_generation: 130
scale_progression_ms: 2798
scale_work_items_created: 0
scale_provider_invocations: 0
scale_physical_effects: 0
scale_replay_equal: true
```

Invariant: canonical proposal survives process death; same WorkItem resumes to
one governed Effect. Maximum passive progression is bounded and model-free.
