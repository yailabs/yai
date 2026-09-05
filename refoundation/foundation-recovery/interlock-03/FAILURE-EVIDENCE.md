# I03 failure evidence

## I03-F01 — semantic evidence without wire evidence

- Run ID: `i03-realization-smoke-20260905`
- Command: `make smoke-typed-provider-realization`
- Exit: suite 0; challenged CLI operation nonzero as required
- Raw output:

```text
{"schema":"yai.cli.error.v1","operation_id":"yai.case.cognitive.realize","status":"error","code":"operation_failed","message":"cognitive_realization_shape_not_qualified"}
```

The fixture target carried exact `speech_to_text` semantic suitability but no
`audio_wav_to_text` evidence. Its request log contained only the qualification
probe and no actual realization dispatch.

## I03-F02 — malformed and non-normalizable results

- Run ID: `i03-realization-smoke-20260905`
- Command: `make smoke-typed-provider-realization`
- Exit: suite 0; challenged CLI operations nonzero as required
- Raw output:

```text
{"schema":"yai.cli.error.v1","operation_id":"yai.case.cognitive.realize","status":"error","code":"provider_unavailable","message":"cognitive_realization_failed:delivery=ResponseInvalid:derived_content=false:provider_response_invalid:status=200:bytes=17127:provider response was not valid JSON: EOF while parsing a list at line 1 column 12"}
{"schema":"yai.cli.error.v1","operation_id":"yai.case.cognitive.realize","status":"error","code":"provider_unavailable","message":"conversation_provider_derived_text_invalid"}
```

Both real dispatches retain truthful result/failure lineage and produce no
derived-content relation.

## I03-F03 — delivery indeterminate is not retried

- Run ID: `i03-realization-smoke-20260905`
- Command: `make smoke-typed-provider-realization`
- Exit: suite 0; challenged CLI operations nonzero as required
- Raw output:

```text
{"schema":"yai.cli.error.v1","operation_id":"yai.case.cognitive.realize","status":"error","code":"provider_unavailable","message":"cognitive_realization_failed:delivery=DeliveryIndeterminate:derived_content=false:provider_delivery_indeterminate:partial_headers:bytes=14795"}
{"schema":"yai.cli.error.v1","operation_id":"yai.case.cognitive.realize","status":"error","code":"operation_failed","message":"cognitive_realization_existing_attempt_not_retry_safe:DeliveryIndeterminate"}
```

The first operation observed possible delivery without authoritative output;
the second refused automatic repetition. Neither published derived content.

## I03-F04 — result/publication crash boundary

- Run ID: `i03-realization-smoke-20260905`
- Command: `make smoke-typed-provider-realization`
- Exit: suite 0; deliberate failpoint operation nonzero as required
- Observed posture:

```text
i03_crash_recovery: result_recorded=true reinvocation=false turn_immutable=true
```

The failpoint stopped after the canonical ProviderResult and before the derived
relation. A fresh process completed deterministic normalization from that
recorded result with no second provider request and no Turn mutation.

## Environment-only negative result

An initial non-escalated smoke run reached all 283 engine tests but two existing
CLI transport tests could not bind loopback sockets (`Operation not permitted`).
The identical command under the repository's required loopback permission
passed 36/36 non-ignored CLI tests. This was sandbox posture, not a YAI defect.

## I03-F05 — persisted v14 schema-marker upgrade regression

- Run ID: `i03-make-check-first-20260905`
- Command: `make check`
- Exit: 2
- Raw output:

```text
yai: record store import failed after journal write remained at build/tmp/new12/daemon-3260173/filesystem/journal.jsonl: unsupported_persisted_schema: meta:canonical_transition_schema expected=yai.transition.v15 actual=yai.transition.v14
make: *** [Makefile:558: smoke-controlled-effect] Error 2
```

The v15 schema advance retained the v14 Transition reader but initially omitted
v14 from the LMDB schema-marker upgrade allowlist. Adding that exact historical
version restored existing stores without weakening rejection of unknown future
schemas. The final complete check reruns this path.

## I03-F06 — canonical capability/shape/source composition gap

- Discovery: final pre-publication source audit
- Affected boundary: `ConversationDerivedContent::validate`

The first implementation validated the cognitive capability, qualified wire
shape and source part references independently, but did not compose them. An
internal caller could therefore attempt a structurally valid speech
transcription relation over text or image source parts. The canonical validator
now binds `speech_to_text` to one WAV source and `image_understanding` to an
ordered text/PNG set. The focused regression reports:

```text
test conversation::tests::provider_derived_content_binds_capability_shape_and_source_modality ... ok
```

This was a real pre-publication source defect found by adversarial review; no
synthetic failing command transcript is claimed.

## I03-F07 — non-standard IDs in the OpenAI-compatible envelope

- Discovery: final pre-publication wire-contract audit
- Affected boundary: typed provider serialization

The first adapter draft emitted `yai_source_part_id` inside provider content
objects even though the qualification probe did not and the public compatible
wire contract does not admit that extension. The fields were removed. Source
identity remains correlated by canonical ordered part lineage, while the final
loopback request-log assertion proves `unexpected_yai_fields` is empty. This
was corrected before publication and is not reported as an external-provider
failure.

## I03-F08 — cognitive binding swap between selection and dispatch

- Discovery: final pre-publication concurrency audit
- Affected boundary: atomic `ProviderInvocationStarted` admission

Exact selection revalidated the I02 plan, but invocation admission originally
rechecked only the provider-governance envelope. A cognitive binding replacement
between those commits could leave the old selection mechanically dispatchable.
The exact selection now carries its realization shape as causal evidence, and
invocation admission atomically revalidates the current cognitive binding,
semantic evidence and v4 shape. Focused output:

```text
i03_exact_selection: ... substituted=false stale_plan=rejected stale_binding_dispatch=rejected
```

## I03-F09 — derived-content inspection lacked Participant linkage

- Discovery: final pre-publication authorization audit
- Affected boundary: `yai case cognitive derived show`

The first inspection adapter proved Tenant access and Participant existence but
did not also require the authenticated Principal↔Participant link. Execution
eventually checked that link in the cognitive planner, but initially loaded Turn
history and content bytes before reaching that fence. Both adapters now reject
an unlinked Participant before reading conversation history/content. The smoke
requires the corresponding typed Principal mismatch failures and retains the
summary `participant_isolation=true`.

## I03-F10 — verified-then-reopened provider source bytes

- Discovery: final pre-publication source-to-wire audit
- Affected boundary: `ConversationContentStore::read_bytes`

The initial I03 reader called `verify_object` and then reopened the object path
to obtain provider input. That second open needlessly separated the bytes
verified by digest from the bytes used by the adapter. The store now reads
payload and metadata through one descriptor-anchored object directory, verifies
length/digest/metadata, and returns those exact verified bytes. No cryptographic
claim is made against an attacker already controlling the same OS Principal.

## I03-F11 — engine selector accepted an incoherent semantic/wire pair

- Discovery: staged-diff boundary inspection
- Affected boundary: `select_case_provider_exact_authorized`

The CLI derived the proper mechanical shape from the explicit semantic
capability, but the public engine selector initially trusted its two typed
arguments independently. A non-CLI caller could request an image wire shape
for a speech-to-text plan. The engine now admits only the closed v1 mapping and
refuses before ProviderSelection. Final focused output:

```text
i03_exact_selection: ... capability_shape_mismatch=rejected ...
```
