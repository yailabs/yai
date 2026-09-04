# Model execution interlock I01 — multipart conversation content

State at report creation: `implemented_qualified_prepublication`.

Known W20 anchor: `d12edd22cbffae3b31c6e7d38ce344ee7b3571d6`.
Reconciled implementation baseline: `398def1e7391b736dd58280cff6f29e96248635b`
on `master`. The one intervening commit, `fix: make W20 acceptance use natural
YAI commands`, was inspected directly and preserved. Intended commit:
`feat: add Case-owned multipart conversation content`.

## Architectural conclusion

The old current path treated operator input as one `String` and wrote
`InteractionTurnRecorded` only after a provider result existed. I01 makes that
shape compatibility behavior rather than a requirement. A new
`ConversationTurnCommitted` transition canonically records one submitted,
ordered multipart Turn before cognition begins. Later
`ProviderInvocationStarted` transitions may cite its Turn ID; provider success
is neither part of Turn identity nor a prerequisite for submission.

The Case/Transition ledger owns the Turn and its immutable references. The new
`ConversationContentStore` owns original bytes because those bytes are
non-reconstructible semantic input. It is the one justified new durable
application-content owner. It is not an Agent, provider, model, UI,
ResourceAttachment, memory plane, or operational authority. Binary bytes stay
outside LMDB and CaseState.

One Turn contains an explicit ordered `ContentPart[]`. Each part refers to a
typed immutable Text, Image, Audio, Video, or File object. Equal bytes imported
twice may share an integrity-correct object identity while retaining two
different part identities and ordinals. Part order participates in the Turn
digest and survives replay/reopen.

Original, derived, and human-edited content are separate immutable objects.
`ContentDerivation` binds exact source part IDs, a generic transformation kind,
actor class/ref, and an exact same-Case/same-Participant ProviderResult when a
provider produced the output. A human edit must be attributed to the
authenticated submitting Principal, cannot cite a ProviderResult, and cannot
overwrite the machine/deterministic output or original source. Provider and
deterministic derivations have disjoint result-lineage rules. No Whisper,
YVEX, or model-family name appears in the ontology.

## Draft and SEND boundary

Drafts are mutable, Case-namespaced application state and are not canonical.
Text/import/derivation operations update only a private draft. Identity preview
does not publish draft bytes. SEND publishes and verifies complete immutable
objects, commits one v13 Transition with exact expected generation, then
discards the draft. A crash before Transition commit can leave only a complete
unreferenced object; a Transition never points at a partially published file.
Failure after SEND cannot erase the committed Turn.

The operator import command reads only an explicit local path supplied by the
authenticated CLI user. It is not exposed to model output and does not add a
model-usable `filesystem.read` capability.

## Storage and security

Physical version: `yai.conversation_content_store.v1`, rooted at
`$YAI_HOME/conversation-content-v1`.

- Linux opens the managed root and all children through descriptor-anchored
  `openat2` with `RESOLVE_BENEATH`, `RESOLVE_NO_SYMLINKS`, `O_NOFOLLOW`, and
  type/owner/mode/link-count validation.
- Store directories are owner-only for mutation and reject group/other-write;
  created directories/files use `0700`/`0600`.
- The secured Transition commit re-resolves the canonical Case Tenant and
  rejects Turn, Participant, or Principal scope that does not match the
  authenticated Case security domain.
- Reads inspect exact metadata size before allocation, remain bounded, and
  recheck size and SHA-256 after reading.
- Objects admit at most 16 MiB each; a Turn admits at most 32 parts and 64 MiB
  total; text admits at most 64 KiB; draft JSON admits 512 KiB; one derivation
  admits at most 16 sources.
- Component files are fsynced, their temporary object directory is fsynced,
  the directory is atomically renamed, and the objects parent is fsynced before
  the object is accepted.
- Non-Linux mutable/import operations fail closed because the strong
  descriptor-relative contract is not implemented there.
- MIME is bounded, syntax/modality-checked declared metadata. I01 does not
  decode content or claim magic-byte MIME verification.

An attacker able to mutate storage as the same OS Principal can cause detected
content loss/unavailability. An altered payload cannot pass its digest/object
identity and cannot become new Transition truth.

## Projection, context, provider, and memory

Projection v7 contains typed conversation entries with each part's ordinal,
modality, MIME, size, digest, optional bounded text, source part refs, and
original/machine-or-deterministic/human-edit posture. ContextFrame v7 retains
that structure and explicitly says that submitted text and transcripts are
application material, not Observation, EffectReceipt, Decision, Grant,
Resource, or semantic-memory authority.

