# Wave 17 — Adaptive Workflow Composition

State: implementation and full local qualification complete; publication
pending.

- Baseline: `ab9dace4250bfd2e259468569917d74791b640a5` on `master`, equal to
  `origin/master` and the remote branch before implementation. No later W16
  correction or unexplained semantic commit existed.
- Intended commit: `feat: add adaptive workflow composition`.
- Historical dirty work: all 13 entries are preserved and excluded from the
  Wave whitelist; their expected tracked checksum is
  `3fdb219654405e6fd40b5c0d1b02b94c04fadef5aa57a139aa5fb8fd6db7777e`.
- External provider qualification: `blocked_external_dependency`; neither
  required provider environment variable was supplied.

## Archaeology and owner verdicts

Fresh direct inspection covered `yai-dev` commits
`94ba627091afbf1100ab386ac1de3d4fb1d2502c`,
`001752f52545fe84a30017ec735794a8f04d189c`,
`840aee9464211e7af6d6811de32320faea14806e` and
`cffb318b980456f2671a297e14a6b05f5ac68320`, including `flow.c`,
`flow_substrate.c`, flow composition/topology/state, planning, routing,
handoff, recovery, CLI and headers. Useful recovered properties are explicit
source/target identity, request/result separation, dependency/gate
progression, parent/child composition intent, bounded current-work projection
and explicit reconciliation intent.

The executable legacy mechanisms did not provide immutable Case-local
amendment lineage, exact concurrent one-winner adoption, qualified duplicate
Subflow instances, same-Tenant authority isolation or source-local handoff
reconciliation. Mutable flow-controller directories, current JSON files,
Agent/provider routing, flow-owned policy/review/authority and the global
Orchestrator remain rejected.

Owner tests remain negative: PlanPatch is candidate material; Amendment is a
Case Transition; EffectiveWorkflowTopology is derived; Subflow is same-Case
definition composition; Handoff truth is split across source/target Case
Transitions; the multi-Case process view is reconstructible. No WorkflowRun,
Process, Agent, manager or scheduler owner is added.

## Schemas and compatibility

Wave 17 introduces `yai.workflow_definition.v2`,
`yai.case_workflow_binding.v2`, `yai.workflow_plan_patch.v1`,
`yai.workflow_amendment.v1`, `yai.effective_workflow_topology.v1`,
`yai.workflow_resolution.v2`, and the v1 Handoff offer/acceptance/decline/
result/reconciliation family. New canonical payloads advance Transition and
CaseState to v11. Readers for v1-v10 and static Workflow Definition/Binding v1
remain supported; v1 meanings are unchanged.

## PlanPatch and amendment

A patch is content-bound to Tenant, Case, exact binding/base Definition,
effective-topology digest, parent amendment/revision, origin and operations.
Origins are authenticated human or one exact Workflow ModelWork
ProviderResult. The strict model output contract carries schema, base topology
digest and operation bound into the generic provider frame. Malformed output
remains a ProviderResult and creates no patch. Reprocessing the same exact
result returns the same canonical patch transition.

The v1 algebra is `AddNode`, `AddEdge`, `DisableNode` and `DisableEdge`; it has
no edit/rename/replace operation. Limits are 256 KiB, 32 operations, 16 added
nodes, 64 added edges and 32 adopted amendments. Tenant Owner adoption occurs
only while the workflow is quiescent, not completed and the Case is open and
uncancelled. It re-derives the current topology and all H15 validation in one
transaction. A stale digest refuses without rebase. Frozen node/execution/
condition/input/proposal/satisfaction history cannot be disabled or have its
justifying dependencies reinterpreted. Eight same-base adoption contenders
produce one amendment and seven stale refusals.

## Effective topology and Subflow

Pure derivation loads the immutable root, applies ordered Case amendments,
resolves exact child Definition ID/digest references, expands qualified
instance paths, and then validates the complete graph before resolution.
The digest and revision are deterministic. Limits are 512 effective nodes,
2,048 edges, nesting depth 4 and 32 referenced Definitions.

