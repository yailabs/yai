# MANUAL ACCEPTANCE — ZERO TO USE CASE

This is an operator walkthrough, not a Bash assertion harness. Run the commands
in order from the YAI repository root. Between the setup and cleanup blocks,
every action and inspection is a normal registry-backed `yai` command. There is
no `set -eu`, `test`, Python, `curl`, `grep`, `sed`, shell parsing, or generated
ID substitution.

## Prerequisites

- Linux with `openat2`, Rust/Cargo, and `make`;
- the YAI repository at the published I01 commit;
- local fixture files already shipped at
  `tests/fixtures/conversation/i01-image-one.svg`,
  `tests/fixtures/conversation/i01-image-two.svg`, and
  `tests/fixtures/conversation/i01-audio.fixture`;
- TCP port 9 on loopback is expected to refuse connections. It is used only to
  demonstrate a downstream provider failure after SEND; no YVEX multimodal
  endpoint is required for I01.

## 1. Clean isolated setup

These are the only preparatory shell commands. The two fixed paths are
disposable and do not touch operator production state.

```bash
rm -rf /tmp/yai-i01-manual-acceptance
rm -rf /tmp/yai-i01-manual-resource
mkdir -p /tmp/yai-i01-manual-resource/allowed
export YAI_HOME=/tmp/yai-i01-manual-acceptance
export NO_COLOR=1
make build-rust
export PATH="$PWD/target/debug:$PATH"
```

## 2. Initialize YAI, Tenant, Case, Principal, and Participant

```bash
yai init --tenant tenant:i01-acceptance --organization organization:cli-product
yai doctor
yai identity whoami
yai case create case:i01-acceptance --tenant tenant:i01-acceptance
yai case participant role add case:i01-acceptance --participant participant:operator --role model-executor
yai case participant link-principal case:i01-acceptance --principal self --participant participant:operator
yai case participant view admit case:i01-acceptance --participant participant:operator --consumer model --view model_context
yai case participant list case:i01-acceptance
```

Expected posture: `doctor` reports the isolated home ready, `identity whoami`
shows the authenticated local Principal, and the Participant listing shows the
role, Principal link, and admitted model view.

## 3. Commit a text-only Turn through the new contract

```bash
yai case conversation draft create case:i01-acceptance text-turn --participant participant:operator --thread thread:i01-main
yai case conversation draft add-text case:i01-acceptance text-turn --text "I01 text-only canonical conversation input."
yai case conversation draft show case:i01-acceptance text-turn
yai case conversation draft send case:i01-acceptance text-turn
yai case conversation turn show case:i01-acceptance latest --participant participant:operator
```

Expected important output: before SEND, `canonical: no`; after SEND,
`canonical: yes`, `ordered_parts: 1`, `type=text`, `provenance=original`,
`provider_execution_started: no`, and `content_integrity: verified`.

## 4. Prepare and SEND ordered multipart content

```bash
yai case conversation draft create case:i01-acceptance multipart-turn --participant participant:operator --thread thread:i01-main
yai case conversation draft add-text case:i01-acceptance multipart-turn --text "Ordered multipart input begins here."
yai case conversation draft import case:i01-acceptance multipart-turn "$PWD/tests/fixtures/conversation/i01-image-one.svg" --type image --mime image/svg+xml
yai case conversation draft import case:i01-acceptance multipart-turn "$PWD/tests/fixtures/conversation/i01-image-two.svg" --type image --mime image/svg+xml
yai case conversation draft import case:i01-acceptance multipart-turn "$PWD/tests/fixtures/conversation/i01-audio.fixture" --type audio --mime audio/x-yai-fixture
yai case conversation draft derive-text case:i01-acceptance multipart-turn --source-part 3 --kind speech-transcription --producer-ref fixture:i01-speech-transcriber --text "Machine transcript: progetto ORCHID-I01, valore 4188."
yai case conversation draft derive-text case:i01-acceptance multipart-turn --source-part 4 --kind human-edit --text "Human-edited transcript: progetto ORCHID-I01, valore finale 4188."
yai case conversation draft show case:i01-acceptance multipart-turn
yai case conversation draft send case:i01-acceptance multipart-turn
yai case conversation turn show case:i01-acceptance latest --participant participant:operator
yai case conversation turn show case:i01-acceptance latest --participant participant:operator --json
yai case conversation turn list case:i01-acceptance --participant participant:operator
```

