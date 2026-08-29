# Current executable architecture

Authority: implementation truth. Foundation-recovery baseline:
`3403ecdd2a321b689e41d747cbeb9d9e7c58e5e1` (the published Wave-7
checkpoint). This document describes the resulting
`YAI.SOURCE.REFOUNDATION.8` worktree.

This document includes current contradictions. It does not claim that the
[Constitution](constitution.md) is implemented. Target changes and sequencing
belong only in the [Roadmap](../ROADMAP.md).

## Executable summary

YAI has two unequal product processes:

```text
operator
  |
  +-- yai (Rust command/process boundary)
  |     +-- typed Projection/ContextFrame compilation and provider invocation
  |     +-- provenance-bound operational-memory derivation/retrieval
  |     +-- derived semantic ResidencyPlan and bounded Case execution loop
  |     +-- controlled filesystem effect transition family
  |     +-- Case-native typed human review on the controlled carrier
  |     +-- durable single-Case runtime admission metadata
  |     +-- deterministic governance intake and immutable policy artifacts
  |     +-- canonical Transition/CaseState authority in LMDB
  |     +-- legacy journal inspect/import/replay compatibility
  |     +-- LMDB graph materialization/query
  |     +-- DuckDB analytical derivation
  |     +-- yai-engine Rust library
  |     +-- selected yaid Unix-socket requests
  |
  +-- yaid (C daemon)
        +-- Unix-socket status/info/shutdown
        +-- prepared minimum/filesystem fixture loops
        +-- C JSONL records/journal and projections used by those loops
        +-- restartable hot-state snapshot
```

[`cmd/yai/src/main.rs`](../cmd/yai/src/main.rs) is a 1,926-line command
parser, dispatcher, common CLI support surface, and compatibility shell. The
current domain implementation is grouped by demonstrated boundary in
[`provider.rs`](../cmd/yai/src/provider.rs),
[`case_runtime.rs`](../cmd/yai/src/case_runtime.rs),
[`policy.rs`](../cmd/yai/src/policy.rs),
[`memory_cli.rs`](../cmd/yai/src/memory_cli.rs),
[`review.rs`](../cmd/yai/src/review.rs),
[`controlled_effect.rs`](../cmd/yai/src/controlled_effect.rs),
[`filesystem.rs`](../cmd/yai/src/filesystem.rs),
[`replay.rs`](../cmd/yai/src/replay.rs),
[`graph_runtime.rs`](../cmd/yai/src/graph_runtime.rs), and
[`analytics.rs`](../cmd/yai/src/analytics.rs). These modules do not imply
future subsystems; they isolate existing behavior for the next semantic
refoundation.

The normal CLI links [`yai-engine`](../engine/yai-engine/src/lib.rs) directly
as Rust. There is no product C→Rust call edge or installed Rust C ABI. The
former marker FFI crate and smoke bridge were removed.

The production C archive contains 16 sources. `yaid` adds only its entrypoint,
IPC, and core loop. Component-only C mechanics build into a separate
characterization archive and are linked only by their tests. Static archive
membership is no longer used to make those components look product-reachable.

## Demonstrated product verticals

“Complete” means that the bounded path has an entrypoint, consequence or
durable residue, operator output, and executable regression proof. It does not
mean constitutional, general, or production-ready.

