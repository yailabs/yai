# Semantic reference

Authority: stable vocabulary and dispositions. Current implementation aliases
and gaps are documented here, but executable truth remains in
[Architecture](../architecture.md).

The disposition vocabulary is `ADOPT`, `COLLAPSE`, `RENAME`, `REJECT`, and
`DEFER`. Adoption defines a semantic contract; it does not require a dedicated
source subsystem.

## Ownership rule

A concept may own only an independently meaningful lifecycle, canonical
resource, transition, execution boundary, or stable multi-consumer contract.
Values, relations, phases, algorithms, views, and presentation nouns do not
become owners merely because they are important.

## Continuity and containers

### Case — ADOPT

A Case is the durable identity and lifecycle boundary of a bounded matter. It
owns lifecycle, chronology, membership, logical Bindings, policy associations,
and continuity across sessions/providers. It persists as canonical identity
and committed transition refs. It may not directly mutate CaseState, own a
physical resource/secret, or absorb all runtime data.

Current reality: the Rust authority now materializes typed Case lifecycle,
generation, participant admission, provider lineage, and fixture review state.
C daemon fixtures and remaining historical views still use legacy records.

### CaseState — ADOPT

CaseState is the transactionally maintained materialized consequence of a
Case's committed transitions at one generation. It is durable but rebuildable
from the Transition Ledger. It owns current query efficiency, not historical
authority.

### Space — REJECT

Space owns nothing in the target. Its rejected meanings include persistent
world, container above Case, shared resource universe, and runtime root. A
future independent multi-Case policy/resource/access/export/replication
lifecycle is the only current falsifier.

### Scope — ADOPT as a value

Scope is the immutable effective boundary for one transition. Its identity is a
digest of exact Case generation, participant authority, logical resources,
bindings, policy/disclosure, and constraints. It lives through
Operation→Decision→Grant→Receipt/Transition closure and then becomes history.
It owns no state or lifecycle of its own.

### Workspace, World, Session, Thread, Pack

| Term | Disposition | Canonical meaning | Rejected meaning |
|---|---|---|---|
| Workspace | RENAME | UI/local-resource alias or local Binding | durable Case/world owner |
| World | COLLAPSE | prose for external reality or the set of Cases | runtime owner/resource universe |
| Session | RENAME | disposable transport, operator, or inference-session state with explicit kind | durable continuity/truth |
| Thread | COLLAPSE | derived conversation ordering within a Case | authority or Case lifecycle |
| Pack | RENAME | versioned import/export/fixture bundle with lineage | Case, context, or runtime owner |

These old names may remain in current schemas or UI, but they cannot acquire
canonical authority through compatibility.

## Identity, participation, and resources

### Participant — ADOPT

Participant is the canonical stable identity for a human, model, service,
reviewer, external system, or delegate participating in a Case. It owns
identity only. Roles, disclosure, authority, provider/model association,
delegation, and resource access are separate Bindings or transition inputs.

`Subject` and `Actor` are RENAME aliases to Participant where they identify a
participant. A physical file/process is a Resource, not a Participant.

### Agent — REJECT as owner

Agent may be a product composition of Participant, model, role, Projection,
Bindings, and permissions. It owns no memory, tools, policy authority,
resources, execution, state, or Case continuity. Historical Agent planes and
current `consumer:agent` labels do not falsify this decision.

### Binding — ADOPT; Attachment — RENAME

A Binding is a typed, versioned relation connecting a Case or Participant to a
role, policy, provider/model, logical Resource, delegation, or machine-local
attachment reference. It has identity, provenance, activation/revocation, and
replacement lifecycle. It describes association and grants no execution by
itself.

Attachment is the machine-local or resource-association kind of Binding. Local
paths, PIDs, handles, endpoints containing secrets, and credentials do not
become portable canonical authority.

### Resource — ADOPT

Resource is a logical identity for something YAI may observe or affect. A Case
may own the logical Binding, not the real machine object. ResourceBoundary
resolves logical identity to a local attachment under a valid Grant. Current
`subject` usage often conflates participant and resource; target schemas must
not.

## Operational control

| Concept | Disposition | Canonical meaning and ownership |
|---|---|---|
| OperationCandidate | COLLAPSE | ephemeral parser/normalizer input; no durable identity unless normalization succeeds |
| Operation | ADOPT | immutable normalized requested transformation; owns exact typed intent, source refs, Scope, and parameters; never permission |
| Attempt | COLLAPSE | invocation/carrier retry metadata on Invocation, Grant, Receipt, and Transition; not a universal object |
| Policy | ADOPT | versioned rules/source material associated with a Case/system context; informs evaluation but cannot decide or execute |
| Authority | COLLAPSE | property established by identity, Binding, policy, and supporting references; not a universal object or ambient role |
| Capability | RENAME | either non-authoritative advertised ability metadata or an ExecutionGrant; descriptors never become permission |
| Decision | ADOPT | immutable authority conclusion for one Operation under exact Scope, policy/binding versions, and Case generation |
| ExecutionGrant | ADOPT | bounded, expiring, revocable carrier admission consumed exactly by ResourceBoundary |
| Review | DEFER | optional participant-mediated Decision workflow/outcome; not an owner; allow/deny remain constitutional |

The former `CapabilityLease` CLI/C views were inspect-only and were removed.
The Rust `filesystem.write` ResourceBoundary now consumes a typed
ExecutionGrant; no generic lease/capability hierarchy or second carrier exists.

## Effects and evidence

