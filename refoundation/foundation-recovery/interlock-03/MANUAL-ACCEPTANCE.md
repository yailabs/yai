# MANUAL ACCEPTANCE — ZERO TO USE CASE

Run the commands below from the YAI repository root. They use `./yai`, the
repository’s canonical launcher, and deterministic loopback OpenAI-compatible
fixtures. No external credential, YVEX administration, ResourceAttachment,
Policy, effect runtime, or private provider protocol is involved in this
provider-realization acceptance.

Build YAI and prepare disposable source files and provider processes:

```bash
make build-rust
./yai version

I03_RUN_ROOT=$(mktemp -d /tmp/yai-i03-manual.XXXXXX)
export YAI_HOME="$I03_RUN_ROOT/yai-home"

base64 -d tests/fixtures/conversation/i03-audio.wav.base64 >"$I03_RUN_ROOT/source.wav"
base64 -d tests/fixtures/conversation/i03-image.png.base64 >"$I03_RUN_ROOT/source.png"

python3 tests/fixtures/provider_governance_server.py --mode full --model misleading-primary-name --requests 64 --log "$I03_RUN_ROOT/primary.log" >"$I03_RUN_ROOT/primary.port" 2>"$I03_RUN_ROOT/primary.err" &
PRIMARY_PID=$!
python3 tests/fixtures/provider_governance_server.py --mode full --model generic-auxiliary-name --requests 64 --log "$I03_RUN_ROOT/auxiliary.log" >"$I03_RUN_ROOT/auxiliary.port" 2>"$I03_RUN_ROOT/auxiliary.err" &
AUXILIARY_PID=$!
python3 tests/fixtures/provider_governance_server.py --mode full --model semantic-without-wire-proof --requests 64 --log "$I03_RUN_ROOT/no-wire.log" >"$I03_RUN_ROOT/no-wire.port" 2>"$I03_RUN_ROOT/no-wire.err" &
NO_WIRE_PID=$!

sleep 1
PRIMARY_PORT=$(sed -n '1p' "$I03_RUN_ROOT/primary.port")
AUXILIARY_PORT=$(sed -n '1p' "$I03_RUN_ROOT/auxiliary.port")
NO_WIRE_PORT=$(sed -n '1p' "$I03_RUN_ROOT/no-wire.port")
```

Initialize a fresh Tenant, inspect the Principal, and create the Case and
Participant:

```bash
./yai init --tenant tenant:i03-manual --organization organization:i03-manual
./yai doctor
./yai identity whoami
./yai case create case:i03-manual --tenant tenant:i03-manual
./yai case participant role add case:i03-manual --participant participant:model --role model-executor
./yai case participant link-principal case:i03-manual --principal self --participant participant:model
./yai case participant view admit case:i03-manual --participant participant:model --consumer model --view model_context
./yai case participant list case:i03-manual
```

Create exact targets. Mechanical realization is admitted only by the actual
typed synthetic probes requested on `provider qualify`:

```bash
PRIMARY_ADD=$(./yai provider add --tenant tenant:i03-manual --provider-key primary --endpoint "http://127.0.0.1:$PRIMARY_PORT" --model misleading-primary-name --credential-ref none --locality loopback)
printf '%s\n' "$PRIMARY_ADD"
PRIMARY_TARGET=$(printf '%s\n' "$PRIMARY_ADD" | sed -n 's/^target_id: //p')

AUXILIARY_ADD=$(./yai provider add --tenant tenant:i03-manual --provider-key auxiliary --endpoint "http://127.0.0.1:$AUXILIARY_PORT" --model generic-auxiliary-name --credential-ref none --locality loopback)
printf '%s\n' "$AUXILIARY_ADD"
AUXILIARY_TARGET=$(printf '%s\n' "$AUXILIARY_ADD" | sed -n 's/^target_id: //p')

NO_WIRE_ADD=$(./yai provider add --tenant tenant:i03-manual --provider-key no-wire --endpoint "http://127.0.0.1:$NO_WIRE_PORT" --model semantic-without-wire-proof --credential-ref none --locality loopback)
printf '%s\n' "$NO_WIRE_ADD"
NO_WIRE_TARGET=$(printf '%s\n' "$NO_WIRE_ADD" | sed -n 's/^target_id: //p')

./yai provider qualify "$PRIMARY_TARGET" --realization-shape text_to_text
./yai provider qualify "$AUXILIARY_TARGET" --realization-shape audio_wav_to_text --realization-shape ordered_png_text_to_text
./yai provider qualify "$NO_WIRE_TARGET"
./yai provider trust approve "$PRIMARY_TARGET"
./yai provider trust approve "$AUXILIARY_TARGET"
./yai provider trust approve "$NO_WIRE_TARGET"
./yai case provider bind case:i03-manual --participant participant:model --target "$PRIMARY_TARGET" --target "$AUXILIARY_TARGET" --target "$NO_WIRE_TARGET" --failover safe_only --max-attempts 3
```