Subflow has no WorkItem or child Case. Child executable nodes enter the same
ReadyWorkSet and RuntimeInstance; passive containers complete mechanically
after their exact child instance. `root/a/x` and `root/b/x` remain distinct
when the same child Definition is used twice. Executor, Resource and Case slot
mappings are explicit; recursion and cross-Tenant composition fail closed.
Replay requires the exact root and every exact child Definition.

## Handoff protocol

The FSM is Offer in source Case → Accept or Decline in target Case → optional
ordinary target-local work → one Result in target Case → Reconcile in source
Case. Acceptance requires the addressed same-Tenant Case, an authenticated
Principal linked to the exact Participant and all requested target roles.
Request/result payloads are strict bounded text or JSON (16 KiB); evidence is
bounded to 32 refs and roles to 16.

Offer alone, acceptance and target result never satisfy a source Workflow
node. Only source-local `HandoffReconciled` truth can do so. Reconciliation
stores bounded material and exact target references, not target Decision,
Grant, Review, EffectReceipt, resources, provider or history. Source
cancellation before acceptance blocks acceptance; after acceptance it cannot
erase target truth, while source-side reconciliation remains permitted for
audit without resuming cancelled progression. Target cancellation/closure is
reconciled as a terminal non-success posture. Active Case-level wait cycles,
self handoff and cross-Tenant handoff fail closed. Waiting holds zero workers
and zero ResourceFences.

## Authority and runtime isolation

Topology adoption is not operational authority. Added DeterministicWork still
traverses Operation → Decision → Review where required → Grant → ResourceFence
→ Carrier. Patch/Handoff values cannot encode or import ALLOW, Grant, reviewer
authority, provider credentials or resource leases. RuntimeInstance remains
the only bounded scheduling owner and one active executable node per Case is
preserved.

## CLI and footprint

The W16 registry gained six `workflow patch` and seven `case handoff`
operations, plus repeatable exact `workflow bind --case-slot SLOT=CASE`.
Executable discovery reports 134 canonical operations (63 Product) and
registry digest
`sha256:c219161abca72268008b9326d4e43a050d8001cde95b0cbe8d6f35c32ebf85a1`.
All new Product operations use the existing centralized parser, operation IDs,
lanes, human/JSON boundary and NO_COLOR contract. `main.rs` remains 12 lines.

The semantic owner count is unchanged. LMDB remains 35/40 named databases;
there is no amendment, EffectiveTopology, Handoff, WorkflowRun or Process DB.
One cohesive `handoff.rs` implements a protocol, not an independent truth
owner.

## Foundation Recovery reclassification

- adaptive Workflow / PlanPatch: `refounded_proven` candidate;
- Workflow amendment: `Case-canonical`, no independent owner;
- EffectiveWorkflowTopology: `derived_no_owner`;
- Subflow: `refounded_proven` candidate for same-Case exact composition;
- Handoff: `refounded_proven` candidate for same-Tenant Case protocol;
- multi-Case Process and WorkflowRun: `derived_no_owner`.

`make check`, elevated local-socket `make characterization`, the 166-test
engine suite, 22-test CLI suite, W16 registry audit, all current lower-wave
smokes, the new adaptive Workflow smoke, formatting, Clippy, documentation and
diff hygiene pass. The candidate labels therefore close to `refounded_proven`
locally; remote equality remains the publication gate.

## YVEX EXTERNAL FINDINGS

`yvex_external_qualification_state=blocked_external_dependency`. The operator
supplied neither `YAI_EXTERNAL_PROVIDER_BASE_URL` nor
`YAI_EXTERNAL_PROVIDER_MODEL`; therefore no live ModelWork PlanPatch request or
malformed-output pressure test was executed. No YVEX source or administrative
surface was inspected, no provider-specific branch was added, and no new YVEX
finding is claimed.

## Remaining boundaries

H17, not implemented, owns longer amendment chains, more hostile concurrent
adoption/reconciliation, nested crash edges, cancellation race pressure,
forged result/cross-definition replay and derived graph/index rebuild attacks.
Wave 18, not implemented, remains provider qualification, capabilities, trust,
health, selection, justified failover and optional YVEX-native extension.
