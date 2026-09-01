# Workflow Refoundation Plan

Status: the bounded Wave-15 kernel and H15 adversarial closure are published.
Wave 16 is the current CLI product refoundation. Waves 17 and 18 remain forward
design boundaries, not current runtime claims.

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
cross-Tenant federation is not authorized.

## Planned delivery sequence

- Wave 15: complete — immutable WorkflowDefinition,
  exact Case binding, deterministic resolver, completion predicates,
  ReadyWorkSet and the six initial node families.
- H15: complete — predicate type safety, concurrent progression, definition
  integrity/retention, deterministic replay, crash recovery and authority
  isolation.
- Wave 16: current — CLI Product Refoundation: porcelain/plumbing separation, canonical
  command registry, parser lanes and output-family alignment.
- Wave 17: typed PlanPatch, bounded subflows, adaptive evolution and
  same-Tenant handoff/reconciliation.
- Wave 18: provider governance and optional provider-native qualification,
  without making provider capability the source of Workflow or Case authority.

The Wave 16 reservation does not make CLI parsing or output a Workflow owner.
The Wave 17 reservation does not authorize mutable definitions, subflows or
handoffs in H15. The Wave 18 reservation does not authorize provider selection,
health or failover semantics in the Workflow kernel.
