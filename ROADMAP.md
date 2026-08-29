# YAI implementation refoundation roadmap

Authority: implementation delta between [the constitution](docs/constitution.md)
and [the current executable architecture](docs/architecture.md). A named
concept or stage does not automatically justify a source subsystem.

## Objective

Recover the strongest executable foundations of the original YAI through a
direct `yai-dev` archaeology → current characterization → semantic
differential → minimal refounded implementation cycle. The recovery ledger is
navigation, never historical authority. Existing transition/effect/context/
memory/review guarantees remain fixed while governance, lifecycle, tenant,
runtime, provider and longevity semantics are recovered without restoring the
old planes, Agent sovereignty or noun-per-module topology.

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

## Completed boundary — governance source intake and policy artifacts

`YAI.SOURCE.REFOUNDATION.8` began the systematic Foundation Recovery program.
Direct archaeology across the March 2026 ingestion, review, registry,
workspace-attachment and later topology/drain epochs recovered the useful
supply-chain properties while rejecting the former governance forest. Current
YAI now provides:

- bounded `yai.policy_source_input.v2` constrained JSON with exact-byte SHA-256
  identity, declared source-origin provenance and retained immutable
  `yai.policy_source_artifact.v2` (v1 remains readable);
- deterministic typed parsing for operation restrictions, review requirements
  and evidence obligations, with source JSON-location provenance;
- normalized `yai.policy_ir.v1` with deterministic digest, deduplication,
  unresolved semantics and typed conflict blockers;
- immutable/versioned `yai.policy_artifact.v2` candidates and an append-only
  independent governance lifecycle (`candidate → validated → published →
  superseded/retired`) in the existing LMDB environment;
- `runtime_consumable` only as a derived published-and-qualified disposition;
  no Case binding, EffectivePolicy, Decision, Grant, provider call or carrier
  effect is produced by authoring;
- idempotent duplicate intake, immutable P@1/P@2 history, pure inspection and
  fail-closed malformed/unknown/conflicting input.

The local `--as` actor is lifecycle provenance, not authenticated enterprise
identity or policy authority. Full source bytes are currently retained under a
hard bound; global privacy/retention policy remains open.

## Completed hardening — governance artifact foundation

`YAI.FOUNDATION.HARDENING.8` re-ran direct cross-family archaeology and made
the Wave-8 foundation safe for later Case binding without implementing it:

- policy lineage is exactly `owner_ref + policy_key`, preventing cross-owner
  supersession while keeping tenant/Principal semantics deferred;
- one declared version in one lineage can identify only one immutable content;
  changed bytes fail atomically instead of creating ambiguous `P@version`;
- duplicate JSON keys, pathological depth, BOM/UTF-8/identifier violations and
  malformed known rules fail before persistence;
- `source_system`/`source_uri` are bounded declared provenance distinct from
  source bytes, local paths, actor identity and authenticated ownership;
- validation is re-derived from stored IR, lifecycle/supersession refs are
  integrity checked, and LMDB abort/concurrency tests prove one current
  publication per lineage;
- a rebuildable current-lineage index accelerates exact lookup without becoming
  governance history;
- the shared LMDB default is now 256 MiB with an explicit 256-source catalog
  contract and fail-closed capacity exhaustion.

Case PolicyBinding, EffectivePolicy, normative readiness, precedence and
policy-driven authority remain the next recovery delta; H8 emitted none of
them.

## Foundation Recovery sequence

The local cancellation-first sequence is superseded. The current provisional
order is:

1. Wave 9: Case PolicyBinding, EffectivePolicy materialization, normative
   readiness, precedence/conflict/missingness.
2. Wave 10: policy-driven authority, DecisionBasis, obligations, review
   eligibility and policy-bound Grant.
3. Wave 11: validity/expiry/refresh/revoke, policy invalidation, historical
   policy replay, durable cancellation and Case closure.
4. Later waves: tenant/security isolation; multi-Case runtime; shared-resource
   fencing and a second carrier; provider governance; lifecycle/build/data
   longevity, each gated by fresh direct archaeology.

The sequence may change only when repository evidence establishes a stronger
dependency. Every wave is incomplete until its isolated commit is pushed and
`HEAD == origin/master == ls-remote`.

## Stage 9 — Case policy materialization

The exact next task is `YAI.SOURCE.REFOUNDATION.9 — Case PolicyBinding,
Effective Policy Materialization, and Normative Readiness`. It must recover and
prove the semantic delta between a published shared PolicyArtifact and the
immutable version actually bound/materialized for a Case. It must address
precedence, conflicts and missingness conservatively, but must not yet fold
authority resolution or policy-bound Grant issuance into the compiler.

## Explicit non-goals

This roadmap does not introduce Space or Agent as owners, import `yai-dev`,
clone YVEX, create a directory per concept, or require ContextDelta. It does
not restore the historical Governance/Compliance/Authority planes, registry
forest, Workflow, supervisor or embedded-law topology.

## Exit criteria for the next source task

`YAI.SOURCE.REFOUNDATION.9` is complete only when a Case can bind exact
immutable published artifact identities, rebuild one provenance-bound
EffectivePolicy materialization, distinguish existence from normative
readiness, fail closed on conflict/missingness/staleness, and preserve every
Wave 2–8 invariant. It must begin with a fresh direct `yai-dev` reinspection
and end with an isolated published commit; do not implement Wave 10 authority
resolution automatically.
