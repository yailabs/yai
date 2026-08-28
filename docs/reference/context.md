# Context and continuity reference

Authority: stable semantics for qualified state selection and model-facing
context. This document defines distinctions, not source subsystems.

## Pipeline

The conceptual pipeline is:

```text
CaseState
  → Query / Resolve
  → Graph / Retrieval
  → Projection
  → Residency
  → ContextFrame
  → Render
  → Tokenize
  → Provider / YVEX
```

Each arrow is a semantic transformation with provenance and failure behavior.
It need not be a process, directory, registry, database, or separately persisted
object.

## CaseState, query, graph, and retrieval

CaseState is transactionally maintained current state. Query/Resolve obtains
typed objects and Bindings at a known generation. Graph and retrieval are
derived selection aids: graph restores relationships and causal reachability;
retrieval proposes candidate material from qualified sources. Neither owns
facts absent from the Transition Ledger/CaseState or a clearly labeled
non-authoritative source.

Retrieval candidates must be resolved to typed source material with Case,
Scope, provenance, disclosure, freshness, and retention posture before they
can enter a Projection. External retrieval does not become Case history without
an explicit import transition.

### Operational derived memory

Operational memory is a compact, typed derivation of committed history used to
improve later semantic selection. It is neither CaseState nor another fact
authority. A trusted operational entry cites its source Transitions and, where
applicable, Observation and EffectReceipt identities. Provider-originated
material may be retained only with an explicit non-authoritative posture.

The executable v1 contract derives five bounded kinds: resource effect,
Decision, unresolved effect, normalization failure, and provider claim. Each
entry binds Case, derivation version, generation range, participant visibility,
typed value, provenance and active/superseded lifecycle. Current CaseState and
finalized observed consequence outrank derived memory; derived memory outranks
provider claims only for current-world assertions. This is an anti-confusion
rule, not a universal epistemic ordering.

The derived store is disposable. Derivation follows canonical commit, may fail
without affecting Transition/CaseState, and is deterministic for the same
history/version. Rebuild never appends a Transition. A newer finalized resource
state supersedes an older current-state entry without deleting history; a
terminal/reconciled effect supersedes its prepared or indeterminate entry.

Qualified retrieval accepts Case and generation, participant/admitted view,
purpose, optional resource/kind/causal constraints, supersession posture and a
hard result budget. It filters authority boundaries before deterministic
ranking. Its `RetrievalSet` is ephemeral derived output with selected entries,
scores/reasons, rejection counts and omission counts. Missing memory or graph
falls back to direct CaseState/canonical selection.

## Projection

Projection is a semantically selected view of qualified state for an exact
consumer, task, and disclosure policy. Projection owns selection meaning and
lineage, not canonical state.

A Projection identity is determined by at least:

```text
projection_id
case_id + source_generation
consumer/participant + task contract
scope/disclosure-policy digest
ordered included semantic refs and omission/redaction manifest
selection algorithm/profile version
freshness/invalidation posture
```

Serialization or caching of a Projection is optional. Its identity must not be
derived from provider rendering, token IDs, or KV/cache identity. Semantically
equivalent renderings may share a Projection identity; a changed selection or
omission boundary must not.

Projection is not a prompt, context window, transcript, graph, memory store, or
permission. It may use all of those as qualified inputs without acquiring their
authority.

## Residency — PROVISIONAL

Residency is a derived decision about which projected material should remain
active, be reintroduced, be summarized/compacted, be evicted, be referenced,
or be made available for a particular invocation.

Conceptually:

```text
Residency = f(
  Projection,
  previous residency,
  task,
  model profile,
  semantic budget,
  runtime constraints
)
```

Residency owns no memory or history. It has no authority to omit required
material silently. Its output includes included/evicted/reintroduced refs,
compaction provenance, reasons, budget/profile version, and invalidation
posture. It may be recomputed or cached.

Residency is not provider KV residency. YAI may decide that a semantic item
remains active while a provider rebuilds all tokens/KV, or may change semantic
residency while a runtime continuation must be invalidated.

No independent Residency object, scheduler, or database exists in the current
repository. Waves 4–5 implement deterministic bounded selection in qualified
retrieval and the Projection compiler, but do not reason about prior residency,
provider cost or semantic reintroduction/eviction. Residency remains a named
optimization boundary until that additional consumer/lifecycle is implemented.

## ContextFrame — ADOPT

ContextFrame has an independent semantic identity as the provider-independent
invocation frame for one task. It is derived and non-authoritative, but it is
the stable boundary between YAI semantic context construction and
provider/model-specific rendering.

Minimum content:

```text
context_frame_id
source_projection_id + source_case_generation
residency decision/profile and previous-frame lineage
consumer/participant and task/instruction material
selected semantic sections with typed refs and provenance
output contract
scope/disclosure and omission/redaction manifest
invocation constraints and semantic budget
freshness/invalidation posture
optional opaque prior-continuation hint (never identity)
```

The frame owns no world state and need not embed every source payload. It may
contain stable refs plus selected inline material. Persistence is normally
limited to invocation lineage, digests, and retention-required content; it does
not become canonical memory because it was serialized.

ContextFrame identity changes when semantically relevant content, order,
instructions, output contract, consumer/task, disclosure, or source generation
changes. Provider formatting-only changes may produce a new Render identity
without changing ContextFrame identity.

## ContextDelta — DEFER

ContextDelta is not constitutionally required. The current repository and lab
have no consumer that applies a typed delta to a prior frame and proves semantic
equivalence. Incremental context sounding useful is insufficient reason to add
an object or subsystem.

If a real incremental consumer appears, a ContextDelta contract must identify:

```text
delta_id
source_context_frame_id
destination_context_frame_id
ordered additions/removals/replacements with provenance
consumer + application semantics
source/destination Projection and Case generation
invalidation causes and fallback behavior
equivalence digest/test against full destination-frame rebuild
```

A delta is derived transport/computation, not memory. It is invalid if source
frame, consumer, disclosure, task/output contract, relevant Binding/policy,
model profile assumptions, or source generation no longer match. Consumers
must be able to discard it and rebuild the destination frame.

Until those conditions exist, `ContextDelta` remains deferred vocabulary and
must not appear as an implemented capability.

## Render

Render transforms provider-independent semantic sections into an adapter-
specific request representation.

YAI owns semantic ordering/priority, task/output contract, provenance and
omission requirements, and the requirement that the adapter preserve them. The
provider adapter owns protocol/model-specific message roles, templates,
escaping/serialization, special tokens, supported feature mapping, and request
bytes. The render result has its own digest, adapter/protocol/template version,
and ContextFrame ref.

Changing an adapter template can change rendered bytes without changing the
Projection or ContextFrame. A lossy adapter that cannot satisfy required
sections/output constraints fails explicitly; it may not silently reinterpret
the frame.

## Tokenization and budget

Model/tokenizer-specific tokenization belongs at or below the inference
boundary. YAI may use an explicit model profile or conservative estimator to
plan a semantic budget, but estimates are labeled and cannot define semantic
identity. The provider/runtime reports authoritative tokenizer/model/version
and actual input identity when available.

```text
Projection identity
!= ContextFrame identity
!= Render identity
!= token-sequence identity
!= provider continuation / KV identity
```

The distinctions permit one ContextFrame to render differently across
providers and allow provider replacement without changing Case continuity.

## Provider continuation

`ProviderContinuationReference` is an optional opaque reference returned or
accepted by one provider/runtime adapter. It may refer to a mutable inference
session, cached prefix, KV pages, prompt-cache key, or another runtime-specific
optimization. YAI records only the provider/runtime/model/tokenizer binding,
opaque value/digest as policy permits, source Render/ContextFrame lineage,
creation/expiry, and invalidation status.

It is:

- opaque to core YAI;
- provider-, runtime-, model-, and tokenizer-specific;
- optional, invalidatable, replaceable, and non-portable by default;
- never canonical memory, Projection identity, ContextFrame identity, or Case
  identity;
- never required to reconstruct Case history or semantic context.

Loss or invalidation forces re-render/re-tokenize/re-prefill. It may harm
latency/cost but not semantic correctness. A provider continuation may be
reused only when the adapter validates all relevant frame/render/policy/model/
tokenizer/runtime assumptions.

## Invalidation

Projection/Residency/ContextFrame dependencies include source Case generation,
typed source refs, graph/retrieval versions, participant/Binding/policy/
disclosure versions, task/output contract, model profile constraints, and
adapter assumptions. A source change invalidates only affected derived
material, but freshness must be explicit.

Provider continuation has stricter invalidation: any change that can alter the
rendered prefix or runtime interpretation invalidates reuse even if semantic
Projection identity remains stable. Provider/runtime failure can also
invalidate continuation without invalidating ContextFrame.

## Current repository reality

Current Rust code implements `yai.projection.v2` and `yai.context_frame.v2` in
one pure compiler. Projection binds exact Case generation,
participant/purpose/admitted view, typed entries, authority posture,
Transition/Observation/Receipt/derived-memory provenance, deterministic bounds,
retrieval identity/counts and explicit omission count. It is rebuilt from
CaseState and ordered Transitions plus an optional qualified RetrievalSet.
Provider claims remain labeled non-authoritative.

ContextFrame has independent identity because a Projection can feed multiple
tasks/output contracts. The Wave-3 `filesystem.write` proposal schema is a
typed output contract in the frame. The OpenAI-compatible adapter produces a
separate `yai.rendered_input.v2` identity/digest and wire body. Invocation and
ProviderResult transitions identify their Projection, frame, Case generation,
render, provider, model and output contract explicitly.

Bounded Projection/ContextFrame values and render metadata are retained in a
droppable derived LMDB database so `yai context inspect` can show lineage. Full
rendered input is not retained. Clearing that database is tested not to change
Transition history or CaseState. New provider invocations neither write nor
consume legacy `ParticipantViewFrame`; historical records remain readable only
through compatibility surfaces.

`yai.operational_memory.v1` and
`yai.operational_memory.derivation.v1` are owned by the single Rust
`memory.rs` algorithm boundary. Two clearly derived LMDB databases hold entries
and a Case generation manifest. They can be cleared/rebuilt without changing
CaseState or ledger count. The active provider compiler refreshes stale/missing
memory, runs Case/participant/purpose-qualified retrieval with an eight-entry
default, and passes only the selected typed material to Projection. Derivation
or store failure uses canonical fallback. The former `/memory propose` command
is retired; historical `MemoryCandidate` remains compatibility input only.

The provider adapter accepts an optional opaque, provider/runtime-bound
continuation reference in memory for one invocation. It persists only the use/
invalidation disposition. An invalid-continuation response retries the same
complete frame without the reference. Product tests replace provider and model
identity after a real controlled effect, restart the provider fixture, and show
that the next frame contains the current observed consequence plus selected
operational memory carrying Transition, Observation and Receipt provenance.

The implementation has no Residency object, embedding/vector retrieval,
learned ranking, semantic compression/promotion, ContextDelta consumer,
authoritative tokenizer, token IDs, KV integration, or native YVEX protocol.
The context-residency lab remains research evidence, not runtime capability.

Historical E07 workset/provider-frame code supports the distinction between a
qualified semantic workset and provider rendering, but its directory/type
system is evidence, not target structure. No available file or Git history
material named `AN-01` was found during this refoundation; no decision depends
on inaccessible research.