| Vertical | Current path and demonstrated consequence | First architectural gap |
|---|---|---|
| Case-bound provider prompt | admitted participant + typed CaseState/history → qualified `yai.operational_memory.v1` retrieval → `yai.residency_plan.v1` → `yai.projection.v4` → `yai.context_frame.v4` → provider/model render → typed Invocation and ProviderResult lineage → non-authoritative ModelInterpretation; real HTTP fixtures prove rebuild, memory-backed provider/model replacement and continuation-loss fallback | HTTP is local/plain and non-streaming; ranking/residency are typed and deterministic; no learned compression or authoritative tokenizer |
| Agentless Case runtime | disposable bounded runner reloads CaseState → reconciles effects/review → repairs memory → retrieves/plans residency → invokes provider → normalizes/decides/effects → repeats from new canonical reality; typed `AWAITING_REVIEW`, provider/model replacement, crash recovery, and one transactionally admitted runner per Case are executable | one synchronous single-host `filesystem.write` loop; no background scheduler or distributed lease |
| Controlled filesystem effect | typed logical attachment + local binding → real HTTP ProviderResult → exact proposal normalization → typed Operation/Decision/ExecutionGrant → durable PREPARE → Rust atomic-replacement carrier → pre/post Observation and EffectReceipt → FINALIZE/RECONCILE → second provider turn | only `filesystem.write`; prefix policy and single-machine binding; no general review/policy/carrier system |
| Human-reviewed filesystem effect | normalized Operation → `REQUIRE_REVIEW` → typed ReviewRequest → bound human Participant action APPROVE/DENY/DEFER → effective Decision → existing Grant/PREPARE/carrier/FINALIZE path; review works with no live runner and survives provider/model replacement | local CLI asserts a bound participant identity but does not authenticate an OS person, SSO principal, or remote signer; only filesystem policy review exists |
| Governance intake | constrained JSON bytes → immutable source digest → typed parsed facts → normalized Policy IR → immutable candidate → deterministic validation → explicit local publication; P@1/P@2 remain distinct and lifecycle events are append-only | artifacts are not yet bound/materialized into any Case; local publisher identity is asserted, full source retention policy is provisional, and no authority is derived |
| Journal compatibility | inspect/dry-run/import `yai.store.record.v0` or `yai.record.v1`, preserving unknowns opaquely in an isolated target; old replay still materializes legacy record indexes | general semantic promotion is deliberately absent; the old record plane remains compatibility data, not authority |
| Graph | typed canonical transitions and explicitly decoded legacy records → derived relations → rebuildable RuntimeGraph → bounded query | generation/version invalidation remains minimal; legacy-only cases still depend on compatibility translation |
| Analytical facts | LMDB operational records → DuckDB extraction → reports | four declared families have no extractor; schema/orchestration remains embedded in the command crate |

`yaid` startup, status/info/shutdown, restart, fixture loops, and hot snapshot
reconstruction remain covered separately. The former direct `carrier
fs-write` product command is removed. Its read-only counterpart remains an
observation compatibility command. The C daemon filesystem loop now emits
explicit descriptor/no-effect fixture residue and no longer performs or claims
a product filesystem mutation.

Partial and component paths must not be promoted into product claims:

- `yai process observe` probes a PID but persists no Observation; CLI process
  signaling does not dispatch.
- the C process carrier observes, signals, and re-observes a test-owned child;
  it is a characterized platform boundary, not a product vertical.
- the C filesystem carrier preserves pre-state, real write, post-state hash,
  and receipt mechanics under component tests only.
- `yaid` filesystem fixtures create an input fixture and descriptor/no-effect
  records; they are case/journal test setup, not product effect evidence.

The provider planning registry, CaseHandle/CapabilityLease inspection views,
generic carrier registries, synthetic dispatch families, C graph/index/memory
mirrors, `net/`, and its unconsumed `proto/` fixtures are absent. None had a
product caller or unique property not already captured by the surviving
verticals/tests.

## Current state and schema authority

### Canonical LMDB authority

Rust owns one canonical semantic write path in
[`transition.rs`](../engine/yai-engine/src/transition.rs) and
[`lmdb.rs`](../engine/yai-engine/src/store/lmdb.rs). Its serialized contracts
are `yai.transition.v4` and `yai.case_state.v4`; readers retain v1/v2/v3 state
and transition compatibility while rejecting unknown future contracts.
Version 3 added provider identity, semantic-frame/render lineage and typed
interaction turns. Version 4 adds Operation-bound ReviewRequest,
integrity-bound ReviewAction, effective Decision refs and resource review
posture. It does not make derived context canonical. One bounded LMDB write
transaction:

1. validates typed payload closure and global Transition identity;
2. compares the expected per-Case generation;
3. appends the immutable Transition by identity and zero-padded Case sequence;
4. reduces it into CaseState;
5. commits ledger and materialization together.

