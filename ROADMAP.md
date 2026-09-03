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

- bounded `yai.policy_source_input.v3` constrained JSON with exact-byte SHA-256
  identity, declared source-origin provenance and retained immutable
  `yai.policy_source_artifact.v3` (v1-v2 remain readable);
- deterministic typed parsing for operation restrictions, review requirements
  and evidence obligations, with source JSON-location provenance;
- normalized `yai.policy_ir.v2` with deterministic digest, deduplication,
  unresolved semantics and typed conflict blockers;
- immutable/versioned `yai.policy_artifact.v3` candidates and an append-only
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

At that checkpoint H8 emitted no Case policy state. Wave 9 has since added the
exact binding/materialization boundary described below without adding
operational authority.

## Completed boundary — Case policy materialization

`YAI.SOURCE.REFOUNDATION.9` recovered the useful legacy fail-closed normative
qualification property while rejecting mutable profile aliases, generated
second truths and free readiness booleans. Current YAI now provides:

- canonical bind/replace/unbind history (introduced in v5, current
  `yai.transition.v6`) and compact `yai.case_state.v6` active bindings, each pinned to one exact immutable
  published PolicyArtifact and bind-time publication event;
- one active binding per owner-scoped policy lineage, explicit atomic
  replacement and no automatic adoption of a newer publication;
- rebuildable `yai.effective_policy.v2` under
  `yai.policy_materializer.v2`, with deterministic sorted composition and full
  contributing provenance;
- conservative DENY-over-ALLOW, required-review dominance, additive evidence
  obligations, and blocking missing/integrity failures;
- derived `unconfigured`/`ready`/`blocked` normative readiness and catalog drift
  reporting that does not yet imply revoke or refresh.

Binding and materialization do not alter the existing operation path and emit
no Decision, review request, Grant, effect or model/provider call.

## Foundation Recovery sequence

The current dependency order is:

1. Waves 10–12: complete — policy-driven authority, temporal governance,
   durable Case termination and authenticated Tenant security domains.
2. Wave 13 and H13: complete — local multi-Case RuntimeInstance, bounded
   workers, Tenant-fair scheduling/backpressure, canonical-first recovery,
   process-bound operational ownership and terminal-ack closure.
3. Wave 14 and H14: complete — shared resource authority, monotonic epochs,
   carrier-enforced fencing, a second physical carrier, and adversarial
   stale-writer, rebuild, TOCTOU and process-uncertainty qualification.
4. Wave 15: complete — immutable Tenant-bound
   WorkflowDefinitions, exact Case adoption, deterministic resolution, bounded
   ready work, ModelWork and deterministic/passive progression.
5. H15: complete — workflow determinism, replay, crash,
   predicate type safety, definition integrity and authority-isolation closure.
6. Wave 16: complete — CLI Product Refoundation: porcelain/plumbing separation,
   canonical command registry, parser lanes and output-family alignment.
7. Wave 17: complete — adaptive workflow evolution, typed same-Tenant
   multi-Case handoff, PlanPatch, bounded exact Subflow and reconciliation.
8. H17: complete — amendment lineage/concurrency, effective-topology replay and
   upgrade closure, nested recovery, Handoff forgery/race resistance and
   derived multi-Case graph rebuild.
9. Wave 18: complete — Tenant-scoped provider targets, evidence-bound
   qualification/capability provenance, explicit trust, shared health,
   deterministic Case-local selection and delivery-safe failover.
10. H18: complete — provider-governance corruption, lifecycle/concurrency,
    endpoint/credential drift, extension spoofing, transport-boundary and
    long-outage pressure qualification.

The post-H18 reassessment found no evidence for another semantic owner or an
automatic numbered Foundation Wave. Subsequent work must be selected from
observed deployment or product pressure and must reopen archaeology when it
changes a load-bearing property. Workflow design is recorded in
`refoundation/foundation-recovery/WORKFLOW-REFOUNDATION-PLAN.md`; Wave 15 owns
the first executable kernel, H15 its adversarial closure, Wave 16 the product
CLI boundary, Wave 17 adaptive composition, H17 its adversarial closure and
Wave 18/H18 provider-governance implementation and adversarial closure. The
selection record is
`refoundation/foundation-recovery/hardening-18/post-h18-roadmap-reassessment.md`.

