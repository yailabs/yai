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

## Current implementation and target boundary

The current executable resource boundary has progressed beyond the original
single filesystem-write carrier: Golden admits bounded filesystem reads/search,
discovery and immutable admission, confined process execution, SQLite reads,
ordinary HTTP reads and a bounded MCP client. Resource effects retain current
Decision/Grant, fenced PREPARE, typed Observation/Receipt and
FINALIZE/INDETERMINATE/reconciliation. Neither a Resource catalog nor a model
request grants authority. These are qualified verticals, not universal adapters.
[Architecture](../architecture.md) owns exact current contracts and limits;
[ZERO-TO-CURRENT](../zero-to-current.md) owns cumulative product acceptance.

I01–I06 govern canonical content, intent, exact cognitive targets/lanes and
provider realization. TLS, deadlines, credentials and shared circuit posture
exist; streaming/transport abort and a public native YVEX state contract are not
claimed. Provider continuation remains optional computational evidence, never
Case authority. Single-host runtime scheduling and resource fencing do not prove
distributed state coordination.

The adopted [semantic/computational-state target](../semantic-state-execution-target.md)
assigns semantic continuity/compilation/admission to YAI and model-native state
lowering/Read/Update/physical execution to YVEX. It is not an implemented public
protocol. YAI may require truthful capabilities, not branch on architecture
families or inspect private engines. Computational state must remain derived and
replaceable relative to admitted semantic state.

[ROADMAP](../../ROADMAP.md) alone owns live maturity, selected implementation
pressure and promotion. Its target architecture does not change these current
resource, provider or authority contracts.