The ledger databases are `transitions_by_id` and
`case_transition_sequence`; `case_state` is the rebuildable materialization.
Per-Case sequence, not timestamp, determines reducer order. Duplicate
Transition IDs and stale generations fail deterministically. Reopening after a
commit exposes both history and state; an injected failure before commit
exposes neither. Rebuild verifies:

```text
materialized CaseState == replay(ordered canonical Transitions)
```

The reducer deliberately covers only current live fields: Case lifecycle and
generation, participant bindings/admitted views, one current provider/model
attachment, latest provider invocation/result/interpretation lineage, typed
Operation-bound review state, logical filesystem attachments, latest Operation/Decision, Grant
lifecycle, and compact prepared/finalized/indeterminate effect refs. Full
content, Observations, and Receipts remain in immutable Transitions rather than
turning CaseState into an object bag.

Machine-local absolute filesystem roots are stored in a separately versioned
`local_resource_bindings` LMDB database. They survive restart because the
carrier needs them, but they are neither portable Case identity nor historical
authority. Canonical Operations target a logical attachment plus normalized
relative path.

### Legacy compatibility

The C/Rust journal remains readable as `yai.store.record.v0`; LMDB legacy
record envelopes remain `yai.record.v1`. Rust still defines 35 legacy kinds and
C 32. [`compatibility.rs`](../engine/yai-engine/src/compatibility.rs) is the
only summary-token decoder. It classifies input as losslessly structurally
promoted, promoted with compatibility metadata, preserved opaque, or rejected
malformed. Unknown future kinds/schemas are retained without invented meaning.

`yai journal compatibility-inspect` and `compatibility-import` support inspect,
dry-run, and import into an explicitly isolated LMDB target. Compatibility
payloads never append canonical Transitions or CaseState. Old `journal replay`
and legacy record indexes survive for operator/data compatibility; they are no
longer historical authority. Provider compatibility commands still emit old
JSONL/record shapes where required. New review actions write only typed
canonical Transitions and never dual-write `control/review.jsonl`.

### Derived/cache stores

- LMDB graph relations are rebuilt from typed Transitions plus the explicit
  legacy compatibility decoder; graph failure cannot affect canonical commit.
- RuntimeGraph is per-command and ephemeral.
- DuckDB facts are rebuildable analytics; historical extraction is routed
  through compatibility fields.
- `operational_memory_by_id` and `operational_memory_case_index` store
  `yai.operational_memory.v1` entries plus a generation/derivation manifest.
  They are updated after canonical commit, may fail independently, and can be
  cleared and deterministically rebuilt from ordered Transitions. They are not
  part of canonical transaction success.
- bounded typed Projection, ContextFrame and rendered-input metadata are stored
  in the separate `semantic_context_artifacts` LMDB database for inspection;
  this database is droppable and is never read by replay or CaseState reduction.
  Full rendered provider input, token sequences, and continuation values are
  not persisted.
- `yai.residency_plan.v1` is stored in the same droppable artifact database for
  inspection. It is a pure derived selection decision, never CaseState or a
  precondition for replay.
- Rust `ProjectionSummary`, `MemorySummary`, query and reconcile summaries
  remain legacy compatibility views and are not provider input. The former
  `/memory propose` producer is retired and cannot append `MemoryCandidate`.
- the C hot-state JSON snapshot is a daemon restart cache and is not updated by
  independent Rust mutations.

No canonical reducer, graph relation, provider/review decision, memory
category, or fact field parses arbitrary `summary` text. Remaining summary
grammar exists only inside the named legacy compatibility boundary and in
presentation assertions.

## Current control and effect behavior

[`effect.rs`](../engine/yai-engine/src/effect.rs) owns the only
product-reachable `filesystem.write` contract and carrier mechanics.
[`controlled_effect.rs`](../cmd/yai/src/controlled_effect.rs) owns its bounded
product orchestration. The implemented path is:

```text
typed CaseState/resource attachment
→ real OpenAI-compatible invocation
→ non-authoritative ProviderResult
→ strict yai.operation_proposal.filesystem_write.v1 decoder
→ yai.operation.v1
→ deterministic prefix Decision(ALLOW|DENY|REQUIRE_REVIEW)
→ integrity-bound yai.execution_grant.v1
→ typed pre-observation
→ EffectPrepared Transition
→ rust.filesystem.atomic_replace.v1
→ typed post-observation + yai.effect_receipt.v1
→ EffectFinalized or EffectIndeterminate/EffectReconciled Transition
→ rebuilt typed consequence view
→ second real provider invocation
```

The proposal decoder rejects unknown fields/schema/operation, malformed or
natural-language-only output, wrong attachments, empty/oversized content,
absolute paths, dot components, and traversal. Normalization creates stable
Operation identity and content digest. Provider material cannot construct a
Decision, Grant, Receipt, or canonical resource identity. The deterministic
policy owner must be a bound participant distinct from the model participant;
ALLOW is limited to the attachment's normalized prefix.

The immutable Grant binds Operation/Decision digests, Case and participant,
logical attachment, normalized target, intended digest, expected Case
generation, idempotency key, and pre/post observation obligations. The carrier
accepts no unprepared Grant: CaseState must show that the exact Grant was
consumed by the current `EffectPrepared` transition. Any intervening Case
transition makes an issued Grant stale before PREPARE. Repeated invocation of
one prepared effect observes intended post-state and returns `already_applied`
without another mutation.

Before mutation YAI records file absent/file/type plus SHA-256 digest and size
where applicable. The carrier canonicalizes the existing target parent, rejects
symlink escape, creates a same-directory temporary file, writes and `fsync`s
it, atomically renames it, and `fsync`s the parent directory. The implemented
durability claim ends at those successful local filesystem calls; hardware,
remote filesystem, or whole-system durability is not claimed. Applied status
requires a post-observed digest equal to the intended digest.

A crash after PREPARE leaves a discoverable prepared effect. `yai effect
reconcile --case ...` enumerates unresolved CaseState refs after restart and
compares the real target with the persisted expected pre-state and intended
post-digest. It concludes effect observed, no effect observed, conflict, or
still indeterminate. `--retry` is allowed only for an unchanged current
PREPARED state; ambiguous or conflicting state is never guessed away. Tests
inject crashes after Grant, after PREPARE, after visible rename, and after
receipt construction but before FINALIZE. The visible-effect crashes finalize
after restart without a duplicate write.

[`review.rs`](../cmd/yai/src/review.rs) owns only the Case-native operator
boundary. A resource attachment may require review by its bound policy-owner
Participant. `REQUIRE_REVIEW` issues no Grant, commits a typed request for the
already-normalized Operation, and stops the runtime. APPROVE/DENY/DEFER actions
are identity- and digest-bound to that review, Case, Operation, reviewer and
expected generation. APPROVE and DENY derive a new effective Decision;
approval itself performs no effect. Resume revalidates current Case/resource
bindings and executes the original Operation through the existing controlled
carrier. `CompatibilityReview`, `PendingOperator`, `ReviewResolved`, and
`Quarantined` remain reader/reducer vocabulary for v1-v3 data only; no active
writer or product command produces them.

The direct `carrier fs-write` command and Rust primitive were removed after
characterization. [`filesystem.rs`](../cmd/yai/src/filesystem.rs) now owns only
read-only compatibility observation. `yaid run-filesystem-loop` no longer
writes `output.txt` or claims an executed receipt; it produces explicit
descriptor/no-effect fixture records used by older tests.

Surviving C control, filesystem/process carrier, receipt, and observation
components contain typed or platform properties protected by component tests.
They are built separately and are not normal product call paths.

## Current governance intake behavior

[`governance.rs`](../engine/yai-engine/src/governance.rs) owns one source
compiler, not a governance plane. It accepts only bounded constrained JSON
under `yai.policy_source_input.v2`. The exact UTF-8 bytes receive a SHA-256
identity and become an immutable `yai.policy_source_artifact.v2`; no model or
free-form policy interpreter participates.