The sequence may change only when repository evidence establishes a stronger
dependency. Every wave is incomplete until its isolated commit is pushed and
`HEAD == origin/master == ls-remote`.

## Completed boundary — policy-driven authority

`YAI.SOURCE.REFOUNDATION.10` freshly re-inspected legacy mediation and recovered
its fail-closed useful properties without its planes, mutable profiles,
default-ALLOW or ambient operator booleans. New live filesystem admission now:

- requires derived normative readiness before provider invocation;
- evaluates a normalized Operation under the exact `yai.effective_policy.v2`
  through immutable `yai.decision_basis.v1`;
- denies an applicable DENY or absence of explicit applicable ALLOW;
- intersects policy with the hard attachment envelope and Case-bound
  proposer/reviewer all-of roles;
- gives source-provenance, audit-reason and pre/post observation obligations
  typed executable meaning;
- records `yai.decision.v2`, optionally pauses on
  `yai.review_request.v2`, and issues `yai.execution_grant.v2` only after final
  ALLOW under the same current policy basis.

Policy change before review or Grant fails closed. Legacy resource policy-owner
and review fields remain readable but are not active authority in this path.

## Completed boundary — temporal governance and Case termination

`YAI.SOURCE.REFOUNDATION.11` adds explicit immutable policy validity,
append-only revoke, a rollback-safe authority-time floor, typed review/Grant
invalidation, finite Grant v3 authority, durable Case cancellation and terminal
non-destructive closure. `yai.transition.v7`/`yai.case_state.v7` replay the
barriers. PREPARE is the non-retroactive cut: authority can contract before it;
after it the effect must finalize or reconcile.

## Completed boundary — authenticated Tenant security domains

`YAI.SOURCE.REFOUNDATION.12` replaces human/admin string claims on the new live
path with a kernel-observed local POSIX Principal projection. One durable
security owner records immutable `yai.security_principal.v1` and
`yai.tenant.v1` objects plus minimal Owner/Member history. Every new Case has
one immutable Tenant; legacy v1-v7 Cases remain read/replay-only and cannot
acquire new authority implicitly.

New policy artifact v5 identity and lineage are Tenant-scoped, Case binding v2
rejects cross-Tenant artifacts, EffectivePolicy v3 and DecisionBasis v3 retain
the exact security domain, and ReviewAction v2 resolves an authenticated
Principal through Tenant membership and an explicit one-to-one Case
Participant link before applying existing policy roles. Organization remains
Tenant metadata, not a second runtime owner. Product reads and canonical
administrative writes enforce the Tenant boundary; exact or overlapping local
filesystem roots across Tenants fail closed. This is local process/runtime
isolation over OS-protected LMDB, not SSO, credential-vault, container or VM
isolation.

## Completed boundary — multi-Case runtime concurrency

`YAI.SOURCE.REFOUNDATION.13` and `YAI.FOUNDATION.HARDENING.13` add one local
foreground RuntimeInstance with a finite worker pool, durable bounded
WorkItems, Tenant round-robin/FIFO, per-Tenant and global backpressure, one
active work item per Case, process-bound instance ownership and restart-stable
fairness. Recovery derives terminal/parked WorkItem posture from exact Case
checkpoint and canonical truth; lost scheduler acknowledgement cannot reopen a
completed or denied Case attempt. This operational state is not Case authority.

At the Wave-13 boundary, the remaining physical gap was cross-process
shared-resource exclusion: scheduler-local root overlap serialization could
not stop a separate direct Case runner from reaching the same mutable resource.
Wave 14/H14 closed that local single-host gap through the fencing boundary
described below; distributed cross-host exclusion remains unclaimed.

## Completed boundary — shared-resource fencing and second carrier

Wave 14 owns one Tenant-bound shared-resource authority, monotonic resource
epochs, PREPARE-atomic fence acquisition, carrier-side current-fence
validation, stale-writer rejection and release only with terminal effect truth.
It also qualifies a second real physical carrier through the same
Operation→Decision→Grant→PREPARE chain and establishes a cumulative external
YVEX provider pressure-test surface. It does not implement Workflow or provider
governance.

