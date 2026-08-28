# Execution and resource boundaries

Authority: stable contracts at YAI's model/provider and external-resource
boundaries. This document does not select an inference runtime, carrier
implementation, or database.

## Boundary rule

YAI owns operational authority and semantic continuity. A provider, YVEX, or
carrier performs bounded execution and returns non-authoritative results. No
executor can commit YAI state merely by reporting success.

```text
YAI semantic continuity
!= provider/YVEX computational continuation

YVEX ExecutionEvidence
!= ProviderResult
!= YAI EffectReceipt
!= YAI committed Transition
```

These are roles at different boundaries, not a requirement for four duplicate
storage objects. One typed envelope may carry several linked roles only when
their producers, authority, and provenance remain distinguishable.

## Model/provider boundary

YAI owns:

- Case identity, lifecycle, chronology, and materialized CaseState;
- participant, Binding, policy, and disclosure continuity;
- Projection lineage, Residency decisions, and semantic ContextFrames;
- provider/model Invocation lineage and output contracts;
- selection and rendering requirements supplied to an adapter;
- interpretation of a ProviderResult as proposed, non-authoritative material;
- admission of any resulting operational Transition;
- external resource effects outside model execution.

A provider/runtime, including YVEX, owns:

- model artifact and runtime identity;
- tokenizer- and model-specific mechanics;
- request execution and mutable inference-session state;
- KV/cache allocation and computational continuation;
- low-level execution evidence, failure detail, timing, and resource use;
- runtime-specific continuation creation and validation.

The adapter is the translation boundary. It renders a ContextFrame into the
provider protocol, invokes the provider, and returns a typed ProviderResult
plus optional runtime evidence and continuation reference. Adapter behavior is
versioned and must fail explicitly when provider capabilities cannot satisfy a
required semantic or output contract.

### ProviderResult

A ProviderResult is the provider's returned content and structured status for
one Invocation. It identifies the provider, model, runtime when known, request
or render lineage, response identity, completion/failure status, and usage or
finish metadata. It may include references to low-level execution evidence and
an opaque continuation.

ProviderResult is not a fact about the world, an EffectReceipt, or a committed
Transition. YAI may interpret it into an OperationCandidate, an Observation
candidate, supporting material, or presentation output. Admission is a
separate decision recorded by a Transition.

### ExecutionEvidence

ExecutionEvidence is provider/runtime-produced information that an inference
attempt ran or failed in a particular way: runtime/model/tokenizer versions,
request digest, timing, token counts, cache behavior, failure diagnostics, and
similar facts. Its authority is limited to the execution boundary. It cannot
prove that a proposed operation was admitted or that an external resource
changed.

### ProviderContinuationReference

The continuation contract is defined in
[Context and continuity](context.md#provider-continuation). Core YAI treats the
reference as opaque and optional. Provider replacement or continuation loss
must preserve semantic correctness by rebuilding from qualified YAI state.

### YVEX

YVEX is one possible independent model runtime. It is not a YAI subsystem, and
YAI does not import its session, KV, scheduler, artifact, or execution ontology
as canonical YAI state. The same adapter contract must remain implementable by
another local runtime, an OpenAI-compatible remote endpoint, or a future
provider.

No authoritative YVEX checkout was available during this refoundation. This
contract therefore constrains only the YAI side; claims about YVEX internals
must be validated in the YVEX project before implementation.

## External-resource boundary

A Resource is an externally meaningful object with stable identity and an
observable generation or pre-state when the resource permits it. A Binding
associates that Resource with a Case or Participant under explicit lifecycle,
authority, disclosure, and retention rules. Neither a filesystem path nor an
attachment summary is sufficient canonical identity by itself.

A carrier consumes an ExecutionGrant, not an unadmitted OperationCandidate.
The grant binds the admitted operation, exact Scope, policy/decision lineage,
resource identity, expected generation/pre-state, attempt identity, expiry,
and idempotency posture. A carrier may narrow execution but may not broaden the
grant.

The external-effect lifecycle is specified in
[State and transition authority](state-transitions.md#external-effect-flow).
Its boundary consequences are:

- durable preparation precedes carrier invocation;
- retries use stable attempt/idempotency identity and revalidate grants;
- expected resource generation or pre-state prevents blind replay;
- a receipt records observed execution outcome, not canonical commitment;
- missing acknowledgement is not evidence that no effect occurred;
- ambiguous outcomes enter `INDETERMINATE` and require reconciliation;
- restart recovery enumerates prepared and indeterminate attempts before
  issuing retries;
- reconciliation observes the resource without inventing certainty.

An Observation is qualified information about resource state, produced by a
carrier, observer, reconciler, user, provider, or imported source. Its
provenance and confidence travel with it. An EffectReceipt is carrier-produced
execution outcome for an Attempt. Supporting Evidence is a relation that may
link either to a Decision or Transition; it does not collapse the two roles.

## Current implementation gap

Rust now implements one product `filesystem.write` ResourceBoundary. A Case
owns a logical attachment (identity, normalized allowed prefix, size bound,
policy owner); a separately versioned local LMDB binding maps it to one
machine-local canonical root. Operations carry only attachment identity plus
relative path. The carrier consumes the exact materialized prepared Grant,
canonicalizes the target parent against symlink escape, performs same-directory
atomic replacement with file and parent sync, and produces typed pre/post
Observations and EffectReceipt.

PREPARE, FINALIZE, INDETERMINATE, and RECONCILE are canonical Transition kinds.
Restart reconciliation compares the actual target with persisted expected
pre-state and intended post-digest, and never infers no effect from missing
acknowledgement. The fixed review approval path reuses this boundary with an
explicit review-origin Operation. The former direct `fs-write` command is
removed, and the old C daemon fixture no longer mutates `output.txt`. C
control/carrier components remain characterized test-only mechanics.

This is not a universal carrier layer: there is no carrier registry, process
carrier migration, generic policy engine, distributed binding, automatic
multi-Case recovery scheduler, or expiry/revocation service. Local absolute
bindings are restart-durable but noncanonical and single-machine.
Confinement currently validates the canonical parent immediately before the
operation; it does not claim race-resistant `openat`/directory-handle security
against a concurrently hostile namespace.

The provider path now compiles typed `yai.projection.v3` and
`yai.context_frame.v3`, renders them for raw OpenAI-compatible HTTP, and records
provider/model/frame/render lineage in `yai.transition.v3` Invocation and
ProviderResult payloads. The controlled effect and ordinary prompt paths use
the same compiler. A qualified, participant-filtered `yai.operational_memory.v1`
input is derived from canonical history and may enrich Projection; it can be
dropped/rebuilt and is never provider or Case authority. Deterministic product
tests replace provider and model, invalidate an opaque continuation, restart the
provider endpoint, and rebuild current semantics and operational experience from
CaseState/history. The opaque continuation value is ephemeral; only its
disposition is persisted.

The implementation still has no deadline/cancellation, TLS/streaming
abstraction, runtime ExecutionEvidence ingestion, native YVEX protocol,
provider-returned continuation lifecycle, or token/KV contract. The current
continuation reference is a caller-supplied OpenAI-compatible adapter extension
used to prove invalidation fallback, not a universal provider feature.

These limitations are executable truth, not exceptions to the constitutional
boundary. Their implementation delta is owned by [ROADMAP](../../ROADMAP.md).