The compiler emits `yai.parsed_policy.v1` facts for exactly three current
families: operation restriction, review requirement and evidence obligation.
Every fact retains its source artifact and JSON location. Normalization emits
`yai.policy_ir.v1`, deterministically deduplicates equivalent semantics,
preserves unknown rule kinds as unresolved, and records contradictory outcomes
as typed conflicts. Unknown syntax/schema or malformed known rules fail;
unresolved/conflicted candidates remain inspectable but cannot validate.

`yai.policy_artifact.v2` embeds the parsed and normalized provenance chain,
including bounded declared `source_system` and `source_uri` origin metadata,
and
uses source version plus content/IR digests for immutable identity. The same
LMDB environment contains four logically separate canonical governance
databases: immutable sources, immutable artifacts, lifecycle events by ID and
their append order. A fifth rebuildable index accelerates current-policy lookup
but is not authority. This is an independent governance history, not a synthetic
Case ledger and not derived state. Lifecycle is reconstructed from integrity-
bound events:

```text
candidate → validated → published → superseded | retired
```

The shared LMDB map is configurable at open and defaults to 256 MiB (formerly
16 MiB). The supported H8 catalog contract is 256 retained sources of up to
256 KiB each plus their artifacts/events in the shared environment; capacity
exhaustion is explicit and cannot partially commit a governance transaction.

The artifact bytes never change. Policy lineage is exactly `owner_ref +
policy_key`; a declared version identifies at most one immutable content inside
that lineage. Publishing another validated version in the same lineage appends
`superseded` for the previous publication and
`published` for the new artifact. `runtime_consumable` is true only for a
qualified artifact whose derived lifecycle is currently `published`; it means
eligible for future Case binding, not effective or authoritative now.

[`policy.rs`](../cmd/yai/src/policy.rs) provides ingest/inspect/validate/
publish/retire/list. Reads are pure. Mutating commands record a claimed local
actor ref for provenance, but H8 does not authenticate that person and actor
identity is not lineage ownership. Full
bounded source bytes are retained to make compilation reproducible; source
artifact absence or corruption leaves the separately stored artifact's
digest/parsed/IR and declared origin inspectable, but byte-level recompilation
then cannot be claimed. There is no product source-deletion lifecycle. Global
source retention/privacy policy remains open.

No policy authoring operation appends a Case Transition, invokes a provider or
carrier, creates a Decision/Grant, or modifies filesystem resources. Case
PolicyBinding, EffectivePolicy, normative readiness, precedence and policy-
driven authority begin in later recovery waves. See the canonical
[governance reference](reference/governance.md).

## Current provider and context behavior

[`context.rs`](../engine/yai-engine/src/context.rs) owns a pure compilation
boundary from typed CaseState, ordered canonical Transitions and an optional
qualified RetrievalSet to an immutable candidate Projection. The pure
[`residency.rs`](../engine/yai-engine/src/residency.rs) planner applies a
`yai.residency_plan.v1` budget before the compiler emits `yai.projection.v4`
and one task/output-contract-specific `yai.context_frame.v4`. Projection identity binds
Case generation, participant/purpose/admitted view, ordered typed entries,
provenance and bounded omission state. Provider availability, rendering,
tokenization, KV state and opaque continuation identity do not participate.
Availability flags alone do not change semantic identity; selected memory and
its explicit omissions do. If graph/memory is unavailable, the required
CaseState/history entries remain reconstructible while optional context may be
absent.

[`memory.rs`](../engine/yai-engine/src/memory.rs) first derives a versioned
operational-memory materialization from typed invocation/result, normalization,
Decision and effect-chain Transitions. Each entry has a deterministic identity,
Case/generation, semantic kind, epistemic posture, bounded typed value,
Transition/Observation/Receipt/causal provenance, participant visibility and
active/superseded lifecycle. Provider claims stay explicitly
`provider_originated_claim`; only finalized/reconciled observations produce an
observed resource-effect memory. PREPARED/INDETERMINATE residue stays unresolved.

Retrieval is a pure `qualify → filter → rank/select` algorithm. It filters Case,
current generation, admitted participant/view, lifecycle, typed kind and direct
resource/causal constraints before deterministic ranking by posture, purpose,
direct match and recency. It defaults to eight entries and reports selection,
omissions, rejections and machine-readable reasons. No embeddings, vector store,
graph requirement or provider-specific input participates. Missing/stale memory
falls back to canonical CaseState/history selection.

