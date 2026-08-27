# State and transition reference

Authority: stable state, transition, evidence, and effect-recovery semantics.
The [Architecture](../architecture.md) records where current code violates this
contract.

## Canonical authority

The committed Transition Ledger is YAI's sole historical authority. Each
Transition binds:

- transition identity, Case identity, phase, and commit sequence;
- prior and resulting Case generation;
- exact Operation and Decision refs;
- optional ExecutionGrant and EffectReceipt refs;
- supporting material/Observation refs and provenance;
- time, schema/version, and reconciliation lineage.

CaseState is maintained transactionally with a ledger append. It is a current
materialization and may be rebuilt; the ledger is the durable explanation of
how it arose. A single database may store both without making them equivalent.

Graph relations, indexes, memory, analytics, hot state, and participant/model/
operator views carry source generation and are rebuildable. They may be dropped
without loss of canonical history. A physical resource is external reality,
not a second YAI database authority.

## Transition forms

### No-external-effect transition

Typed input or candidate material is normalized into an Operation. YAI resolves
Case/Scope and supporting material, evaluates policy/authority, produces a
Decision, and atomically appends a terminal Transition plus resulting CaseState.

This form covers deny, invalid input, failed provider invocation, internal
state change, review/defer state, expiry, and other outcomes that do not invoke
a resource carrier. No EffectReceipt is fabricated.

```text
source material
  → normalize Operation
  → evaluate Decision
  → COMMIT terminal Transition + CaseState
```

### External-effect transition

```text
PREPARE
  atomically commit Operation + Decision + ExecutionGrant
  + PREPARED Transition + pending CaseState

EFFECT
  resolve logical Resource Binding
  → validate Grant/Scope/generation/expiry
  → observe pre-state
  → invoke carrier with idempotency key
  → observe or discover outcome

FINALIZE
  atomically commit Observations + EffectReceipt
  + FINALIZED Transition + resulting CaseState
```

PREPARE is durable intent/admission, not a claim that the effect occurred.
FINALIZE is YAI's committed knowledge of an observed outcome, not retroactive
atomicity with the external resource.

## Indeterminate outcomes and reconciliation

A timeout, lost acknowledgement, carrier crash, process crash, network split,
or reply-parse failure can leave the effect's reality unknown.

```text
PREPARED
   ↓ carrier invocation may have crossed the resource boundary
INDETERMINATE
   ↓ resource-specific reconciliation
effect_observed | no_effect_observed | still_indeterminate
```

If YAI restarts with only PREPARED knowledge, it must first determine whether
invocation could have begun. It may resume an unstarted, still-valid Grant only
when that fact is established. Otherwise it enters reconciliation. It never
maps “no acknowledgement” to “no effect” and never blindly repeats a
non-idempotent operation.

Reconciliation is a Transition activity, not a graph or background-world
owner. It validates the original idempotency identity, expected generation and
pre-state, resource-native markers where available, and new Observations. It
then commits a RECONCILED terminal transition or leaves the Case explicitly
indeterminate.

## Operation, Decision, Grant, Receipt

### Operation

An Operation persists exact normalized intent: Case, initiating Participant,
Scope digest, logical Resource/Binding, operation kind, typed parameters,
source material refs, and content/parameter digest. It may not contain ambient
paths, credentials, inferred missing targets, or permission.

An OperationCandidate is ephemeral. Parse/schema failure retains the source
ProviderResult or other input and may commit a no-effect failure transition;
it does not invent an Operation.

### Decision

A Decision is immutable and independently auditable. It binds Operation
identity/digest, Case generation, Scope digest, policy and Binding versions,
supporting references, outcome, constraints, and reason/evaluation trace.

Allow, deny, and evaluation failure are required outcomes. Review/defer/
quarantine are provisional product outcomes, not separate owners. Re-evaluation
under changed policy/state creates a new Decision with lineage; it does not
mutate the old one.

### ExecutionGrant

ExecutionGrant is the only authority a resource carrier accepts. Its fields
must have real consumers:

```text
grant_id
operation_id + operation_digest
decision_id + decision_digest
case_id + participant_id + scope_digest
logical_resource_id + allowed_effect + constraints
expected_case/resource_generation or expected pre-state
expiry + revocation reference
idempotency key
pre/post observation obligations
```

The carrier validates this envelope and narrow resource-specific preconditions;
it does not re-evaluate broad policy. Expiry, revocation, stale generation,
scope mismatch, or unresolved resource fails closed before mutation.

### EffectReceipt

EffectReceipt exists only after a carrier attempt. It binds the Grant,
Operation, Decision, actual resource, status, idempotency identity, timing/
error, and pre/post Observation refs. Status is exact and versioned:
`applied`, `already_applied`, `no_effect`, `failed`, or `indeterminate` are the
minimum meanings.

An applied claim requires adequate post-state evidence. A failed attempt may
still have partial effect and therefore may be indeterminate. A Receipt cannot
serve as a new Grant.

## Evidence as provenance role

Evidence is not a universal stored object. A Decision or Transition identifies
typed supporting references and the role each reference played. The referenced
object retains its own schema:

| Typed material | What it establishes | What it cannot establish by itself |
|---|---|---|
| ProviderResult | what a provider returned or how invocation failed | truth of the content, authority, or resource effect |
| Observation | what a declared observation method reported | policy authority or causal success without applicable limits |
| EffectReceipt | carrier-attempt outcome linked to observations | future permission or unrelated facts |
| policy/binding/source material | inputs to evaluation | the Decision without the evaluation record |
| participant attestation/claim | what a participant asserted | external reality without corroboration |

Provenance includes producer, capture method/version, time, Scope/Case,
content/digest, retention/redaction, and relevant confidence/limitations. A
shared Record envelope may serialize all these types, but does not collapse
their semantics.

## Record, summary, and serialization

`Record` is a versioned persistence/serialization envelope. It carries schema,
semantic object kind, object identity, Case/transition refs, provenance, and a
typed payload. It does not own the domain ontology.

`summary` is presentation generated from typed payload. No target behavior may
parse it for review status, resource identity, graph edges, fact fields,
thread/frame identity, admission, or replay. Historical `yai.store.record.v0`
and `yai.record.v1` require an explicit typed migration corpus; renaming
`Record` to `Evidence` would preserve the same failure under a new noun.

## Replay, rebuild, and retention

Replay:

1. verifies schema/type compatibility and stable identity;
2. verifies sequence, prior/result generation, phase transitions, and refs;
3. rejects or reports partial/corrupt histories without inventing state;
4. rebuilds CaseState deterministically;
5. optionally rebuilds derived graph/index/memory/analytics under explicit
   algorithm versions.

Derived rebuild failure cannot roll back canonical state. Queries are pure and
append no transitions.

Retention may discard caches, indexes, derived artifacts, superseded
materialized versions, and raw payload bytes under explicit privacy policy. It
must preserve stable transition identities, order/multiplicity, phase and
closure meaning, necessary digests/provenance, and reconstructible rollup/
snapshot lineage. Compaction may not erase the difference between requested,
admitted, attempted, observed, and indeterminate.

## Idempotency and retry rules

- Idempotency identity is fixed before EFFECT and is scoped to a Grant and real
  resource operation.
- A transport retry and a semantic re-evaluation are different: the first keeps
  invocation/grant lineage; the second produces a new Decision/Grant.
- Resource adapters define how to detect already-applied outcomes and which
  operations cannot be retried safely.
- Expected generation/pre-state prevents stale Grants from applying to a
  changed resource.
- Restart recovery never assumes a missing response means no application.
- Reconciliation may conclude failure or no effect only from adequate
  observation, not absence of a receipt.

## Current implementation gap

Current JSONL→LMDB dual writes, write-before-persist review behavior,
receipt-shaped denial records, direct filesystem bypass, and summary parsers do
not implement this contract. They remain characterized evidence to migrate,
not compatibility definitions. See [Architecture](../architecture.md) and the
[Roadmap](../../ROADMAP.md).