The existing text provider adapter may consume a text-only committed Turn via
`yai case run --input-turn`. The runtime verifies every owned object, resolves
the moving `latest` alias to an exact ID in its v3 checkpoint, and places that
ID in ProviderInvocation causality. It refuses a media-containing Turn with
`conversation_turn_requires_typed_media_provider_adapter`; I01 does not flatten
media or silently select a transcript. Native modality versus auxiliary
derivation is deliberately I02 routing work.

Existing `--prompt` Case runs and historical `InteractionTurnRecorded` readers
remain supported. When the new Turn path is used, a successful provider call
does not append a second legacy interaction as competing conversation truth.

W20 remains a positive control. New Turn transitions are structurally visible
to Episode derivation but conversation objects are not automatically
OperationalMemory or SemanticAssertions. W20 currently does not connect a new
Turn episode to later provider execution through causal refs; that explicit
multi-family integration is recorded for I02/H20 rather than smuggled into
I01.

## Schemas, DBs, and owners

New logical schemas:

- `yai.conversation_content_object.v1`
- `yai.conversation_content_part.v1`
- `yai.content_derivation.v1`
- `yai.conversation_draft.v1` (non-canonical)
- `yai.conversation_turn.v1`
- `yai.conversation_content_store.v1` (physical owner contract)

Advanced schemas:

- Transition `v12 → v13`
- Projection `v6 → v7`
- ContextFrame `v6 → v7`
- RenderedInput metadata `v6 → v7`
- Case runtime checkpoint `v2 → v3`

Unchanged: CaseState v12, RetrievalSet v3, ProviderQualification v3, W20
episodic/semantic schemas, and H19 derived-memory store v2. Historical readers
remain. LMDB remains `37 / 40`; binary content adds no LMDB database.

Owner delta:

- durable application-content owners: `+1` (immutable original bytes);
- canonical Case owners: `+0` (the existing Transition ledger owns Turns);
- semantic-memory owners: `+0`;
- operational/execution owners: `+0`.

## CLI product surface

Registry-backed commands cover draft create/add/import/derive/show/discard/send
and Turn list/show. Human output exposes IDs, order, MIME, byte length, digest,
storage reference, provenance and integrity without binary dumps. Native JSON
is typed and ANSI-free. `case run` accepts exactly one of the historical
`--prompt` input or new `--input-turn` reference.

## YVEX INTERLOCK REQUIREMENT

Remote `models1` observed during I01 was
`c3f675d1213ce3a6d7387179bb22415775e40e37`, equal to the clean read-only local
reference. Directly inspected files were `docs/openai-compatibility.md`,
`include/yvex/provider.h`, `src/provider/core.c`,
`include/yvex/internal/multimodal.h`, and the adjacent OpenAI/provider tests.
`yvex.openai.compat.v2` and provider schema/wire v4 accept bounded text byte
spans, structured JSON, explicit reasoning, and function calls; the OpenAI
profile explicitly refuses multimodal content. Lower YVEX media execution
facts do not constitute an application multipart input contract.

I01 hard-codes no YVEX endpoint, ABI, engine, residency, ModelSet, or model
family. The binding status is `awaiting_parallel_provider_contract`. I02 needs
an adapter to map ordered typed YAI parts to an exact mechanically qualified
target, preserve per-part identity/provenance correlation, and return typed
results/failures. YVEX remains the owner of model catalog, engine generations,
residency, leases, placement, compute sessions, and execution evidence.

## YVEX EXTERNAL FINDINGS

`YAI_EXTERNAL_PROVIDER_BASE_URL` and `YAI_EXTERNAL_PROVIDER_MODEL` were absent
from the qualification environment, so no live YAI↔YVEX invocation was
attempted or claimed. Classification: `DEPLOYMENT_LIMITATION`. No new YVEX
defect was inferred. The read-only contract inspection found the expected
parallel-boundary gap: current OpenAI compatibility refuses multimodal input,
so live typed-media binding remains `awaiting_parallel_provider_contract`.

## Scope held

H20, W21/H21, W22/H22, cognitive-role routing, operational capabilities,
CaseBlueprint, Studio, capture devices, codecs, OCR/STT engines, and YVEX source
changes were not started. The root README was not modified.

## Closure

```text
multipart_conversation:
    DONE
immutable_content_ownership:
    DONE
turn_submission_semantics:
    DONE
original_derived_provenance:
    DONE
provider_independent_content:
    DONE
backward_compatibility:
    DONE
yvex_interlock_preparation:
    DONE

model_execution_interlock_i01:
    COMPLETE
yai_conversation_content_foundation:
    true
downstream_safe:
    true_for_i02_cognitive_bindings_and_execution_lanes
progression_decision:
    RETURN_TO_USER
```
