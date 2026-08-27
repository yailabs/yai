# YAI constitution

Authority: constitutional target. This document states invariants for the next
implementation refoundation. It does not claim that the current repository
implements them; [Architecture](architecture.md) owns that truth.

## Primitive

YAI governs the admitted transformation of canonical operational state.

An admitted transformation is a typed transition accepted into YAI's canonical
history under the applicable identity, scope, policy, provenance, and state
rules. Admission to history is not permission to execute an external effect. A
denial, provider failure, internal state change, expired grant, or unresolved
effect can be a valid committed transition.

The operational transition is the primitive. Evidence closure is an invariant
of that transition: every committed claim must link the typed basis and
observations appropriate to its phase and outcome. No successful external
effect is required for a transition to be valid.

## Constitutional boundaries

### Case and CaseState

A `Case` is the durable identity and lifecycle boundary for one bounded matter
or trajectory. It owns chronology, lifecycle, participants, logical bindings,
policy associations, and continuity across client, model, provider, runtime,
and machine replacement. It may open, close, reopen, export, or clone with
explicit lineage.

`CaseState` is the materialized consequence of committed transitions for one
Case generation. It may be rebuilt. It does not turn Case into a mutable God
Object, and it cannot advance without a committed transition.

### Scope

`Scope` is the immutable effective boundary of one transition. It binds the
relevant Case generation, participant, authority, logical resource, policy,
disclosure, operation constraints, and Binding versions. A Decision and any
ExecutionGrant bind its digest. Scope has no independent open/close lifecycle
and owns no daemon, database, world, resource, or subsystem.

### Space

`Space` is rejected as a canonical owner. The set of Cases plus system policy
context and machine-local Bindings is sufficient for the demonstrated system.
Grouping, search, navigation, or a shared UI does not justify another durable
container.

The falsifier is concrete: introduce Space only if multiple Cases need a
shared durable policy/resource/participant access, authority, export/import, or
replication lifecycle that cannot be represented without overloading Case.

### Participant and Agent

A `Participant` is a stable identity participating in a Case. Roles,
delegation, model/provider association, disclosure, authority, and resource
access are separate typed Bindings or transition inputs; registration grants
none implicitly.

`Agent` is rejected as a canonical runtime owner. A product may call
`model + role + projection + bindings + permissions` an agent. That composition
owns no canonical memory, authority, resources, execution, state, tools, or
Case continuity. A future Agent owner requires an independently durable
lifecycle or transition that Participant, Binding, Case, and Transition cannot
represent.

## State authority

YAI has one canonical historical authority: the committed Transition Ledger.
Materialized current state is transactionally maintained from that ledger.
They may share one physical ACID database, but they are not ontologically equal.

```text
Committed Transition Ledger        canonical historical authority
Materialized Current CaseState     transactional materialization
Graph / Index / Memory / Analytics derived and rebuildable
Participant / Model / Operator View projection
Runtime hot state                  cache
```

The ledger cannot be replaced by a graph, current row, summary, model memory,
or analytical fact. Materialized state, indexes, graph, memory, analytics,
projections, and caches may be dropped and rebuilt subject to retention and
privacy rules. Replay validates transition identity, order, referential
closure, phase rules, and generation; it rebuilds materialized state and later
derivations. It is not copying one ambiguous store into another and calling
both canonical.

Database technology and concurrency strategy are deliberately undecided.

## Operational transition closure

Canonical semantics distinguish at least:

```text
Operation      normalized requested transformation, never permission
Decision       authority conclusion under an exact Scope and state generation
ExecutionGrant bounded, expiring admission consumed by a resource carrier
EffectReceipt  carrier-attempt outcome linked to real-resource observations
Transition     canonical commit fact and phase change
```

Policy material is not a Decision. Authority is a property established by
typed relations and basis references, not a universal token. A model proposal
is not an Operation until normalization succeeds. A Decision is not execution.
A receipt is not authority for a later effect.

Every semantic field is typed and versioned. A presentation summary may be
generated from semantic objects; no target decision, replay, graph relation,
or migration may recover meaning by parsing arbitrary summary text.

## External effects

No ordinary ACID transaction spans arbitrary filesystems, processes, devices,
or remote services. Every governed external effect therefore follows:

```text
PREPARE    durably commit Operation, Decision, Grant, expected pre-state,
           idempotency identity, and pending CaseState
EFFECT     resolve the real resource, observe pre-state, invoke the carrier,
           and observe the returned or discoverable outcome
FINALIZE   commit observations, EffectReceipt, terminal Transition, and
           resulting CaseState atomically
```

