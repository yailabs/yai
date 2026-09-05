# MANUAL ACCEPTANCE — ZERO TO USE CASE

Run these commands from the YAI repository root. They use the canonical
`./yai` launcher and ordinary Advanced YAI commands. There is no `set -e`,
`set -u`, `pipefail`, hidden Case, ResourceAttachment, Policy, effect runtime,
private YVEX protocol or manual internal-store editing.

Build YAI and start two disposable OpenAI-compatible loopback providers:

```bash
make build-rust
./yai version

I04_RUN_ROOT=$(mktemp -d /tmp/yai-i04-manual.XXXXXX)
export YAI_HOME="$I04_RUN_ROOT/yai-home"
base64 -d tests/fixtures/conversation/i03-audio.wav.base64 >"$I04_RUN_ROOT/source.wav"

python3 tests/fixtures/provider_governance_server.py --mode full --model primary-text-only --requests 64 --log "$I04_RUN_ROOT/primary.log" >"$I04_RUN_ROOT/primary.port" 2>"$I04_RUN_ROOT/primary.err" &
PRIMARY_PID=$!
python3 tests/fixtures/provider_governance_server.py --mode full --model auxiliary-transformer --requests 64 --log "$I04_RUN_ROOT/auxiliary.log" >"$I04_RUN_ROOT/auxiliary.port" 2>"$I04_RUN_ROOT/auxiliary.err" &
AUXILIARY_PID=$!

sleep 1
PRIMARY_PORT=$(sed -n '1p' "$I04_RUN_ROOT/primary.port")
AUXILIARY_PORT=$(sed -n '1p' "$I04_RUN_ROOT/auxiliary.port")
```

Initialize a fresh Tenant, Case and cognitive Participant:

```bash
./yai init --tenant tenant:i04-manual --organization organization:i04-manual
./yai doctor
./yai identity whoami
./yai case create case:i04-manual --tenant tenant:i04-manual
./yai case participant role add case:i04-manual --participant participant:model --role model-executor
./yai case participant link-principal case:i04-manual --principal self --participant participant:model
./yai case participant view admit case:i04-manual --participant participant:model --consumer model --view model_context
./yai case participant list case:i04-manual
```

Create and qualify the exact primary and auxiliary targets. Their names do not
grant capability; the primary has text wire evidence only and the auxiliary has
WAV-to-text evidence:

```bash
PRIMARY_ADD=$(./yai provider add --tenant tenant:i04-manual --provider-key primary --endpoint "http://127.0.0.1:$PRIMARY_PORT" --model primary-text-only --credential-ref none --locality loopback)
printf '%s\n' "$PRIMARY_ADD"
PRIMARY_TARGET=$(printf '%s\n' "$PRIMARY_ADD" | sed -n 's/^target_id: //p')

AUXILIARY_ADD=$(./yai provider add --tenant tenant:i04-manual --provider-key auxiliary --endpoint "http://127.0.0.1:$AUXILIARY_PORT" --model auxiliary-transformer --credential-ref none --locality loopback)
printf '%s\n' "$AUXILIARY_ADD"
AUXILIARY_TARGET=$(printf '%s\n' "$AUXILIARY_ADD" | sed -n 's/^target_id: //p')

./yai provider qualify "$PRIMARY_TARGET" --realization-shape text_to_text
./yai provider qualify "$AUXILIARY_TARGET" --realization-shape audio_wav_to_text
./yai provider trust approve "$PRIMARY_TARGET"
./yai provider trust approve "$AUXILIARY_TARGET"
./yai case provider bind case:i04-manual --participant participant:model --target "$PRIMARY_TARGET" --target "$AUXILIARY_TARGET" --failover safe_only --max-attempts 2
```

Record honestly labelled operator-attested semantic evidence and bind the
primary and auxiliary cognitive roles:

```bash
PRIMARY_EVIDENCE_OUTPUT=$(./yai provider suitability record "$PRIMARY_TARGET" --capability primary_conversation --suite manual:i04 --run manual:primary --evidence-ref evidence:primary-conversation)
printf '%s\n' "$PRIMARY_EVIDENCE_OUTPUT"
PRIMARY_EVIDENCE=$(printf '%s\n' "$PRIMARY_EVIDENCE_OUTPUT" | sed -n 's/^evidence_id: //p')

STT_EVIDENCE_OUTPUT=$(./yai provider suitability record "$AUXILIARY_TARGET" --capability speech_to_text --suite manual:i04 --run manual:stt --evidence-ref evidence:speech-to-text)
printf '%s\n' "$STT_EVIDENCE_OUTPUT"
STT_EVIDENCE=$(printf '%s\n' "$STT_EVIDENCE_OUTPUT" | sed -n 's/^evidence_id: //p')

./yai case cognitive bind case:i04-manual --participant participant:model --role primary --capability primary_conversation --target "$PRIMARY_TARGET" --evidence "$PRIMARY_EVIDENCE"
./yai case cognitive bind case:i04-manual --participant participant:model --role auxiliary --capability speech_to_text --target "$AUXILIARY_TARGET" --evidence "$STT_EVIDENCE"
./yai case cognitive show case:i04-manual --participant participant:model --json
```

Commit the immutable ordered `text + audio + text` Turn, then inspect it and
read the audio part identity from normal YAI output:

```bash
./yai case conversation draft create case:i04-manual audio-conversation --participant participant:model
./yai case conversation draft add-text case:i04-manual audio-conversation --text 'Context before the audio'
./yai case conversation draft import case:i04-manual audio-conversation "$I04_RUN_ROOT/source.wav" --type audio --mime audio/wav
./yai case conversation draft add-text case:i04-manual audio-conversation --text 'Context after the audio'

TURN_SEND=$(./yai case conversation draft send case:i04-manual audio-conversation)
printf '%s\n' "$TURN_SEND"
TURN_ID=$(printf '%s\n' "$TURN_SEND" | sed -n 's/^turn_id: //p')

TURN_SHOW=$(./yai case conversation turn show case:i04-manual "$TURN_ID" --participant participant:model)
printf '%s\n' "$TURN_SHOW"
AUDIO_PART_ID=$(printf '%s\n' "$TURN_SHOW" | sed -n 's/^part: .* id=\([^ ]*\) type=audio .*/\1/p')
printf 'audio_part_id: %s\n' "$AUDIO_PART_ID"
```

The first command stops deliberately after the auxiliary result has become
canonical derived content. Run the second identical composition in a fresh YAI
process: it reuses that prerequisite, constructs the source closure and invokes
the primary without a second auxiliary request.

```bash
./yai case cognitive compose case:i04-manual --participant participant:model --goal primary_conversation --turn "$TURN_ID" --prerequisite speech_to_text --prerequisite-part "$AUDIO_PART_ID" --failpoint after-prerequisite --json

./yai case cognitive derived show case:i04-manual --participant participant:model --json

COMPOSED_RESULT=$(./yai case cognitive compose case:i04-manual --participant participant:model --goal primary_conversation --turn "$TURN_ID" --prerequisite speech_to_text --prerequisite-part "$AUDIO_PART_ID" --json)
printf '%s\n' "$COMPOSED_RESULT"
./yai case conversation turn show case:i04-manual "$TURN_ID" --participant participant:model
```

Repeat the complete command. The same composition request/source-closure
identities are reconstructed, and both prior provider results are reused:

```bash
./yai case cognitive compose case:i04-manual --participant participant:model --goal primary_conversation --turn "$TURN_ID" --prerequisite speech_to_text --prerequisite-part "$AUDIO_PART_ID" --json
sed -n '1,$p' "$I04_RUN_ROOT/auxiliary.log"
sed -n '1,$p' "$I04_RUN_ROOT/primary.log"
```

Exercise the semantic-intent negative path. WAV MIME alone does not authorize
speech-to-text, so omitting the explicit prerequisite fails before auxiliary
execution:

```bash
./yai case cognitive compose case:i04-manual --participant participant:model --goal primary_conversation --turn "$TURN_ID" --json
```

Reopen the same `YAI_HOME` through new CLI processes and inspect the stable
Turn, derived provenance, cognitive bindings and canonical Case:

```bash
./yai doctor
./yai case show case:i04-manual --json
./yai case conversation turn show case:i04-manual "$TURN_ID" --participant participant:model --json
./yai case cognitive derived show case:i04-manual --participant participant:model --json
./yai case cognitive show case:i04-manual --participant participant:model --json
./yai case provider show case:i04-manual
```

Stop fixtures and remove only the disposable acceptance root:

```bash
kill "$PRIMARY_PID" "$AUXILIARY_PID"
wait "$PRIMARY_PID"
wait "$AUXILIARY_PID"
unset YAI_HOME
rm -rf "$I04_RUN_ROOT"
printf 'I04 manual acceptance completed; disposable state removed\n'
```