### Effect — ADOPT as external occurrence

An Effect is an occurrence in external resource reality. YAI does not make the
occurrence canonical by naming it; it makes its own prepared/finalized or
indeterminate knowledge canonical through Transitions, Observations, and an
EffectReceipt. Effect has no independent YAI storage owner.

### Observation — ADOPT

An Observation is immutable typed material produced by observing a boundary
under declared capture semantics. It owns observation identity, subject/
resource identity, method/version, time, digest/value, confidence/limitations,
and provenance. Observation is not authority and is not an EffectReceipt.

### EffectReceipt — ADOPT

EffectReceipt is the immutable carrier-attempt outcome. It owns Grant,
Operation, Decision, idempotency, actual resource, timing/error, and pre/post
Observation refs. It can report applied, already applied, no effect,
failed-no-effect, conflict, or indeterminate. It cannot authorize a later
operation. No carrier attempt means no EffectReceipt.

### Evidence — COLLAPSE to role/relation

Evidence is the role typed material plays when supporting a Decision,
Transition, interpretation, or reconciliation, plus provenance linkage. It is
not a universal object, bucket, or ontology. ProviderResult, Observation,
EffectReceipt, policy material, participant attestations, and other typed
material retain their distinct semantics when referenced as evidence.

Decision basis/supporting material is a typed reference set and evaluation
trace on a Decision. It is not automatically another stored object.

## Persistence and experience

| Concept | Disposition | Authority and lifecycle |
|---|---|---|
| Record | RENAME | versioned serialization/persistence envelope; no first-class domain ontology |
| Event | COLLAPSE | generic occurrence/source input; use a typed source object or committed Transition |
| Transition | ADOPT | append-only canonical commit fact with phase, object refs, prior/result generation, and provenance |
| Fact | ADOPT as derived | analytical/provenance-bearing assertion rebuilt from canonical transitions; never policy/execution authority |
| Graph | ADOPT as derived | causal/relation projection with generation/provenance and deterministic rebuild |
| Index | ADOPT as derived | lookup acceleration that may be dropped/rebuilt |
| Memory | ADOPT as derived | selected/compacted material for future use, sourced from canonical transitions or explicitly non-authoritative inputs |
| Experience | ADOPT as derived | episode/pattern/procedure projection from committed transitions and observed consequences |

A Record may carry an Operation, Decision, ProviderResult, Observation,
EffectReceipt, Transition, or derived artifact. It must not erase their
differences. `summary` is a generated presentation field only in the target.

Conversation history is ordered input/result material. Operational history is
the Transition Ledger. Current state is CaseState. Learned/retrieved memory is
derived. These meanings may not be collapsed into one “memory” store.

## Context and retrieval

| Concept | Disposition | Canonical meaning |
|---|---|---|
| Retrieval | ADOPT as derived algorithm | finds candidate material from qualified sources; a candidate is not evidence or truth until resolved |
| Projection | ADOPT | semantically selected, provenance-bearing view of qualified state for a consumer/task; never canonical state |
| Residency | PROVISIONAL derived decision | names what projected material remains active, re-enters, compacts, evicts, references, or is available; no independent executable owner exists yet |
| ContextFrame | ADOPT | provider-independent semantic invocation frame derived from Projection plus one task/output contract; a future Residency decision may inform selection without owning it |
| ContextDelta | DEFER | optional frame-to-frame transport optimization; no current consumer or required constitutional role |

Graph, retrieval, Projection, Residency, and ContextFrame may be implemented by
fewer source units than these semantic distinctions. Detailed identity and
invalidation rules are in [Context](context.md).

## Provider and model vocabulary

| Concept | Disposition | Canonical meaning |
|---|---|---|
| Provider | ADOPT | replaceable execution boundary/adapter target; owns no Case authority or memory |
| Model | ADOPT | model artifact/profile identity executed by a provider/runtime; owns no YAI semantic continuity |
| Invocation | ADOPT | one typed provider request with identity, lineage, ContextFrame/render refs, configuration, deadline/cancellation, and result/failure link |
| ProviderResult | ADOPT | typed invocation result/failure; canonical record of what the provider returned, while its content remains non-authoritative |
| ProviderContinuationReference | ADOPT as optional cache ref | opaque provider/runtime-specific computational continuation; invalidatable and replaceable |

ProviderResult is not Observation of an external YAI resource effect and not
EffectReceipt. A provider may return its own low-level execution evidence; YAI
retains that provenance without reclassifying it as a YAI transition.

## Current aliases and compatibility

Current compatibility names such as `EffectReceipt` for model output,
historical `ParticipantViewFrame` summary-token records, and broad `RecordKind`
values remain readable input/output reality. `MemoryCandidate` is likewise a
read-only legacy input/category; the former prompt command that appended it is
retired. New provider invocations no longer write or consume ParticipantViewFrame.
Canonical provider history uses typed
Invocation, ProviderResult, InteractionTurnRecorded, and ModelInterpretation
payloads; operational experience is `yai.operational_memory.v1` derived from
those and effect/control Transitions with mandatory provenance. The former `CaseHandle` and
`CapabilityLease` views had no product consumer and no longer survive as
compatibility types. Remaining compatibility requires a named consumer,
version policy, test, and removal/migration condition.

The exhaustive ownership, durability, producer/consumer, aliases, current
reality, target gap, documentation owner, and confidence fields are retained in
`../refoundation/doc-refoundation/semantic-conflict-matrix.tsv` relative to the
YAI repository root.