The compiler fails before rendering if the participant lacks the exact
`model/model_context` admission. It includes the participant's own binding,
current provider/model binding, logical resources, latest Decision, all
unresolved effects, the four most recent finalized effects, bounded recent
typed interaction turns/provider claims, and typed provenance-bearing retrieved
memory. Candidate selection is intentionally broader than provider input.
Residency pins mandatory current/unresolved/observed truth first, then retains
or reintroduces ranked optional entries under item and semantic-unit limits;
every omission has an inspectable reason. Provider claims carry an explicit
non-authoritative posture;
finalized resource consequences cite Transition, Observation and EffectReceipt
refs; indeterminate effects remain unresolved. Runtime selection defaults to
24 items and 4,096 semantic units, never dumps the complete ledger, reports
omitted material, and rejects a budget smaller than mandatory current state.

ContextFrame has separate identity because one Projection supports different
tasks and typed output contracts. It carries provider-independent instructions,
selected semantic entries and the Wave-3 filesystem proposal contract. It owns
no CaseState, prompt transcript, token IDs, or runtime cache. The
OpenAI-compatible render function in
[`context.rs`](../engine/yai-engine/src/context.rs) combines a frame with the
minimal provider/model profile and creates a distinct render identity/digest;
[`provider.rs`](../cmd/yai/src/provider.rs) owns the HTTP transport.

`yai.transition.v4` Invocation and ProviderResult payloads explicitly reference
provider ID, model ID, Projection, ContextFrame, Case generation, render ID/
digest and output-contract ID. ProviderResult content remains non-authoritative.
A typed `InteractionTurnRecorded` transition preserves bounded task lineage;
the old JSONL `InteractionTurn` remains compatibility output. New invocations
no longer write or consume `ParticipantViewFrame`; that RecordKind survives
only as historical input/counting compatibility. The free-form Case-entry
preview is explicitly labeled compatibility output and never reaches a
provider.

An optional `ProviderContinuationReference` is accepted only as an opaque,
provider-bound, runtime-bound transport optimization. Its value is never put in
CaseState, canonical Transition history, Projection/Frame identity, or the
derived artifact store. Only `not_provided`, `used`, or
`invalidated_and_retried` disposition is recorded in invocation lineage. An
`invalid_continuation` response triggers one retry of the same complete rendered
frame without the reference.

Product tests prove that Provider A can propose a real Wave-3 filesystem write,
Provider B can replace its binding after FINALIZE and observe both the current
typed resource consequence and its selected derived memory with Transition,
Observation and Receipt provenance, and a model ID can change under one provider
ID without changing Case identity. A separate fixture loses
continuation state, retries a full frame, restarts on a new endpoint, rebuilds a
new Projection/Frame from Case state, and preserves typed interaction/result
continuity. Loss of the derived context-artifact database likewise leaves
ledger and CaseState unchanged. OpenAI-compatible usage fields are captured
when supplied; token counts and latency are invocation telemetry rather than
operational authority. Unavailable usage remains unknown.

There is still no TLS, streaming/cancellation, authoritative token estimator,
native YVEX/KV protocol, embedding/learned ranking, semantic compression, or
ContextDelta consumer. The context-residency lab remains research evidence and
does not prove KV reuse.

## Current agentless Case runtime

[`case_runtime.rs`](../cmd/yai/src/case_runtime.rs) owns one disposable
transition algorithm. It is not an Agent, workflow, scheduler, or state owner.
Each iteration reloads CaseState, reconciles prepared/indeterminate filesystem
effects, repairs stale or missing derived memory, performs qualified retrieval
and Residency planning, compiles a fresh Projection/ContextFrame, invokes the
current provider/model, persists ProviderResult, and advances a valid candidate
through the existing Operation/Decision/Grant/effect boundary. The next
iteration starts from the newly committed Case generation rather than an
in-process narrative.

