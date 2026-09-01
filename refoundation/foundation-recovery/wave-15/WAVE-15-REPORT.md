# Wave 15 — Case-Bound Workflow Kernel

State: implementation and local qualification complete; publication pending.

- Baseline actually found: `208c055e4b6f2747136a0a4d21b5e654db64786a`.
- Prompt baseline: `3adb76d090501f12d0aad0f370152a107d01ee12`; the delta is the separately requested and already published root `./yai` launcher.
- Intended commit: `feat: add case-bound workflow kernel`.
- Historical dirty work: all 13 entries preserved and excluded from the Wave whitelist; tracked checksum remains `3fdb219654405e6fd40b5c0d1b02b94c04fadef5aa57a139aa5fb8fd6db7777e`.
- YVEX external state: `blocked_external_dependency`; neither an endpoint nor an exact provider-exposed model was supplied.

## Direct archaeology

Wave 15 re-inspected `yai-dev` source and history directly at
`94ba627091afbf1100ab386ac1de3d4fb1d2502c`,
`001752f52545fe84a30017ec735794a8f04d189c`,
`840aee9464211e7af6d6811de32320faea14806e`, the stronger later
`cffb318b9`, and final topology `68095595327a6ea11024c044ac9205496701a854`.
Inspected surfaces included `core/orchestration/flow/model/flow.c`,
`core/orchestration/substrate/flow_substrate.c`, planning/routing/handoff
headers and CLI flow consumers, later `src/orchestrator/workflow/*`, and
runtime current-work projections.

Recovered properties are explicit flow identity, dependencies, gates,
blockers, evidence references, human interaction distinct from automatic
progress, Case linkage and inspectable current work. Rejected structures are
the flow mega-owner, flow-owned policy/authority/review/effects, scope
filesystem hierarchy, module/mount registries, Agent capability routing and a
global Orchestrator owner. No legacy executable mechanism was stronger than
the refounded store re-derivation and authority isolation.

## Owner and canonical-state verdict

`WorkflowDefinition` earns one Case-independent semantic owner: it is
Tenant-bound, immutable, versioned, reusable and independently inspectable.
Exact adoption and progression facts remain in the existing
Transition/CaseState owner. `WorkflowResolution`, `ReadyWorkSet`, queue state
and RuntimeWorkItem attribution remain derived or operational. There is no
canonical `WorkflowRun` and no Workflow ledger.

The single new engine owner is `engine/yai-engine/src/workflow.rs`. It owns the
bounded DAG definition, binding contract, typed predicate algebra and pure
resolver. It owns no policy, Decision, Grant, Review, ResourceFence, provider,
memory, Case lifecycle or effect truth.

## Implemented contract

- `yai.workflow_definition.v1`: exact Tenant/key/version, content-bound ID,
  immutable semantic bytes, 128 nodes, 512 edges, bounded identifiers, labels,
  tasks, input and deterministic operation content.
- `yai.case_workflow_binding.v1`: zero-or-one exact Definition ID/digest per
  Case, exact executor/resource slots, Tenant equality and Tenant Owner write.
- `yai.transition.v10` / `yai.case_state.v10`: binding, execution start,
  satisfaction, frozen condition, human input and deterministic proposal facts;
  v1–v9 readers remain supported.
- Pure resolver: stable topological rank then node ID, frozen condition branch,
  inactive-branch skipping and conditional joins that ignore only the frozen
  unselected branch.
- Typed predicates: execution ProviderResult, any execution-finalized effect,
  exact execution-finalized filesystem path, human input, Case lifecycle,
  prior node satisfaction, Decision outcome, terminal Review and explicit
  broader finalized-effect goal.
- Node families: `ModelWork`, `DeterministicWork`, `HumanInput`, `Condition`,
  `Wait`, `EffectGoal`; no script/plugin/subflow/handoff node.
- RuntimeInstance bounded pump: passive progression, atomic Ready recheck +
  `WorkflowNodeExecutionStarted` + WorkItem enqueue, existing Tenant-fair queue,
  queue backpressure before semantic start and one active node per Case.
