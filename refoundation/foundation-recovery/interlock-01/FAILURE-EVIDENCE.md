# I01 failure evidence

All items below were directly observed in source, review, or execution. No
output is reconstructed as a test transcript.

## F-I01-001 — provider success defined the only typed interaction

- source baseline: `398def1e7391b736dd58280cff6f29e96248635b`
- inspected files: `cmd/yai/src/provider.rs`,
  `engine/yai-engine/src/transition.rs`, and
  `engine/yai-engine/src/context.rs`
- reproduction: direct `git show` of the baseline
- result: `append_interaction_turn(...)` was called only after
  `append_model_output_receipt(...)`; `InteractionTurnRecorded` required both
  `invocation_id` and `result_id`; Projection held one `operator_input: String`
- invariant violated: a submitted user Turn could not exist independently of a
  successful provider result and could not preserve ordered multipart content
- correction: v13 `ConversationTurnCommitted` is a prior canonical event;
  invocation/result are later causal lineage. The old payload remains readable.

Verbatim source excerpt from the inspected baseline:

```text
let result_id = append_model_output_receipt(...)?;
...
if let Err(error) = append_interaction_turn(
    &session,
    &invocation.attempt_id,
    &invocation.invocation_id,
    &result_id,
    task,
    &transport.output,
) {
```

## F-I01-002 — initial draft preview crossed SEND prematurely

- phase: I01 implementation review before publication
- initial behavior: `derive-text` called `publish_draft` to calculate the source
  part ID, which copied otherwise uncommitted draft bytes into the immutable
  object namespace
- impact: no Transition was created, but object adoption occurred before the
  declared SEND boundary and discarded drafts could leave avoidable objects
- correction: `preview_draft` now derives identical content/part IDs without
  publishing; only SEND calls `publish_draft`
- retained proof:
  `identical_draft_labels_are_namespaced_by_case_and_preview_does_not_adopt_bytes`

## F-I01-003 — failed provider invocation has no completed context selector

- run: `i01-smoke-projection-expectation-20260904`
- command: I01 smoke with an extra
  `yai case context show ... --kind projection` assertion after connecting to
  deliberately unavailable `127.0.0.1:9`
- actual bounded runtime output:

```text
provider_retry: 1 reason:provider_not_dispatched:connect:Connection refused
case_runtime_schema: yai.case_runtime_checkpoint.v3
runtime_status: ProviderFailureBudgetExhausted
projection_id: none
context_frame_id: none
```

- classification: the `case context show latest` selector intentionally follows
  completed runtime artifact lineage; it is not proof that the already
  canonical Turn was lost
- correction to qualification: the provider-failure smoke asserts the Turn and
  owned content survive. A separate pure test proves the same Turn compiles to
  Projection v7 and ContextFrame v7 without ProviderResult lineage.

## F-I01-004 — sandbox-only CLI socket denial

- command: `make build-rust` inside the restricted filesystem/network sandbox
- engine result: passed
- CLI result: two existing provider-transport tests failed to bind loopback
  sockets with `Operation not permitted`
- classification: qualification-environment restriction, not a YAI defect
- correction: the exact CLI suite was rerun outside that sandbox and passed
  `35 passed; 0 failed; 2 ignored`.

## F-I01-005 — new smoke lacked executable mode

- command: `make smoke-multipart-conversation`
- output:

```text
make: tests/characterization/multipart-conversation/test_multipart_conversation.sh: Permission denied
make: *** [Makefile:665: smoke-multipart-conversation] Error 127
```

- correction: the versioned test file was marked executable; its direct rerun
  then emitted all four `pass` results.

## F-I01-006 — storage output change weakened one smoke matcher

- phase: complete staged-diff review
- initial behavior: the human Turn output gained an explicit `storage=` field,
  but the duplicate-image assertion still searched for `object=... provenance=`
- impact: the extraction returned an empty line whose unique-line count was
  accidentally one, so that single assertion could pass without observing the
  object ID
- correction: the matcher now binds `object=... storage=` and the direct smoke
  was rerun successfully as `i01-smoke-corrected-20260904`
- boundary: product object/part identity was already independently covered by
  the Rust unit test; this was a qualification defect and is retained rather
  than hidden

## F-I01-007 — Turn scope needed a canonical Case/Tenant comparison

- phase: final adversarial source review before publication
- initial behavior: `ConversationTurn::validate()` proved internal agreement
  among Turn and object Tenant labels, while the secured Transition commit
  authenticated the Case and participant but did not compare the Turn Tenant
  label to the canonical Case security-domain Tenant
- impact: a directly constructed internally self-consistent Turn could carry a
  foreign Tenant label into a real Case; the CLI path supplied the correct
  Tenant, so this required bypassing the CLI constructor but violated the core
  boundary contract
- correction: secured LMDB commit now resolves the canonical Case Tenant and
  rejects any mismatch before append
- retained proof:
  `i01_conversation_turn_tenant_must_match_canonical_case_security_domain`

## F-I01-008 — derivation kind and actor class were underconstrained

- phase: final adversarial source review before publication
- initial behavior: actor/result pairs were validated locally, but a caller
  could label a provider-produced transformation as `Human` or attach a
  ProviderResult to `HumanEdit`
- impact: bytes remained immutable and scoped, but provenance semantics could
  be mislabeled by a non-CLI constructor
- correction: `HumanEdit` now requires the submitting Principal and forbids a
  ProviderResult; other derivations cannot claim a Human actor; Provider actors
  require ProviderResult lineage and Deterministic actors forbid it
- retained proof: forged actor/result combinations are rejected by
  `original_machine_transcript_and_human_edit_are_distinct`

## Suspected attacks already safe

- A derived part naming a source outside its same Turn/Case is rejected by
  source-set and scope validation.
- A directly constructed Turn carrying a Tenant other than its canonical Case
  Tenant is rejected at the authenticated store boundary.
- A derivation cannot change the actor class implied by its exact provenance.
- Equal payload bytes across Cases or Tenants do not share object identity.
- A draft directory replaced with a symlink is refused by descriptor-relative
  no-symlink open and does not touch the outside target.
- Altered immutable bytes fail digest verification and never alter canonical
  Transition content.
- A non-text Turn sent to the current text-only adapter fails explicitly rather
  than being silently converted or partially delivered.
