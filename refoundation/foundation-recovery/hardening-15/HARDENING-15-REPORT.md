# Foundation Hardening 15 Report

Status: implementation and local qualification complete; publication pending.

Baseline: `e8b9974b6e4cee3996dba346c1896fa17fd38ed2` on `master`.
Intended semantic commit: `harden: close workflow progression semantics`.
This versioned report deliberately does not predict the SHA of the commit that
contains it.

## Verdict

H15 closes the Workflow Kernel at its existing owners. No WorkflowRun,
progression manager, lock owner, scheduler, Agent or database was added.
`WorkflowDefinition + CaseWorkflowBinding + ordered Case Transitions` remains
sufficient for exact semantic resolution, provided the immutable bound
Definition is retained.

The hardening adds mechanical predicate scopes and node compatibility,
combined structural/semantic dependency analysis, strict replay snapshot
validation, bounded aggregate Definition size, strict unknown-field parsing,
canonical-presence validation for deterministic proposal recovery, and
one-shot crash recovery at deterministic and ModelWork boundaries.

## Direct archaeology

Fresh source/history inspection covered `94ba627091afbf1100ab386ac1de3d4fb1d2502c`,
`001752f52545fe84a30017ec735794a8f04d189c`,
`840aee9464211e7af6d6811de32320faea14806e`, and full object
`cffb318b980456f2671a297e14a6b05f5ac68320`, plus adjacent later history.
Inspected paths include `core/orchestration/flow/model/flow.c`,
`core/orchestration/flow/*`, `core/orchestration/planning/*`,
`core/orchestration/routing/*`, `core/orchestration/handoff/*`,
`include/orchestration/planning.h`, `include/yai/orchestrator/workflow/workflow.h`,
`src/orchestrator/workflow/flow.c`, CLI flow surfaces and current-work/
interaction records.

Legacy's strongest useful properties were explicit dependencies, blockers,
progress/gate records, interaction separation, evidence references and an
inspectable current-work projection. Its stronger-looking later topology still
used mutable controller directories, `*_current.json`, append files and
second-resolution identifiers across separate writes. Direct inspection found
no stronger concurrent same-node commit, immutable exact Definition retention,
semantic dependency-cycle analysis, deterministic Case-history replay or
crash-safe progression acknowledgement. H15 therefore hardens beyond legacy.

## Predicate and dependency closure

Predicates are mechanically scoped as `Execution`, `Node`, `Case` or
`Progression`. Execution predicates are admitted only on executable node
completion. `FinalizedEffect` remains an intentionally broad Case goal for
EffectGoal, while execution completion remains causally bound to its exact
Workflow execution. `HumanInputRecorded` is not admitted as an arbitrary
predicate; HumanInput has its dedicated canonical record.

Definition validation builds one dependency relation from explicit edges plus
`NodeSatisfied(B) -> consumer` semantic edges. The same deterministic
topological validator rejects simple and multi-node hidden cycles before
persistence. A valid reference to an already-prior node is accepted.

## Concurrency and recovery

Eight independent OS processes compete through RuntimeInstance admission; one
exact process owns materialization, producing one `WorkflowNodeExecutionStarted`
and one WorkItem. Eight independent LMDB handles racing the same write likewise
produce one start. Condition/passive progression is idempotent. Concurrent
HumanInput uses first canonical value wins; a conflicting later value is
rejected without overwrite.

Definition publication is idempotent for exact content and rejects concurrent
same-lineage/version content divergence. Missing or corrupt bound Definitions
fail closed. Restoring the exact retained bytes restores the exact prior
resolution. Node/edge array order is intentionally identity-bearing in v1;
JSON object-key order is normalized by typed parsing.

ModelWork recovery distinguishes a canonical completed turn from the next
required turn. A crash after an unsatisfying ProviderResult does not re-invoke
that completed turn; the next turn continues within the persisted budget. Two
identical output byte strings remain distinct invocation/result identities and
cannot satisfy an effect predicate. Crash after a satisfying result repairs
NodeSatisfied without transport repetition. A request sent but not canonically
recorded remains generic remote-delivery uncertainty; H15 does not claim
exactly-once remote inference.

Deterministic proposal and Operation creation are separate idempotent seams.
Recovery requires exact canonical proposal presence and exact WorkItem/
binding/execution fields. A crash after proposal reuses it, creates one
Operation, passes the normal Decision/Grant/Fence/carrier path, and yields one
Effect with zero provider invocations.

## Replay and scale

The pure resolver rejects malformed snapshots where history count, sequence,
Case identity or CaseState generation diverges. Frozen Condition,
NodeSatisfied and HumanInput facts dominate later resolver behavior. Status is
read-only. Operational WorkItems/checkpoints are not inputs to semantic
resolution.

The admitted maximum 128-node Definition and a 512-edge DAG validate and
resolve deterministically in focused tests. A product 128-node passive chain
encoded as 31,704 bytes progressed to Case generation 130 in 2,798 ms in the
retained run, created zero WorkItems, invoked zero providers, caused zero
physical effects, and returned identical status after RuntimeInstance stop.
A 2 MiB aggregate Definition bound prevents the former independently bounded
node payloads from producing an 8,414,131-byte admitted object.

## Ownership and footprint

- semantic owners added: 0;
- canonical WorkflowRun stores: 0;
- LMDB named databases: 35 of 40, unchanged;
- Agent runtime owners: 0;
- schedulers/worker pools added: 0;
- serialized schemas: unchanged at WorkflowDefinition v1, binding v1,
  Transition/CaseState v10;
- `main.rs`: unchanged by H15.

## Forward boundary

Forward planning now reserves Wave 16 for CLI Product Refoundation, Wave 17
for adaptive Workflow/PlanPatch/subflow/same-Tenant handoff, and Wave 18 for
provider governance. H15 implements none of those boundaries.

## YVEX external qualification

`YAI_EXTERNAL_PROVIDER_BASE_URL` and `YAI_EXTERNAL_PROVIDER_MODEL` were not
supplied. Live black-box qualification is therefore
`blocked_external_dependency`; no YVEX CLI, repository, server administration,
profile, artifact or engine identity was inspected or modified.
