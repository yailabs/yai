# I04 failure evidence

## I04-F01 — media does not create intent

- Run ID: `i04-composition-smoke-20260905`
- Command: `make smoke-cognitive-execution-composition`
- Suite exit: 0; challenged operation nonzero as required
- Raw output:

```text
{"schema":"yai.cli.error.v1","operation_id":"yai.case.cognitive.compose","status":"error","code":"operation_failed","message":"cognitive_composition_prerequisite_required"}
```

The Turn contains valid WAV content and a misleading primary model name, but
the primary lacks the exact mechanical audio shape. With no explicit semantic
prerequisite, composition refuses without an auxiliary dispatch.

## I04-F02 — delivery-indeterminate prerequisite blocks dependents

- Run ID: `i04-composition-smoke-20260905`
- Command: `make smoke-cognitive-execution-composition`
- Suite exit: 0; challenged operation nonzero as required
- Raw output:

```text
{"schema":"yai.cli.error.v1","operation_id":"yai.case.cognitive.compose","status":"error","code":"provider_unavailable","message":"cognitive_realization_failed:delivery=DeliveryIndeterminate:derived_content=false:provider_delivery_indeterminate:partial_headers:bytes=19590"}
i04_failure: delivery_indeterminate_blocks_primary=true malformed_blocks_primary=true normalization_blocks_primary=true unresolved_blocks_primary=true derived_duplicates=0
```

The auxiliary request may have reached the remote provider but has no
authoritative result. I04 publishes no derived content and the primary provider
dispatch count remains unchanged.

## I04-F03 — prerequisite/publication restart boundary

- Run ID: `i04-composition-smoke-20260905`
- Command: `make smoke-cognitive-execution-composition`
- Suite exit: 0; deliberate failpoint operation nonzero as required
- Observed posture:

```text
i04_composed: auxiliary_dispatch_once=true restart_resume=true primary_dispatch=true turn_immutable=true audio_and_image=true
```

The first process stopped after canonical derived-content publication and
before the primary. A new process reconstructed the compatible derivation and
source closure, dispatched no second auxiliary request, and completed the
primary.

## I04-F04 — sandbox-only loopback refusal

The first unprivileged characterization attempt could not bind local fixture
sockets (`Operation not permitted`). The identical test with the repository's
loopback permission passed. This is environment posture, not a YAI defect and
does not alter the provider evidence.

## I04-F05 — malformed, non-normalizable and unresolved prerequisites

- Run ID: `i04-composition-focused-final-20260906`
- Command: `tests/characterization/cognitive-execution-composition/test_cognitive_execution_composition.sh`
- Suite exit: 0; all three challenged operations nonzero as required
- Raw output:

```text
{"schema":"yai.cli.error.v1","operation_id":"yai.case.cognitive.compose","status":"error","code":"provider_unavailable","message":"cognitive_realization_failed:delivery=ResponseInvalid:derived_content=false:provider_response_invalid:status=200:bytes=8919:provider response was not valid JSON: EOF while parsing a list at line 1 column 12"}
{"schema":"yai.cli.error.v1","operation_id":"yai.case.cognitive.compose","status":"error","code":"provider_unavailable","message":"conversation_provider_derived_text_invalid"}
{"schema":"yai.cli.error.v1","operation_id":"yai.case.cognitive.compose","status":"error","code":"operation_failed","message":"cognitive_composition_prerequisite_not_auxiliary:Unresolved"}
```

An invalid wire result, a durably recorded but empty result that fails the
normalizer, and an unresolved I02 prerequisite all block the dependent primary
dispatch. Neither failing Case contains a canonical derived-content relation.
