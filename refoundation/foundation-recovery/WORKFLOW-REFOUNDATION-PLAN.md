# Workflow Refoundation Plan

Status: the bounded Wave-15 kernel, H15 adversarial closure and Wave-16 CLI
product boundary are published. Wave 17 implements the current adaptive
composition boundary. Wave 18 remains forward design, not a current runtime
claim.

## Constitutional ownership

```text
Case             owns semantic truth and durable continuity
Workflow         describes progression
RuntimeInstance  schedules bounded work
Model            supplies bounded cognition
Policy           supplies normative constraints
Decision/Grant   supplies authority
Carrier          changes external reality
```

Agent is not restored as a runtime owner. A Case remains legal without a
Workflow.

`CaseBlueprint`, `WorkflowDefinition` and `Case` are distinct. The implemented
WorkflowDefinition is immutable, versioned, Tenant-bound and adopted by exactly
one Case through an explicit identity-bearing relationship. Its initial node
kinds are `ModelWork`, `DeterministicWork`, `HumanInput`, `Condition`, `Wait`
and `EffectGoal`.

`ModelWork` means bounded adaptive Case execution until a canonical completion
predicate is satisfied. A model statement that work is complete is provider
candidate material, not proof that the node predicate is satisfied.

`WorkflowRun` remains a derived/materialized projection. It earns
independent canonical ownership only if a future owner test proves a lifecycle
that cannot be reconstructed from WorkflowDefinition, Case Transitions and
current resolution rules.

## Resource-authority constraint

None of these may own a ResourceLease or fencing epoch:

- WorkflowDefinition;
- WorkflowRun;
- WorkflowNode;
- RuntimeWorkItem.

A ready node may create or submit work. Physical authority remains:

```text
ready work
  → Case execution
  → Operation
  → Decision
  → ExecutionGrant
  → PREPARE-time resource acquisition/fence
  → carrier validation
  → physical effect
```

`resource_temporarily_owned` is not policy DENY. A future resolver may keep a
node non-runnable/retryable without altering its policy Decision.

## Same-Tenant multi-Case handoff

The first admissible multi-Case design has no shared mutable workflow state:

```text
Case A: HandoffOffered
  → Case B: HandoffAccepted
  → Case B local governed work
  → Case B: HandoffResult
  → Case A: accepted or reconciled
```

Every Case retains its own history and authority. Initial scope is same-Tenant;
cross-Tenant federation is not authorized. Source Workflow progression uses a
source-local `HandoffReconciled` fact and never reads target history as live
Workflow truth.

## Case-local adaptation

The implemented composition is:

```text
immutable root WorkflowDefinition
  + exact CaseWorkflowBinding
  + ordered WorkflowAmendment facts in the Case Transition history
  + exact immutable child WorkflowDefinitions
  -> derived EffectiveWorkflowTopology
  -> existing deterministic resolver and RuntimeInstance
```

PlanPatch is bounded candidate material. Human or model proposal is distinct
from Tenant-Owner adoption; the current topology digest provides optimistic
concurrency and there is no rebase or merge. Frozen progression cannot be
rewritten. Subflow expands an exact same-Tenant Definition inside the same
Case using qualified instance paths; it creates no child Case, worker pool or
authority owner.

## Planned delivery sequence

- Wave 15: complete — immutable WorkflowDefinition,
  exact Case binding, deterministic resolver, completion predicates,
  ReadyWorkSet and the six initial node families.
- H15: complete — predicate type safety, concurrent progression, definition
  integrity/retention, deterministic replay, crash recovery and authority
  isolation.
- Wave 16: complete — CLI Product Refoundation: porcelain/plumbing separation,
  canonical command registry, parser lanes and output-family alignment.
- Wave 17: current — typed PlanPatch, Case-local immutable amendment lineage,
  bounded exact Subflow composition and same-Tenant handoff/reconciliation.
- Wave 18: provider governance and optional provider-native qualification,
  without making provider capability the source of Workflow or Case authority.

Wave 17 does not make the CLI, PlanPatch, EffectiveTopology, Subflow or a
multi-Case process into new authority owners. The Wave 18 reservation does not
authorize provider selection, health or failover semantics in this Workflow
kernel.
