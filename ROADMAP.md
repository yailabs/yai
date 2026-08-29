# YAI implementation refoundation roadmap

Authority: implementation delta between [the constitution](docs/constitution.md)
and [the current executable architecture](docs/architecture.md). A named
concept or stage does not automatically justify a source subsystem.

## Objective

Preserve canonical transition/effect/context/memory/review authority while
giving Case lifecycle, cancellation and bounded authority expiry executable
meaning. Human review and single-host runtime admission now exist; the next
change must make stale work and Case closure fail closed without introducing a
Workflow, scheduler, or Agent owner.

## Completed boundary — executable reality convergence

`YAI.SOURCE.REFOUNDATION.1` characterized the current product verticals before
collapsing physical ownership. It:

- separated the 16-source production C archive from component-only C tests;
- removed the marker Rust FFI crate, smoke-only bridge, net/proto scaffold,
  synthetic carrier/registry families, inspection-only Case/lease views, and
  duplicate C graph/index/memory/reconcile implementations;
- retained and characterized C filesystem/process/observation mechanics with
  unique platform value;
- extracted provider, review, direct filesystem, replay, graph, and analytics
  behavior from the former 10,901-line `main.rs`;
- added real HTTP provider and direct filesystem bypass characterization;
- preserved the five demonstrated product verticals and `yaid` lifecycle;
- left JSONL/LMDB authority, Record schemas, and effect semantics unchanged.

The removed source protected no uncharacterized product behavior. Historical
E05/E07/V11 properties not implemented today are retained as regression
requirements in the source-refoundation evidence package, not restored as old
runtime directories.

## Completed boundary — typed transition authority

`YAI.SOURCE.REFOUNDATION.2` selected the existing LMDB environment after
testing it against the required transaction semantics. It implemented:

- `yai.transition.v1`, with global identity, Case identity, per-Case sequence,
  source, optional Scope, causal refs, typed payload, provenance, and
  presentation-only summary;
- `yai.case_state.v1`, atomically reduced with every ledger append and fully
  rebuildable from ordered Transitions;
- deterministic duplicate/stale-generation rejection, rollback-before-commit,
  restart, replay equivalence, materialization rebuild, persisted-version
  rejection, and derived failure isolation;
- typed Case/participant/provider/invocation/result/interpretation and fixed
  review request/resolution payloads as the minimum current consumer set;
- typed graph derivation and deterministic graph replacement from canonical
  transitions, with historical records routed through one compatibility
  decoder;
- a corpus covering all 35 Rust and 32 C legacy kinds, drift cases, both old
  schemas, optional/malformed/unknown input, repeated IDs, and old summary
  variants;
- inspect, dry-run, and isolated compatibility import that preserves unknown
  information opaquely and never creates canonical meaning;
- live provider and review consumers while retaining old JSONL/record output
  for compatibility.

LMDB is now physical storage for both canonical ledger and current
materialization, but only the Transition ledger is historical authority. The
old journal/record databases remain compatibility input/output and operator
evidence, not a second mutable canon.

## Completed boundary — first constitutional effect vertical

`YAI.SOURCE.REFOUNDATION.3` implemented one narrow end-to-end Case path:

```text
qualified CaseState
→ bounded controlled-effect projection
→ provider Invocation + typed ProviderResult
→ interpreted OperationCandidate
→ Decision + ExecutionGrant
→ PREPARE / filesystem carrier / FINALIZE
→ committed Transition + materialized CaseState
→ rebuilt next Projection
```

The provider result remains raw candidate material until an exact
`filesystem.write` proposal contract is normalized. ALLOW/DENY is deterministic
and owned by a bound non-model policy participant. Only ALLOW produces an
integrity-bound, generation-bound, one-time Grant. The Rust carrier requires a
materialized durable PREPARE, performs real pre/post observation and atomic
replacement, and finalizes only established outcomes.

Crash injection now covers after Grant/before PREPARE, after PREPARE/before
effect, after visible effect/before observation, and after receipt construction/
before FINALIZE. Explicit restart reconciliation concludes effect observed, no
effect observed, conflict, or still indeterminate from real state. Duplicate
reconciliation does not duplicate the semantic effect. The direct Rust write
command is removed, review approval uses the same carrier, and the C daemon
filesystem fixture no longer performs or claims an effect.

The deterministic HTTP vertical performs a required second provider call. Its
view derives success or denial from typed Transition/CaseState consequence,
never from the first model's assertion. This proves the first constitutional
effect path, not a generalized carrier or final context architecture.

## Completed boundary — typed Projection and semantic continuity

`YAI.SOURCE.REFOUNDATION.4` implemented one provider-independent semantic
compiler used by ordinary prompts and controlled effects:

- `yai.projection.v1` binds Case generation, admitted participant/purpose,
  ordered typed entries, authority posture, provenance and bounded omissions;
- `yai.context_frame.v1` gives one invocation task/output contract identity
  independent of provider render, token sequence and continuation;
- `yai.rendered_input.v1` metadata identifies provider/model render and digest
  without retaining full rendered input;
- `yai.transition.v3` records provider/model/Projection/Frame/render/output-
  contract lineage and typed bounded interaction turns;
- new provider invocations no longer write or consume ParticipantViewFrame;
- optional opaque continuation is ephemeral and invalidation retries the full
  frame; its loss cannot change Case history or CaseState;
- deterministic HTTP proofs replace provider and model after a real filesystem
  effect, restart provider state, and preserve the observed consequence;
- participant visibility fails before rendering, indeterminate effects remain
  unresolved, false provider claims remain claims, and long history produces a
  bounded frame;
- the droppable semantic-context artifact store supports inspection but owns no
  canonical data.