`yai case run`, `resume`, `status`, and `stop` expose bounded operator control.
A versioned JSON run checkpoint records only disposable execution-attempt
metadata: budgets, counters, pending ProviderResult identity, last derived
artifact/effect refs and stop reason. It owns no Case facts. Stops distinguish
completion, denial, malformed/provider failure, unresolved effect, budget
exhaustion, operator stop and invariant failure; a stopped run does not close
the Case.

One `yai.case_runtime_admission.v1` record in a separate LMDB database provides
single-host cross-process mutual exclusion for active Case advancement. The
claim binds Case, run, opaque owner token, PID and bounded expiry. LMDB write
serialization makes acquisition exclusive; same-owner renewal is explicit,
live competing owners fail closed, and expired or demonstrably dead local
owners can be reclaimed. Normal stop, completion, budget stop and
`AWAITING_REVIEW` release it. This metadata is not Transition history,
participant authority, an ExecutionGrant, or Case continuity.

Invocation, operation, semantic-context and cumulative estimated-input budgets
are enforced before transport or effect. Semantic size and conservative
rendered-input estimates are distinct from optional provider-reported token
usage. Mandatory current truth cannot be displaced by optional memory; if it
alone exceeds the configured budget, invocation fails explicitly.

Deterministic real-HTTP characterization performs 26 invocations and 24 real
Grant-controlled writes, including one DENY followed by a compliant proposal,
provider/model replacement after a committed ProviderResult, bounded 12-item
context, and restart after canonical-result, Grant, visible-effect and
post-FINALIZE/pre-memory boundaries. A visible filesystem effect with no
FINALIZE is reconciled before the next provider call. A separate 128-iteration
state/memory/context test grows more than 380 Transitions while keeping
retrieval and frames bounded. No Agent, Workflow or Orchestrator object
participates.

## Physical ownership after source refoundation

| Surface | Executable role | Classification |
|---|---|---|
| `cmd/yai/src/main.rs` | parsing, dispatch, common CLI/process initiation and residual compatibility commands | product-reachable command boundary |
| `cmd/yai/src/case_runtime.rs` | bounded disposable Case iteration, stop/budget checkpointing, automatic reconciliation and memory repair | product-reachable transition algorithm; never canonical owner |
| `cmd/yai/src/provider.rs` | Case admission/attachment compatibility, HTTP transport and typed invocation/result residue | product-reachable provider boundary |
| `engine/yai-engine/src/context.rs` | bounded typed Projection compilation, ContextFrame construction, provenance and the OpenAI-compatible render contract | product-reachable derived semantic compiler/render boundary |
| `engine/yai-engine/src/residency.rs` | deterministic mandatory/retained/reintroduced/omitted semantic selection and budget accounting | product-reachable pure derived planner; no persistent authority |
| `engine/yai-engine/src/memory.rs` | deterministic operational-memory derivation, provenance validation, supersession and qualified bounded retrieval; legacy MemoryCandidate summary compatibility | product-reachable derived algorithm/store contract; never canonical authority |
| `cmd/yai/src/controlled_effect.rs` + `engine/yai-engine/src/effect.rs` | controlled proposal/admission/recovery orchestration and the Grant-validating Rust filesystem carrier | product-reachable first constitutional effect family |
| `cmd/yai/src/review.rs` | Case-native typed participant actions and effective Decision recording; never carrier execution | product-reachable human review boundary |
| `engine/yai-engine/src/governance.rs` + `cmd/yai/src/policy.rs` | deterministic source compiler, immutable PolicyArtifact/lifecycle contracts and thin operator surface | product-reachable Case-independent governance authoring boundary; no Case authority |
| `cmd/yai/src/filesystem.rs` | read-only filesystem compatibility observation | product-reachable compatibility boundary |
| `cmd/yai/src/replay.rs` | legacy inspect/dry-run/isolated import and replay reports | product-reachable state compatibility boundary |
| `cmd/yai/src/graph_runtime.rs` | graph relation materialization, rebuild and query | product-reachable derived owner |
| `cmd/yai/src/analytics.rs` | DuckDB schemas, extraction and reports | product-reachable derived owner |
| `engine/yai-engine` | canonical Transition/CaseState semantics, LMDB authority, typed semantic-context compiler, legacy decoder, and reusable derived algorithms | product-reachable semantic/data authority |
| `cmd/yaid` + selected `system/` sources | daemon IPC, fixture loops, C journal/projection/hot snapshot | product-reachable process/platform boundary |
| separate C component archive | gates, carriers, process/observation and compatibility mechanics | component characterization; not product capability |
| tests/labs/history | current proof, research, and historical specification | evidence, never implementation authority |

