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

## Residency

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

Current Rust code renders summary-only participant views from journal records,
persists a `ParticipantViewFrame` summary-token record, and sends manually
framed OpenAI-compatible HTTP. The context-residency lab compares full case
context, lexical retrieval, and a logical base/delta estimate. It does not
implement the identities or contracts above, does not use an authoritative
tokenizer, and does not demonstrate KV continuation.

The controlled filesystem vertical also renders two narrow typed views: an
initial Case/resource/output-contract view and a post-Decision/effect
consequence view. The second provider invocation is executable proof that an
applied consequence or denial comes from typed CaseState/Transition refs, not
the first ProviderResult. These views are purpose-specific serialization and
do not yet have independent Projection or ContextFrame identity, Residency,
general invalidation, or provider-specific render objects.

Historical E07 workset/provider-frame code supports the distinction between a
qualified semantic workset and provider rendering, but its directory/type
system is evidence, not target structure. No available file or Git history
material named `AN-01` was found during this refoundation; no decision depends
on inaccessible research.
