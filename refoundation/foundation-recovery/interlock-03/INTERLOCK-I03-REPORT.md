# Model/provider interlock I03 report

## State

I03 closes the typed provider-realization boundary over the published I02
baseline `09030ab2064ab943ae2cf22b0258125253259c54`. A fresh I02 plan can now be
revalidated and dispatched to its exact target through the existing governed
ProviderSelection, ProviderInvocation and ProviderResult lineage. A derived
speech or image plan can publish bounded immutable text with exact source-part
and ProviderResult provenance without mutating the submitted Turn.

Intended semantic commit: `feat: add typed provider realization and auxiliary execution`.
The containing commit SHA and publication equality belong in the post-commit
handoff, not this versioned report.

## Architectural conclusion

I02 semantic suitability remains distinct from I03 mechanical realizability.
`ProviderQualification v4` records exact adapter/target evidence for the closed
initial shapes `text_to_text`, `audio_wav_to_text`, and
`ordered_png_text_to_text`. These facts come from bounded synthetic requests
through the same OpenAI-compatible transport used by realization. A model name,
provider name, MIME label, generic qualification, or semantic attestation alone
confers no wire shape.

Realization accepts an explicit cognitive capability and canonical Turn/part
selection. Immediately before dispatch it revalidates Case generation,
Principal/Participant linkage, active cognitive binding, exact semantic
evidence, unchanged provider envelope, target identity/digest, current v4
mechanical evidence, trust, credential posture and health. Exact-target
selection uses no lower-level alternate candidate, so provider failover cannot
silently substitute a different semantic target.

The adapter preserves source-part order and distinct repeated parts. It admits
bounded UTF-8 text, signature-checked PNG and signature-checked RIFF/WAVE input
for the declared shapes. Provider serialization is transport-only and never
enters canonical conversation meaning. YAI part IDs remain in local canonical
lineage and are not injected as non-standard fields into the OpenAI-compatible
wire envelope.

Native realization records the ordinary governed result on the primary lane.
Derived `speech_to_text` and `image_understanding` realization first records the
same governed result, then deterministically normalizes bounded text into the
existing immutable ConversationContentStore and commits
`yai.conversation_derived_content.v1`. That relation binds the original Turn
digest, ordered source part IDs, semantic capability, plan/lane/binding,
suitability evidence, exact target/qualification, selection, invocation,
ProviderResult, the v1 normalization contract, content object and derivation.
Provider output remains candidate material: it becomes no Observation, memory
assertion, Decision, Grant or effect.

## Crash, retry and continuation posture

A failpoint after durable ProviderResult proves restart recovery can normalize
and publish the same relation without another provider dispatch. A complete
unreferenced content object remains permitted by I01 object-first publication;
no canonical relation can reference partial or digest-divergent bytes.
Recovery reports the historical execution `plan_id` separately from the fresh
`validation_plan_id`; the latter revalidates current bindings while the former
continues to identify the exact ProviderResult lineage.

Missing wire evidence and stale plans refuse before ProviderSelection. Invalid
JSON and invalid normalized output retain any real provider result but publish
no derived relation. Delivery indeterminate publishes no derived content and is
not automatically retried. Continuations remain optional lane-and-target-scoped
hints; their absence never changes semantic planning or Case correctness.

## Authority and version delta

- Transition: v14 -> v15 for the canonical post-SEND derived-content relation;
  v1-v14 readers remain.
- ProviderQualification: v3 -> v4 for evidence-bound realization shapes;
  v1-v3 readers remain and gain no realization shape by compatibility.
- New logical schema: `yai.conversation_derived_content.v1`.
- Unchanged: CaseState v13, ConversationTurn v1, ConversationContentObject v1,
  CognitiveExecutionPlan v1, Projection v7, ContextFrame v7 and RetrievalSet v3.
- LMDB: 37/40. Semantic owner delta: +0. Operational owner delta: +0.

The Case already owns canonical causal history and now owns the minimum durable
relation. The existing ConversationContentStore still owns immutable bytes.
Provider-governance storage still owns qualifications. Plans, lanes and
continuations remain noncanonical, and no database, model-runtime owner, lane
store, media store or provider session owner was added.

## Product and compatibility

Two Advanced registry-backed operations realize a plan and inspect derived
content. They use stable operation IDs, typed deterministic JSON, bounded human
output and registry help/completion. The current text provider path, I01 Turns,
I02 planning, Case runtime, W19/H19 retrieval and W20 memory remain compatible.
No `yai chat`, Replia, Studio, streaming redesign or private YVEX protocol was
introduced.

## External boundary

The observed YVEX public `main` remote at closure preparation was
`3f4a1c182d35e5a0e163adb81008ae7a366efcc6`; `refs/heads/models1` was not
advertised. No operator-supplied live endpoint/model variables were available,
so live YVEX typed realization was not claimed. The generic YAI boundary is
qualified through real loopback OpenAI-compatible dispatch. Public YVEX typed
media realization remains an external adapter qualification, not a YAI defect
or permission to consume a private protocol.

## Closure posture

The internally provable I03 contract is complete: fresh semantic plan -> exact
mechanical evidence -> exact governed dispatch -> ProviderResult, and for a
derived plan, exact immutable source -> auxiliary dispatch -> immutable derived
content -> exact provenance. Production STT/vision targets, public YVEX
multipart interoperability, automatic auxiliary-to-primary orchestration,
dynamic cognitive arbitration and non-text generated output remain post-I03.