## Explicit non-goals

This roadmap does not introduce Space or Agent as owners, import `yai-dev`,
modify YVEX from YAI work, create a directory per concept, or require
ContextDelta. It does
not restore the historical Governance/Compliance/Authority planes, registry
forest, Workflow, supervisor or embedded-law topology.

## Post-W19 work-selection gate

There is no active numbered Foundation Wave. A next source task requires
executable product, deployment or external-consumer evidence that identifies a
specific missing property. W19 was admitted by explicit long-horizon memory
product pressure after H18; it did not authorize H19. Distributed consensus,
production trust and credential provisioning, higher-order memory
consolidation, governed read/list/stat/search capabilities, retention policy,
background indexing and richer diagnostics remain candidate pressure areas,
not pre-authorized semantic owners. Their classifications and non-claims are
maintained in the W19 dossier.

## Completed boundary — adaptive Workflow semantic closure

Wave 17 added typed bounded PlanPatch candidates, authenticated Case-local
amendment adoption, deterministic EffectiveWorkflowTopology derivation, exact
same-Case Subflow expansion and same-Tenant Case Handoff. H17 closes long
lineage, concurrent mutation, nested recovery, cross-Case forgery,
cancellation/close ordering, concurrent cycle admission and software-upgrade
digest drift. Immutable Definitions and the existing
policy/Decision/Grant/ResourceFence/RuntimeInstance boundaries remain intact.
No Agent, Orchestrator, WorkflowRun, multi-Case Process owner or provider
governance is added.

## Completed boundary — provider governance and adversarial closure

Wave 18 adds immutable Tenant-scoped ProviderTargets, synthetic evidence-bound
qualification, derived capability provenance, Tenant-Owner approval/denial,
shared fresh operational health/circuit state, exact Case provider bindings,
mechanical invocation requirements, deterministic Case-canonical selections
and bounded attempt outcomes. Legacy ProviderAttached Cases remain exact pins.

Failover is only `none` or `safe_only`: failure before any request byte may
select another exact eligible target, while possible remote delivery without
an authoritative result stops as `delivery_indeterminate`. Selection remains
cognition routing, never Policy/Decision/Grant/Resource authority. The
optional documented YVEX HTTP extension is observation-only; YAI does not
administer YVEX or depend on its private local protocol. H18 adversarial
hardening subsequently closed qualification/trust replay, credential rotation,
rollback-safe health, half-open concurrency, DNS/TLS transport, selector,
delivery and extension-spoofing boundaries. Cross-host governance, deployment
credential stores and external-provider availability remain explicit gaps, not
provider-brand branches or evidence for a new owner.

## Completed boundary — derived hybrid memory indexing

Wave 19 preserves Transition history as memory authority and OperationalMemory
as its rebuildable typed materialization. It adds deterministic
`yai.memory_representation_document.v1` values, exact Tenant/encoder/profile
identity, derived embedding artifacts and content-addressed corpus/index
manifests without increasing the 37/40 LMDB database count.

Disposable filesystem bundles provide BM25 and normalized exact-cosine
candidate planes. Case/generation/Participant/view/lifecycle/resource/causal
qualification remains ahead of deterministic reciprocal-rank fusion, so
similarity cannot grant visibility, upgrade ProviderClaims or authorize an
Operation. Builds are bounded, locked, corruption-detecting and atomically
published; profile replacement creates a new namespace; drop/rebuild preserves
Case truth. The existing Projection v5 → ResidencyPlan → ContextFrame v5 path
consumes v2 RetrievalSet selections and degrades to v1/canonical retrieval.

ANN is explicitly deferred after 1k/10k/50k exact-scan characterization; exact
scan remains the oracle. Embedding calls are limited to separately qualified
loopback targets under W18 ProviderGovernance. YVEX/DeepSeek remains the
cognitive provider and gains no embedding endpoint or core special case.
Learned reranking, contradiction/consolidation and governed filesystem/process
capability expansion remain outside W19.