Residency remains provisional and ContextDelta deferred because neither has an
independent current consumer. The context-residency lab remains research
evidence, not implementation authority.

## Completed boundary — provenance-bound operational memory and qualified retrieval

`YAI.SOURCE.REFOUNDATION.5` implemented one rebuildable experience layer:

- `yai.operational_memory.v1` derives resource effects, Decisions, unresolved
  effects, normalization failures and explicitly non-authoritative provider
  claims from canonical Transitions;
- every entry binds deterministic identity, derivation version, generation,
  typed value, participant visibility and Transition/Observation/Receipt/
  causal provenance;
- active/superseded lifecycle prevents stale resource state or unresolved
  effect residue from outranking a newer terminal consequence;
- two derived LMDB databases are atomically replaceable per Case but remain
  outside canonical commit success; drop/rebuild and derivation failure leave
  ledger and CaseState unchanged;
- qualified retrieval filters Case, generation, participant/admitted view,
  lifecycle, semantic kind and resource/causal refs before explainable purpose/
  posture/recency ranking and a hard result budget;
- `yai.projection.v2`/`yai.context_frame.v2` consume the typed RetrievalSet with
  provenance and canonical fallback; Provider A→effect→Provider B/model
  replacement proves memory is independent of conversation/KV;
- the active `/memory propose` MemoryCandidate writer is retired; legacy
  MemorySummary/RecordKind handling is compatibility-only.

No embedding model, vector database, learned summary, compression hierarchy,
memory daemon or Agent owner was added.

## Completed boundary — agentless Case runtime and semantic residency

`YAI.SOURCE.REFOUNDATION.6` connected the existing semantic boundaries into a
disposable synchronous Case runner:

- every iteration reloads canonical CaseState, reconciles unresolved effects,
  repairs derived memory, retrieves qualified experience, plans residency,
  compiles a fresh Projection/ContextFrame and invokes the current provider;
- `yai.residency_plan.v1` classifies mandatory pinned, retained,
  reintroduced and omitted semantic refs with inspectable reasons under item
  and semantic-unit limits; it is derived artifact metadata, not Case memory;
- invocation, operation, semantic-context and cumulative estimated-input
  budgets stop before extra transport/effect, while optional provider usage is
  recorded as telemetry rather than tokenizer truth;
- `case run/resume/status/stop` expose bounded execution-attempt state without
  creating Agent, Workflow or Orchestrator ownership;
- canonical ProviderResult, Grant, visible-effect and post-FINALIZE/pre-memory
  crash points resume from ledger/CaseState, including automatic reconciliation
  before the next model call;
- a real-HTTP 26-turn proof performs 24 controlled writes, one DENY/adaptation,
  provider/model replacement and bounded 12-item context; a separate
  128-iteration test grows more than 380 Transitions while context stays
  bounded;
- `main.rs` is smaller than its Wave-5 baseline because memory command-family
  behavior moved behind an explicit CLI boundary.

The current runner is synchronous and single-process. Its run checkpoint is
non-authoritative operator metadata, and the only effect family remains local
`filesystem.write`.

## Completed boundary — typed human review and durable runtime admission

`YAI.SOURCE.REFOUNDATION.7` removed the fixed review world and implemented:

- `yai.transition.v4` / `yai.case_state.v4` typed Operation-bound
  ReviewRequest, integrity-bound ReviewAction and effective Decision refs;
- explicit `REQUIRE_REVIEW`, where no Grant/PREPARE/effect exists until an
  eligible bound human Participant records APPROVE and runtime resumes;
- durable APPROVE/DENY/DEFER state with query purity, duplicate/stale/wrong-
  participant rejection and replay-equivalent CaseState;
- provider/model replacement during review while preserving the original
  Operation identity and observed second-turn consequence;
- provenance-bound review memory and mandatory unresolved review Projection
  posture without treating approval as resource evidence;
- `yai.case_runtime_admission.v1` in noncanonical LMDB metadata, providing
  process-safe single-Case exclusion, normal/pause release and dead/expired
  owner reclamation;
- deterministic R1–R6 review crash recovery and removal of the old fixed
  Case/review/attempt/path, direct review effect, JSONL dual-write, quarantine
  writer and `CompatibilityReview` constructor.

The local CLI trust boundary verifies a claimed bound Participant and review
eligibility; it does not authenticate an operating-system person, SSO identity
or remote signature.

## Stage 8 — durable cancellation, expiry, and Case closure

The exact next task is `YAI.SOURCE.REFOUNDATION.8 — Durable Cancellation,
Authority Expiry, and Case Closure`. It should give current live consumers one
typed fail-closed lifecycle contract for:

- operator cancellation versus disposable run stop;
- open/closing/closed Case transitions and their effect on runtime admission;
- stale pending Operation/review invalidation when Case/resource policy changes;
- ExecutionGrant expiry/revocation before PREPARE, without pretending an
  already-started external effect can be rolled back;
- deterministic restart/replay behavior at cancellation and closure
  boundaries;
- provider Projection of cancellation/closure truth without adding Workflow or
  a generic policy language.

## Explicit non-goals

This roadmap does not introduce Space or Agent as owners, import `yai-dev`,
clone YVEX, create a directory per concept, or require ContextDelta. Stage 8
does not introduce a generic Workflow, policy language, inbox daemon,
distributed lease service or second carrier while closing lifecycle authority.

## Exit criteria for the next source task

`YAI.SOURCE.REFOUNDATION.8` is complete only when Case closure/cancellation is
canonical, pending review/Operation and unconsumed Grant authority fail closed
under the lifecycle contract, restart/replay are deterministic, and no
external-effect rollback is falsely claimed. Every earlier review, runtime-
admission, replay, residency, memory, provider-replacement and controlled-
effect invariant must remain green.