Record honestly labelled operator-attested semantic evidence and bind the
primary and auxiliary cognitive roles:

```bash
PRIMARY_EVIDENCE_OUTPUT=$(./yai provider suitability record "$PRIMARY_TARGET" --capability primary_conversation --suite manual:i03 --run manual:primary --evidence-ref evidence:primary-conversation)
printf '%s\n' "$PRIMARY_EVIDENCE_OUTPUT"
PRIMARY_EVIDENCE=$(printf '%s\n' "$PRIMARY_EVIDENCE_OUTPUT" | sed -n 's/^evidence_id: //p')

STT_EVIDENCE_OUTPUT=$(./yai provider suitability record "$AUXILIARY_TARGET" --capability speech_to_text --suite manual:i03 --run manual:stt --evidence-ref evidence:speech-to-text)
printf '%s\n' "$STT_EVIDENCE_OUTPUT"
STT_EVIDENCE=$(printf '%s\n' "$STT_EVIDENCE_OUTPUT" | sed -n 's/^evidence_id: //p')

IMAGE_EVIDENCE_OUTPUT=$(./yai provider suitability record "$AUXILIARY_TARGET" --capability image_understanding --suite manual:i03 --run manual:image --evidence-ref evidence:image-understanding)
printf '%s\n' "$IMAGE_EVIDENCE_OUTPUT"
IMAGE_EVIDENCE=$(printf '%s\n' "$IMAGE_EVIDENCE_OUTPUT" | sed -n 's/^evidence_id: //p')

NO_WIRE_EVIDENCE_OUTPUT=$(./yai provider suitability record "$NO_WIRE_TARGET" --capability speech_to_text --suite manual:i03 --run manual:no-wire --evidence-ref evidence:semantic-only)
printf '%s\n' "$NO_WIRE_EVIDENCE_OUTPUT"
NO_WIRE_EVIDENCE=$(printf '%s\n' "$NO_WIRE_EVIDENCE_OUTPUT" | sed -n 's/^evidence_id: //p')

./yai case cognitive bind case:i03-manual --participant participant:model --role primary --capability primary_conversation --target "$PRIMARY_TARGET" --evidence "$PRIMARY_EVIDENCE"
./yai case cognitive bind case:i03-manual --participant participant:model --role auxiliary --capability speech_to_text --target "$AUXILIARY_TARGET" --evidence "$STT_EVIDENCE"
./yai case cognitive bind case:i03-manual --participant participant:model --role auxiliary --capability image_understanding --target "$AUXILIARY_TARGET" --evidence "$IMAGE_EVIDENCE"
./yai case cognitive show case:i03-manual --participant participant:model --json
```

Commit a text Turn before execution and realize its native primary plan:

```bash
./yai case conversation draft create case:i03-manual native-text --participant participant:model
./yai case conversation draft add-text case:i03-manual native-text --text 'I03 native conversation input'
NATIVE_SEND=$(./yai case conversation draft send case:i03-manual native-text)
printf '%s\n' "$NATIVE_SEND"
NATIVE_TURN=$(printf '%s\n' "$NATIVE_SEND" | sed -n 's/^turn_id: //p')

./yai case cognitive realize case:i03-manual --participant participant:model --capability primary_conversation --turn "$NATIVE_TURN" --json
./yai case conversation turn show case:i03-manual "$NATIVE_TURN" --participant participant:model --json
```