Ambiguity is first-class:

```text
PREPARED → lost reply / timeout / crash / ambiguous outcome
         → INDETERMINATE → RECONCILE
```

Reconciliation may conclude `effect_observed`, `no_effect_observed`, or
`still_indeterminate`. No acknowledgement is not evidence that no effect
occurred. Restart must not blindly repeat an ambiguous effect.

Each carrier contract consumes expected resource generation/pre-state, an
idempotency key, expiry/revocation data, and observation obligations. Retries
reuse or explicitly supersede identities according to the resource profile.
An `EffectReceipt` may report applied, already applied, no effect, failed, or
indeterminate; an applied claim requires an identified real resource and
adequate post-observation. Denial without a carrier attempt produces no fake
receipt.

## Evidence semantics

`Evidence` is primarily a role and provenance relation: typed material is used
as support for a Decision, Transition, interpretation, or reconciliation. It
is not a universal domain object or storage bucket.

The following meanings remain distinct even when a shared persistence envelope
serializes them:

- `ProviderResult` is the typed result or failure of a model invocation. Its
  bytes are non-authoritative candidate material.
- `Observation` is a typed statement produced by observing a boundary or
  resource under declared capture semantics.
- `EffectReceipt` is the typed outcome of a carrier attempt and refers to the
  relevant pre/post Observations.
- decision basis/supporting material is a set of typed references used to
  justify an authority conclusion.

Provenance states who produced material, when, by which method/version, under
which scope, and with which digest/retention posture. Evidence quality does not
turn provider prose into resource fact.

## Semantic and computational continuity

YAI owns semantic continuity: Case history, CaseState, Projection lineage,
ContextFrame lineage, participant continuity, invocation lineage, and the
meaning of ProviderResults. A provider or model runtime owns computational
continuation: tokenizer mechanics, mutable inference-session state, KV/cache,
low-level execution state, and runtime-specific execution evidence.

Therefore:

```text
Projection identity
!= rendered token-sequence identity
!= KV continuation identity

YAI semantic continuity
!= provider/YVEX computational continuation
```

Provider continuation is optional, opaque, invalidatable, and replaceable.
Losing it may increase cost or reduce computational continuity; it cannot make
Case history unreconstructible or change semantic correctness.

## Derived-state rules

Projection, Residency, ContextFrame, graph, retrieval, index, memory,
experience, analytics, hot state, and provider continuation own no historical
truth. Every derived artifact carries source generation, provenance,
algorithm/profile version, and invalidation posture. Reads are pure: building
or querying a view does not append operational history.

Real operational experience derives from committed transitions and observed
consequences, never from model prose alone. Derived failure cannot roll back a
canonical commit. Stale material must be rebuilt, bypassed with an honest
canonical fallback, or rejected; it may not be presented as fresh.

## Source minimalism

Documentation granularity does not dictate source granularity. An important
concept may need a contract without acquiring a directory, registry, service,
or process. A source owner requires an independently meaningful lifecycle,
canonical state or resource, transition, execution boundary, and stable
multi-consumer contract.

Registries require heterogeneous live consumers and selection behavior.
Public schemas or APIs require a named producer, consumer, owner, versioning
policy, and conformance test. File presence, build membership, descriptors, or
documentation phrases are never proof of capability.

## Amendment decisions

| Amendment | Decision | Constitutional consequence |
|---|---|---|
| operational transition is primitive; evidence closure is invariant | ADOPT | valid no-effect and failure transitions remain representable |
| CaseState is materialized, not a mutable Case | ADOPT | all state advance is transition-mediated |
| one historical authority even in one ACID database | ADOPT | ledger and materialization retain different authority |
| PREPARE/EFFECT/FINALIZE plus INDETERMINATE/RECONCILE | ADOPT | ambiguous outcomes survive restart honestly |
| ProviderResult, Observation, EffectReceipt remain distinct | ADOPT | no overloaded receipt/evidence object |
| Evidence is a role/provenance linkage | ADOPT | Audit 3's universal Evidence object is superseded |
| semantic continuity differs from computational continuation | ADOPT | provider replacement cannot break Case correctness |
| Projection, Residency, ContextFrame, tokens, and KV are distinct | ADOPT | identities and invalidation stay at their proper boundary |
| provider continuation is opaque and non-canonical | ADOPT | loss degrades optimization only |
| documentation concepts do not imply source owners | ADOPT | refoundation cannot repeat module-per-noun growth |

Changing these decisions requires evidence against the relevant falsifier and
an explicit constitutional supersession, not an implementation convenience.