There are 55 surviving headers under `include/yai`, down from 101. They are a
source compatibility surface for the product C subset and characterized
platform components; the repository does not install them. No external
consumer is known. The daemon socket JSON, JSONL record shapes, and public Rust
crate remain compatibility risks because external absence cannot be proven
from one checkout.

## Requirement/current/gap register

| Constitutional requirement | Current implementation | Gap |
|---|---|---|
| one canonical Transition Ledger with transactional CaseState | implemented in LMDB for typed payloads; provider and review are live consumers | migrate remaining current workflows; add operational checkpoint/compaction policy |
| carrier consumes an ExecutionGrant | implemented for the sole product `filesystem.write` carrier and migrated review approval; C carriers remain component-only | migrate a second resource carrier only when its lifecycle justifies the contract; no registry exists |
| PREPARE/EFFECT/FINALIZE with indeterminate recovery | implemented for local filesystem write, including prepared discovery, failpoints, reconciliation and stable idempotency identity; the Case runtime reconciles before its next invocation | automatic multi-Case background recovery, explicit abandoned issued-Grant cleanup, and non-filesystem protocols |
| filesystem attachment confinement | lexical validation plus canonical-parent containment rejects traversal and symlink-parent escape in the current single-machine tests | race-resistant directory-handle confinement for adversarial concurrent namespace mutation |
| distinct ProviderResult, Observation, EffectReceipt | implemented as separate Rust types and Transition payload roles; compatibility export still has old receipt-shaped rows | migrate process observations or other live resource families when they become product-reachable |
| Case plus materialized CaseState | implemented and replayable for provider/review/resource/operation/grant/effect refs | extend only for demonstrated future consumers; migrate daemon hot/fixture state only if it becomes canonical input |
| summary is presentation only | canonical reducers and migrated paths do not parse it; old projection/frame and analytics records use the compatibility decoder | migrate or retire remaining legacy-only producers and views |
| Projection/Residency/ContextFrame/KV separation | typed Projection, pure `yai.residency_plan.v1`, independent ContextFrame and distinct render identity are implemented; opaque continuation is optional and tokens/KV are absent from canonical state | semantic units and rendered-size estimation are conservative rather than tokenizer-authoritative; no ContextDelta consumer |
| provenance-bound operational memory | `yai.operational_memory.v1` is deterministically derived, participant-filtered before ranking, bounded, inspectable and droppable/rebuildable; the runtime repairs it automatically and current state/observed consequence retain precedence | no learned compression, embedding/reranker or global retention policy |
| agentless long-horizon execution | synchronous Case runner repeatedly consumes canonical reality, derived memory/residency and the controlled effect boundary with explicit budgets/stops, typed human pause/resume, LMDB run admission and restart tests | generalized operation families, distributed admission and daemon scheduling are absent |
| provider replacement preserves semantic continuity | real HTTP Provider A→filesystem FINALIZE→Provider B, same-provider model replacement, continuation invalidation and provider restart are deterministic product tests | generalized routing/economics and native runtime continuation protocols are deliberately absent |
| derived data rebuilds from canonical state | graph rebuild consumes typed transitions and legacy compatibility; operational memory can be dropped/rebuilt with deterministic identity and canonical fallback; facts remain legacy-derived | adaptive invalidation/scheduling, compression and full typed analytics inputs |
| governance source/artifact history | exact-byte source identity, typed deterministic parse/IR, immutable artifacts and append-only lifecycle share LMDB while remaining independent from Cases | Case PolicyBinding, effective materialization, precedence/conflict resolution, policy authority, publisher authentication and retention policy |

The remaining gaps are intentional boundaries of this wave. The
[Roadmap](../ROADMAP.md) owns their implementation sequence.