- ModelWork reuses the existing bounded Case runtime. It supports cognitive
  one-turn completion, exact operational completion and a real two-turn
  execution where the second ContextFrame observes the first finalized effect.
- DeterministicWork records a canonical typed proposal and then traverses the
  same DecisionBasis/Decision/Review/Grant/PREPARE/fence/carrier spine with
  zero provider invocations.
- HumanInput re-authenticates Principal/Tenant/Participant role and creates no
  ReviewAction. Condition, Wait and EffectGoal occupy no worker.

## Crash, replay and operational recovery

The failpoint `runtime_after_provider_result` proves a RuntimeInstance may die
after canonical ProviderResult but before NodeSatisfied. Restart first commits
the mechanically proven satisfaction, then converges the exact WorkItem and
Case checkpoint to Completed without Case re-execution. Provider invocation
count remains one, Operation count remains zero, and stale CaseRuntimeAdmission
is released.

Resource admission remains distinct: a valid ALLOW Decision that encounters
`resource_temporarily_owned` parks the same workflow execution as WorkItem
`Blocked`, releases the worker, and resumes after terminal resource release
under fresh authority. Existing Review parks as `WaitingReview`; authenticated
approval resumes the same execution.

Workflow resolution rebuilds from exact Definition + Case binding + Case
Transition history without WorkItems, RuntimeInstance memory or a WorkflowRun
database. Definition v2 does not alter Cases pinned to v1. Two Cases may bind
one immutable Definition while keeping distinct bindings, histories and
resources.

## Discovered failures and fixes

1. A frozen conditional join initially treated the unselected branch as a
   missing dependency. The resolver now derives skipped nodes first and ignores
   only incoming edges from the frozen unselected branch; a focused unit test
   covers the join.
2. The first terminal-ack recovery implementation repaired WorkItem truth but
   left its exact Case checkpoint at `InvocationBudgetExhausted`. Recovery now
   proves canonical node satisfaction in the store, marks the exact WorkItem
   Completed, repairs the matching checkpoint and releases stale Case admission
   without provider re-entry.
3. The first iterative product fixture omitted an ALLOW effect rule, so both
   candidate Operations were correctly denied and no effect occurred. The
   unchanged reproduction passes after fixing fixture policy configuration;
   YAI authority behavior itself was correct.

## Qualification and maturity verdict

`make check`, `make characterization`, all smoke targets from H10 through H14,
`smoke-workflow-kernel`, full 138-test engine and 11-test CLI suites, formatting,
Clippy under the repository warning baseline, docs/layout and
`git diff --check` pass in the pre-publication state. The combined regression
also proves the 26-turn governed Case, review crash R1–R6, authority attacks,
Tenant isolation, bounded multi-Case runtime, worker-panic recovery,
carrier-enforced fencing, eight-process resource contention and process-signal
uncertainty remain intact.

This is a material maturity increase: YAI can now describe durable progression
without making workflow, model or scheduler authoritative. It is not a claim of
full workflow hardening, dynamic planning, remote execution or provider
governance.

## Footprint and non-claims

- New semantic owners: 1 (`WorkflowDefinition` plus its pure resolver contract).
- New LMDB named databases: 1 (`workflow_definitions`), 34 → 35 of 40.
- Canonical WorkflowRun databases: 0.
- `main.rs`: 2009 → 2024 lines; parse/dispatch/help only.
- No Agent, workflow daemon, second scheduler, workflow resource lease,
  provider routing, subflow, handoff or PlanPatch.

H15 remains responsible for adversarial same-node multi-process resolution,
definition corruption/missing-history pressure, larger predicate/join matrices,
replay divergence and 128-node scale. Wave 16 remains responsible for typed
PlanPatch/amendment, subflows and same-Tenant handoff/reconciliation.

## YVEX EXTERNAL FINDINGS

Live qualification was not executed: no black-box endpoint and exact
provider-exposed model identity were supplied through
`YAI_EXTERNAL_PROVIDER_BASE_URL` and `YAI_EXTERNAL_PROVIDER_MODEL`. No YVEX
repository, CLI, profile, artifact, session or engine was inspected or
administered. There are no new YVEX-side findings in this Wave.
