# Current executable architecture

Authority: implementation truth. Source-refoundation baseline:
`36c93947d589519c75dd5c261fd1d4e2a0fd74d2` (the isolated
`YAI.SOURCE.REFOUNDATION.2` checkpoint). This document describes the resulting
`YAI.SOURCE.REFOUNDATION.3` worktree.

This document includes current contradictions. It does not claim that the
[Constitution](constitution.md) is implemented. Target changes and sequencing
belong only in the [Roadmap](../ROADMAP.md).

## Executable summary

YAI has two unequal product processes:

```text
operator
  |
  +-- yai (Rust command/process boundary)
  |     +-- provider/case invocation
  |     +-- controlled filesystem effect transition family
  |     +-- controlled review compatibility on the same carrier
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

[`cmd/yai/src/main.rs`](../cmd/yai/src/main.rs) is a 1,864-line command
parser, dispatcher, common CLI support surface, and compatibility shell. The
current domain implementation is grouped by demonstrated boundary in
[`provider.rs`](../cmd/yai/src/provider.rs),
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
| Case-bound provider prompt | `yai case enter` → typed participant/provider attachment → typed Invocation and ProviderResult → non-authoritative ModelInterpretation; a real OpenAI-compatible HTTP fixture proves the path while legacy records remain readable | generalized ContextFrame does not exist; ordinary prompt render/frame/thread material remains legacy-derived |
| Controlled filesystem effect | typed logical attachment + local binding → real HTTP ProviderResult → exact proposal normalization → typed Operation/Decision/ExecutionGrant → durable PREPARE → Rust atomic-replacement carrier → pre/post Observation and EffectReceipt → FINALIZE/RECONCILE → second provider turn | only `filesystem.write`; prefix policy and single-machine binding; no general review/policy/carrier system |
| Filesystem review fixture | typed ReviewRequested/ReviewResolved remains compatibility-visible; approval is normalized with explicit review origin and uses the same Grant/PREPARE/carrier/FINALIZE path | fixed review identities/path and compatibility records remain; deny/defer/quarantine retain their historical fixture model |
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
are `yai.transition.v2` and `yai.case_state.v2`; readers promote v1 state and
transitions while rejecting unknown future contracts. One bounded LMDB write
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
attachment, latest provider invocation/result/interpretation lineage, fixture
review state, logical filesystem attachments, latest Operation/Decision, Grant
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
longer historical authority. Provider and review commands commit canonical
state first and emit their old JSONL/record shapes afterward for existing
consumers.

### Derived/cache stores

- LMDB graph relations are rebuilt from typed Transitions plus the explicit
  legacy compatibility decoder; graph failure cannot affect canonical commit.
- RuntimeGraph is per-command and ephemeral.
- DuckDB facts are rebuildable analytics; historical extraction is routed
  through compatibility fields.
- Rust projection/memory/query/reconcile values are derived legacy views.
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
→ deterministic prefix Decision(ALLOW|DENY)
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

The fixed review fixture in [`review.rs`](../cmd/yai/src/review.rs) retains its
operator/compatibility surface. Approval now records an Operation with explicit
`compatibility_review` origin and passes through the same typed ALLOW, Grant,
PREPARE, carrier, Observation, Receipt, and FINALIZE boundary before committing
ReviewResolved. Deny, defer, and quarantine invoke no carrier. Historical
receipt-shaped records remain compatibility output and are not canonical
EffectReceipts.

The direct `carrier fs-write` command and Rust primitive were removed after
characterization. [`filesystem.rs`](../cmd/yai/src/filesystem.rs) now owns only
read-only compatibility observation. `yaid run-filesystem-loop` no longer
writes `output.txt` or claims an executed receipt; it produces explicit
descriptor/no-effect fixture records used by older tests.

Surviving C control, filesystem/process carrier, receipt, and observation
components contain typed or platform properties protected by component tests.
They are built separately and are not normal product call paths.

## Current provider and context behavior

[`provider.rs`](../cmd/yai/src/provider.rs) promotes legacy Case/participant
bindings through the compatibility boundary, then commits typed participant
admission, provider attachment, Invocation, ProviderResult, and
ModelInterpretation transitions. ProviderResult carries provider/model/
invocation identity and returned content; CaseState stores only current lineage
and output size. The compatibility journal still emits the former
`EffectReceipt` record so old readers and vertical assertions remain valid.

The invocation path sends a manually framed HTTP/1.x
`/v1/chat/completions` request and extracts one `content` string. Transport
endpoint replacement is allowed without changing Case or model identity; the
current attachment retains endpoint configuration but reducer closure matches
participant/provider/model. A failed HTTP invocation leaves its typed
invocation committed and invents no ProviderResult.

The controlled-effect command supplies a narrow typed participant view and an
exact operation-proposal output contract through this same real HTTP boundary.
Its first ProviderResult is normalized only after persistence. After DENY or
FINALIZE it constructs a second bounded view from typed Decision/Effect
CaseState and invokes the provider again. The deterministic fixture rejects a
second request unless it reports `observed_applied` for a proved effect or
`no_effect_authorized` for a denial. The previous model assertion is never a
source for that consequence. This is a special-purpose projection, not the
final ContextFrame/Residency implementation.

There is no formal deadline, streaming/cancellation, TLS, tokenizer, native
YVEX, or continuation contract. The ParticipantViewFrame is a summary-token
record rather than the provider-independent ContextFrame defined by the
reference. The implementation has no formal Projection identity contract,
Residency decision, ContextFrame schema, ContextDelta invalidation contract,
token identity, or KV integration. Loss/replacement of a model runtime does
not erase the JSONL Case history, but provider replacement is not yet a typed
or fully tested contract.

The context-residency lab estimates logical base/delta token reduction and
does not prove KV reuse or an implemented Context Compiler.

## Physical ownership after source refoundation

| Surface | Executable role | Classification |
|---|---|---|
| `cmd/yai/src/main.rs` | parsing, dispatch, common CLI/process initiation and residual compatibility commands | product-reachable command boundary |
| `cmd/yai/src/provider.rs` | current case admission, projection rendering, HTTP provider invocation and result residue | product-reachable transition family |
| `cmd/yai/src/controlled_effect.rs` + `engine/yai-engine/src/effect.rs` | controlled proposal/admission/recovery orchestration and the Grant-validating Rust filesystem carrier | product-reachable first constitutional effect family |
| `cmd/yai/src/review.rs` + `filesystem.rs` | review compatibility on the controlled carrier plus read-only filesystem observation | product-reachable compatibility boundary |
| `cmd/yai/src/replay.rs` | legacy inspect/dry-run/isolated import and replay reports | product-reachable state compatibility boundary |
| `cmd/yai/src/graph_runtime.rs` | graph relation materialization, rebuild and query | product-reachable derived owner |
| `cmd/yai/src/analytics.rs` | DuckDB schemas, extraction and reports | product-reachable derived owner |
| `engine/yai-engine` | canonical Transition/CaseState semantics, LMDB authority, legacy decoder, and reusable derived algorithms | product-reachable semantic/data authority |
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
| PREPARE/EFFECT/FINALIZE with indeterminate recovery | implemented for local filesystem write, including prepared discovery, failpoints, reconciliation and stable idempotency identity | automatic multi-Case recovery scheduling, explicit abandoned issued-Grant cleanup, and non-filesystem protocols |
| filesystem attachment confinement | lexical validation plus canonical-parent containment rejects traversal and symlink-parent escape in the current single-machine tests | race-resistant directory-handle confinement for adversarial concurrent namespace mutation |
| distinct ProviderResult, Observation, EffectReceipt | implemented as separate Rust types and Transition payload roles; compatibility export still has old receipt-shaped rows | migrate process observations or other live resource families when they become product-reachable |
| Case plus materialized CaseState | implemented and replayable for provider/review/resource/operation/grant/effect refs | extend only for demonstrated future consumers; migrate daemon hot/fixture state only if it becomes canonical input |
| summary is presentation only | canonical reducers and migrated paths do not parse it; old projection/frame and analytics records use the compatibility decoder | migrate or retire remaining legacy-only producers and views |
| Projection/Residency/ContextFrame/KV separation | frame/projection identity is encoded in summaries; no residency/KV contract | typed lineage and provider-independent invocation frame |
| provider replacement preserves semantic continuity | Case/participant/provider/model invocation lineage is typed; effect consequence and second turn come from canonical state, not provider continuation | normalized provider Binding lifecycle, ContextFrame, and explicit replacement proof |
| derived data rebuilds from canonical state | graph rebuild consumes typed transitions and legacy compatibility; facts remain legacy-derived | generation, invalidation, and full typed analytics inputs |

The remaining gaps are intentional boundaries of this wave. The
[Roadmap](../ROADMAP.md) owns their implementation sequence.