Expected important output: the committed Turn contains six parts in exact
ordinal order: text, image, image, audio, derived text, human-edited text. The
two images remain two Turn positions. The audio is `original`; the transcript
cites audio part 3 and is `machine_or_deterministic_derived`; the edit cites
transcript part 4 and is `human_edited_derived`. Every object shows MIME, byte
length, digest, storage identity, and verified integrity; no binary bytes are
printed. The fixture producer is explicit and is not represented as a real
provider result.

## 5. Configure a bounded downstream failure path

```bash
yai case resource attach filesystem case:i01-acceptance --resource resource:i01-acceptance --root /tmp/yai-i01-manual-resource --allow-prefix allowed --policy-owner participant:operator --max-bytes 4096
yai policy ingest tests/fixtures/cli-product-policy.json --tenant tenant:i01-acceptance --validate --publish --reason "I01 downstream failure acceptance"
yai case policy bind case:i01-acceptance --policy-key cli.product.governed --reason "I01 downstream failure acceptance"
yai case provider attach case:i01-acceptance --participant participant:operator --endpoint http://127.0.0.1:9/v1 --model unavailable-i01 --provider provider:i01-unavailable
yai case policy show case:i01-acceptance
yai case provider show case:i01-acceptance
yai case resource list case:i01-acceptance
```

Expected posture: Policy and Resource are admitted and the provider binding is
explicitly the deliberately unavailable loopback target.

## 6. Commit before provider failure, then inspect survival

```bash
yai case conversation draft create case:i01-acceptance failure-turn --participant participant:operator --thread thread:i01-main
yai case conversation draft add-text case:i01-acceptance failure-turn --text "This canonical Turn must survive downstream provider failure."
yai case conversation draft send case:i01-acceptance failure-turn
yai case run case:i01-acceptance --participant participant:operator --resource resource:i01-acceptance --input-turn latest --max-invocations 1 --max-operations 1 --max-runtime-ms 2000
yai case conversation turn show case:i01-acceptance latest --participant participant:operator
yai case conversation turn list case:i01-acceptance --participant participant:operator
yai case show case:i01-acceptance
```

Expected negative-path output: the run stops with a bounded provider-failure
posture and reports the exact `input_conversation_turn_id`. The subsequent Turn
inspection still reports `canonical: yes` and `content_integrity: verified`.
No ProviderResult is required for that Turn identity.

## 7. Reopen from a new process and prove stable identity/order

Every command below starts a fresh CLI process and reopens the durable stores.

```bash
yai doctor
yai case show case:i01-acceptance --json
yai case conversation turn list case:i01-acceptance --participant participant:operator --json
yai case conversation turn show case:i01-acceptance latest --participant participant:operator --json
```

Expected posture: the same Turn, part, object, digest, ordinal, and derivation
IDs are present after reopen. `CaseState` remains compact; bulk binary media is
not embedded in it.

## 8. Cross-Case negative path

```bash
yai case create case:i01-isolation-negative --tenant tenant:i01-acceptance
yai case participant role add case:i01-isolation-negative --participant participant:operator --role model-executor
yai case participant link-principal case:i01-isolation-negative --principal self --participant participant:operator
yai case participant view admit case:i01-isolation-negative --participant participant:operator --consumer model --view model_context
yai case conversation turn list case:i01-isolation-negative --participant participant:operator
```

Required result: `multipart_turns: 0` and `legacy_text_turns: 0`. No content ID,
digest, media metadata, or derivation from `case:i01-acceptance` appears.

## 9. Cleanup

```bash
rm -rf /tmp/yai-i01-manual-acceptance
rm -rf /tmp/yai-i01-manual-resource
printf 'I01 manual acceptance completed; disposable state removed\n'
```