Commit original audio, simulate a crash after the durable ProviderResult, then
rerun the same command. The first command reports the deliberate failpoint; the
second reports `recovered_without_dispatch` and publishes a separate immutable
transcript without changing the Turn:

```bash
./yai case conversation draft create case:i03-manual audio-turn --participant participant:model
./yai case conversation draft import case:i03-manual audio-turn "$I03_RUN_ROOT/source.wav" --type audio --mime audio/wav
AUDIO_SEND=$(./yai case conversation draft send case:i03-manual audio-turn)
printf '%s\n' "$AUDIO_SEND"
AUDIO_TURN=$(printf '%s\n' "$AUDIO_SEND" | sed -n 's/^turn_id: //p')

./yai case conversation turn show case:i03-manual "$AUDIO_TURN" --participant participant:model --json
./yai case cognitive realize case:i03-manual --participant participant:model --capability speech_to_text --turn "$AUDIO_TURN" --failpoint after-provider-result --json
./yai case cognitive realize case:i03-manual --participant participant:model --capability speech_to_text --turn "$AUDIO_TURN" --json
./yai case cognitive derived show case:i03-manual --participant participant:model --json
./yai case conversation turn show case:i03-manual "$AUDIO_TURN" --participant participant:model --json
```

Commit ordered `text + image + image + text`, realize image understanding, and
inspect both source-part ordering and provider request evidence. Equal image
bytes retain two distinct part positions and IDs:

```bash
./yai case conversation draft create case:i03-manual image-turn --participant participant:model
./yai case conversation draft add-text case:i03-manual image-turn --text 'first ordered text'
./yai case conversation draft import case:i03-manual image-turn "$I03_RUN_ROOT/source.png" --type image --mime image/png
./yai case conversation draft import case:i03-manual image-turn "$I03_RUN_ROOT/source.png" --type image --mime image/png
./yai case conversation draft add-text case:i03-manual image-turn --text 'last ordered text'
IMAGE_SEND=$(./yai case conversation draft send case:i03-manual image-turn)
printf '%s\n' "$IMAGE_SEND"
IMAGE_TURN=$(printf '%s\n' "$IMAGE_SEND" | sed -n 's/^turn_id: //p')

./yai case cognitive realize case:i03-manual --participant participant:model --capability image_understanding --turn "$IMAGE_TURN" --json
./yai case conversation turn show case:i03-manual "$IMAGE_TURN" --participant participant:model --json
./yai case cognitive derived show case:i03-manual --participant participant:model --json
sed -n '$p' "$I03_RUN_ROOT/auxiliary.log"
```

Exercise the no-dispatch negative path. The target has semantic suitability but
no `audio_wav_to_text` mechanical evidence, so realization fails closed before
a provider selection or HTTP request:

```bash
./yai case cognitive bind case:i03-manual --participant participant:model --role auxiliary --capability speech_to_text --target "$NO_WIRE_TARGET" --evidence "$NO_WIRE_EVIDENCE" --replace
./yai case cognitive realize case:i03-manual --participant participant:model --capability speech_to_text --turn "$AUDIO_TURN" --json
sed -n '$p' "$I03_RUN_ROOT/no-wire.log"
```

Reopen the same `YAI_HOME` in fresh CLI processes and inspect canonical and
immutable state. The derived IDs, source part IDs, ProviderResult IDs, object
digests and Turn ordering remain the same:

```bash
./yai doctor
./yai case show case:i03-manual --json
./yai case conversation turn show case:i03-manual "$AUDIO_TURN" --participant participant:model --json
./yai case conversation turn show case:i03-manual "$IMAGE_TURN" --participant participant:model --json
./yai case cognitive derived show case:i03-manual --participant participant:model --json
./yai case provider show case:i03-manual
```

Stop the fixtures and remove only the disposable acceptance root:

```bash
kill "$PRIMARY_PID" "$AUXILIARY_PID" "$NO_WIRE_PID"
wait "$PRIMARY_PID"
wait "$AUXILIARY_PID"
wait "$NO_WIRE_PID"
unset YAI_HOME
rm -rf "$I03_RUN_ROOT"
printf 'I03 manual acceptance completed; disposable state removed\n'
```
