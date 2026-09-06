#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
YAI_BIN="$ROOT/yai"
RUN_ROOT="$(mktemp -d)"
PIDS=()

cleanup() {
  for pid in "${PIDS[@]}"; do
    kill "$pid" 2>/dev/null || true
    wait "$pid" 2>/dev/null || true
  done
  rm -rf "$RUN_ROOT"
}
trap cleanup EXIT

start_provider() {
  local name="$1"
  local mode="$2"
  local model="$3"
  python3 "$ROOT/tests/fixtures/provider_governance_server.py" \
    --mode "$mode" --model "$model" --requests 128 \
    --log "$RUN_ROOT/$name.log" \
    >"$RUN_ROOT/$name.port" 2>"$RUN_ROOT/$name.err" &
  PIDS+=("$!")
}

start_provider primary full whisper-primary-is-only-a-name
start_provider auxiliary full generic-auxiliary-is-not-whisper
start_provider no_wire full whisper-without-wire-evidence
start_provider malformed malformed_realization valid-name-malformed-result
start_provider empty empty_realization text-output-normalizes-empty
start_provider drop drop_realization delivery-indeterminate-model

for _ in $(seq 1 100); do
  [[ -s "$RUN_ROOT/primary.port" \
    && -s "$RUN_ROOT/auxiliary.port" \
    && -s "$RUN_ROOT/no_wire.port" \
    && -s "$RUN_ROOT/malformed.port" \
    && -s "$RUN_ROOT/empty.port" \
    && -s "$RUN_ROOT/drop.port" ]] && break
  sleep 0.05
done

PRIMARY_PORT="$(head -1 "$RUN_ROOT/primary.port")"
AUXILIARY_PORT="$(head -1 "$RUN_ROOT/auxiliary.port")"
NO_WIRE_PORT="$(head -1 "$RUN_ROOT/no_wire.port")"
MALFORMED_PORT="$(head -1 "$RUN_ROOT/malformed.port")"
EMPTY_PORT="$(head -1 "$RUN_ROOT/empty.port")"
DROP_PORT="$(head -1 "$RUN_ROOT/drop.port")"
for port in "$PRIMARY_PORT" "$AUXILIARY_PORT" "$NO_WIRE_PORT" "$MALFORMED_PORT" "$EMPTY_PORT" "$DROP_PORT"; do
  [[ "$port" =~ ^[0-9]+$ ]]
done

base64 -d "$ROOT/tests/fixtures/conversation/i03-audio.wav.base64" >"$RUN_ROOT/source.wav"
base64 -d "$ROOT/tests/fixtures/conversation/i03-image.png.base64" >"$RUN_ROOT/source.png"

export YAI_HOME="$RUN_ROOT/yai-home"
"$YAI_BIN" init --tenant tenant:i03-cli --organization organization:i03 >/dev/null
"$YAI_BIN" case create case:i03-cli --tenant tenant:i03-cli >/dev/null
"$YAI_BIN" case participant role add case:i03-cli \
  --participant participant:model --role model-executor >/dev/null
"$YAI_BIN" case participant role add case:i03-cli \
  --participant participant:other --role model-executor >/dev/null
"$YAI_BIN" case participant link-principal case:i03-cli \
  --principal self --participant participant:model >/dev/null
"$YAI_BIN" case participant view admit case:i03-cli \
  --participant participant:model --consumer model --view model_context >/dev/null

add_target() {
  local key="$1"
  local port="$2"
  local model="$3"
  "$YAI_BIN" provider add --tenant tenant:i03-cli \
    --provider-key "$key" --endpoint "http://127.0.0.1:$port" \
    --model "$model" --locality loopback \
    | sed -n 's/^target_id: //p'
}

PRIMARY_TARGET="$(add_target misleading-primary "$PRIMARY_PORT" whisper-primary-is-only-a-name)"
AUXILIARY_TARGET="$(add_target generic-auxiliary "$AUXILIARY_PORT" generic-auxiliary-is-not-whisper)"
NO_WIRE_TARGET="$(add_target misleading-no-wire "$NO_WIRE_PORT" whisper-without-wire-evidence)"
MALFORMED_TARGET="$(add_target malformed-output "$MALFORMED_PORT" valid-name-malformed-result)"
EMPTY_TARGET="$(add_target empty-output "$EMPTY_PORT" text-output-normalizes-empty)"
DROP_TARGET="$(add_target indeterminate-delivery "$DROP_PORT" delivery-indeterminate-model)"

"$YAI_BIN" provider qualify "$PRIMARY_TARGET" \
  --realization-shape text_to_text >/dev/null
"$YAI_BIN" provider qualify "$AUXILIARY_TARGET" \
  --realization-shape audio_wav_to_text \
  --realization-shape ordered_png_text_to_text >/dev/null
"$YAI_BIN" provider qualify "$NO_WIRE_TARGET" >/dev/null
"$YAI_BIN" provider qualify "$MALFORMED_TARGET" \
  --realization-shape audio_wav_to_text >/dev/null
"$YAI_BIN" provider qualify "$EMPTY_TARGET" \
  --realization-shape audio_wav_to_text >/dev/null
"$YAI_BIN" provider qualify "$DROP_TARGET" \
  --realization-shape audio_wav_to_text >/dev/null
for target in "$PRIMARY_TARGET" "$AUXILIARY_TARGET" "$NO_WIRE_TARGET" "$MALFORMED_TARGET" "$EMPTY_TARGET" "$DROP_TARGET"; do
  "$YAI_BIN" provider trust approve "$target" >/dev/null
done
"$YAI_BIN" case provider bind case:i03-cli --participant participant:model \
  --target "$PRIMARY_TARGET" --target "$AUXILIARY_TARGET" \
  --target "$NO_WIRE_TARGET" --target "$MALFORMED_TARGET" --target "$EMPTY_TARGET" \
  --target "$DROP_TARGET" \
  --failover safe_only --max-attempts 3 >/dev/null

record_evidence() {
  local target="$1"
  local capability="$2"
  local run="$3"
  "$YAI_BIN" provider suitability record "$target" \
    --capability "$capability" --suite fixture:i03 \
    --run "$run" --evidence-ref "evidence:$run" \
    | sed -n 's/^evidence_id: //p'
}

PRIMARY_EVIDENCE="$(record_evidence "$PRIMARY_TARGET" primary_conversation primary)"
AUX_STT_EVIDENCE="$(record_evidence "$AUXILIARY_TARGET" speech_to_text auxiliary-stt)"
AUX_IMAGE_EVIDENCE="$(record_evidence "$AUXILIARY_TARGET" image_understanding auxiliary-image)"
NO_WIRE_EVIDENCE="$(record_evidence "$NO_WIRE_TARGET" speech_to_text no-wire-stt)"
MALFORMED_EVIDENCE="$(record_evidence "$MALFORMED_TARGET" speech_to_text malformed-stt)"
EMPTY_EVIDENCE="$(record_evidence "$EMPTY_TARGET" speech_to_text empty-stt)"
DROP_EVIDENCE="$(record_evidence "$DROP_TARGET" speech_to_text drop-stt)"

"$YAI_BIN" case cognitive bind case:i03-cli --participant participant:model \
  --role primary --capability primary_conversation --target "$PRIMARY_TARGET" \
  --evidence "$PRIMARY_EVIDENCE" >/dev/null
"$YAI_BIN" case cognitive bind case:i03-cli --participant participant:model \
  --role auxiliary --capability speech_to_text --target "$AUXILIARY_TARGET" \
  --evidence "$AUX_STT_EVIDENCE" >/dev/null
"$YAI_BIN" case cognitive bind case:i03-cli --participant participant:model \
  --role auxiliary --capability image_understanding --target "$AUXILIARY_TARGET" \
  --evidence "$AUX_IMAGE_EVIDENCE" >/dev/null

"$YAI_BIN" case conversation draft create case:i03-cli native-text \
  --participant participant:model >/dev/null
"$YAI_BIN" case conversation draft add-text case:i03-cli native-text \
  --text 'I03 native primary conversation source' >/dev/null
NATIVE_SEND="$("$YAI_BIN" case conversation draft send case:i03-cli native-text)"
NATIVE_TURN="$(sed -n 's/^turn_id: //p' <<<"$NATIVE_SEND")"
NATIVE_RESULT="$("$YAI_BIN" case cognitive realize case:i03-cli \
  --participant participant:model --capability primary_conversation \
  --turn "$NATIVE_TURN" --json)"
grep -Fq '"route":"native"' <<<"$NATIVE_RESULT"
grep -Fq '"provider_execution_performed_now":true' <<<"$NATIVE_RESULT"
grep -Fq '"derived_content":null' <<<"$NATIVE_RESULT"

"$YAI_BIN" case conversation draft create case:i03-cli audio-source \
  --participant participant:model >/dev/null
"$YAI_BIN" case conversation draft import case:i03-cli audio-source \
  "$RUN_ROOT/source.wav" --type audio --mime audio/wav >/dev/null
AUDIO_SEND="$("$YAI_BIN" case conversation draft send case:i03-cli audio-source)"
AUDIO_TURN="$(sed -n 's/^turn_id: //p' <<<"$AUDIO_SEND")"
AUDIO_BEFORE="$("$YAI_BIN" case conversation turn show case:i03-cli "$AUDIO_TURN" \
  --participant participant:model --json)"

set +e
FAILPOINT_OUTPUT="$("$YAI_BIN" case cognitive realize case:i03-cli \
  --participant participant:model --capability speech_to_text \
  --turn "$AUDIO_TURN" --failpoint after-provider-result --json 2>&1)"
FAILPOINT_EXIT=$?
set -e
[[ "$FAILPOINT_EXIT" -ne 0 ]]
grep -Fq 'cognitive_realization_failpoint_after_provider_result' <<<"$FAILPOINT_OUTPUT"
AUX_DISPATCHES_BEFORE_RECOVERY="$(grep -c '"synthetic":false' "$RUN_ROOT/auxiliary.log")"

RECOVERED="$("$YAI_BIN" case cognitive realize case:i03-cli \
  --participant participant:model --capability speech_to_text \
  --turn "$AUDIO_TURN" --json)"
grep -Fq '"posture":"recovered_without_dispatch"' <<<"$RECOVERED"
grep -Fq '"recovered_from_recorded_result":true' <<<"$RECOVERED"
python3 -c 'import json,sys; root=json.loads(sys.argv[1]); value=root.get("data",{}).get("value",root); assert value["plan_id"]==value["derived_content"]["plan_id"]; assert value["validation_plan_id"]' "$RECOVERED"
AUX_DISPATCHES_AFTER_RECOVERY="$(grep -c '"synthetic":false' "$RUN_ROOT/auxiliary.log")"
[[ "$AUX_DISPATCHES_BEFORE_RECOVERY" -eq "$AUX_DISPATCHES_AFTER_RECOVERY" ]]

AUDIO_AFTER="$("$YAI_BIN" case conversation turn show case:i03-cli "$AUDIO_TURN" \
  --participant participant:model --json)"
[[ "$AUDIO_BEFORE" == "$AUDIO_AFTER" ]]
DERIVED_AFTER_RESTART="$("$YAI_BIN" case cognitive derived show case:i03-cli \
  --participant participant:model --json)"
grep -Fq 'fixture transcript ORCHID-I03' <<<"$DERIVED_AFTER_RESTART"
grep -Fq '"content_integrity":"verified"' <<<"$DERIVED_AFTER_RESTART"
set +e
OTHER_PARTICIPANT="$("$YAI_BIN" case cognitive derived show case:i03-cli \
  --participant participant:other --json 2>&1)"
OTHER_PARTICIPANT_EXIT=$?
set -e
[[ "$OTHER_PARTICIPANT_EXIT" -ne 0 ]]
grep -Fq 'cognitive_derived_content_principal_participant_mismatch' <<<"$OTHER_PARTICIPANT"
set +e
OTHER_REALIZE="$("$YAI_BIN" case cognitive realize case:i03-cli \
  --participant participant:other --capability speech_to_text \
  --turn "$AUDIO_TURN" --json 2>&1)"
OTHER_EXIT=$?
set -e
[[ "$OTHER_EXIT" -ne 0 ]]
grep -Fq 'cognitive_realization_principal_participant_mismatch' <<<"$OTHER_REALIZE"

"$YAI_BIN" case conversation draft create case:i03-cli image-source \
  --participant participant:model >/dev/null
"$YAI_BIN" case conversation draft add-text case:i03-cli image-source \
  --text 'first ordered text' >/dev/null
"$YAI_BIN" case conversation draft import case:i03-cli image-source \
  "$RUN_ROOT/source.png" --type image --mime image/png >/dev/null
"$YAI_BIN" case conversation draft import case:i03-cli image-source \
  "$RUN_ROOT/source.png" --type image --mime image/png >/dev/null
"$YAI_BIN" case conversation draft add-text case:i03-cli image-source \
  --text 'last ordered text' >/dev/null
IMAGE_SEND="$("$YAI_BIN" case conversation draft send case:i03-cli image-source)"
IMAGE_TURN="$(sed -n 's/^turn_id: //p' <<<"$IMAGE_SEND")"
IMAGE_RESULT="$("$YAI_BIN" case cognitive realize case:i03-cli \
  --participant participant:model --capability image_understanding \
  --turn "$IMAGE_TURN" --json)"
grep -Fq '"route":"derived"' <<<"$IMAGE_RESULT"
grep -Fq '"realization_shape":"ordered_png_text_to_text"' <<<"$IMAGE_RESULT"
python3 -c 'import json,sys; root=json.loads(sys.argv[1]); result=root.get("data",{}).get("value",root); source_ids=result["derived_content"]["source_part_ids"]; assert len(source_ids)==4 and len(set(source_ids))==4; rows=[json.loads(line) for line in open(sys.argv[2],encoding="utf-8")]; actual=[row for row in rows if not row["synthetic"]]; assert actual[-1]["typed_kinds"] == ["text","text","image_url","image_url","text"]; assert actual[-1]["unexpected_yai_fields"] == []' "$IMAGE_RESULT" "$RUN_ROOT/auxiliary.log"

"$YAI_BIN" case cognitive bind case:i03-cli --participant participant:model \
  --role auxiliary --capability speech_to_text --target "$NO_WIRE_TARGET" \
  --evidence "$NO_WIRE_EVIDENCE" --replace >/dev/null
NO_WIRE_DISPATCHES="$(grep -c '"synthetic":false' "$RUN_ROOT/no_wire.log" || true)"
set +e
NO_WIRE_OUTPUT="$("$YAI_BIN" case cognitive realize case:i03-cli \
  --participant participant:model --capability speech_to_text \
  --turn "$AUDIO_TURN" --json 2>&1)"
NO_WIRE_EXIT=$?
set -e
[[ "$NO_WIRE_EXIT" -ne 0 ]]
grep -Fq 'cognitive_realization_shape_not_qualified' <<<"$NO_WIRE_OUTPUT"
[[ "$NO_WIRE_DISPATCHES" -eq "$(grep -c '"synthetic":false' "$RUN_ROOT/no_wire.log" || true)" ]]

"$YAI_BIN" case cognitive bind case:i03-cli --participant participant:model \
  --role auxiliary --capability speech_to_text --target "$MALFORMED_TARGET" \
  --evidence "$MALFORMED_EVIDENCE" --replace >/dev/null
set +e
MALFORMED_OUTPUT="$("$YAI_BIN" case cognitive realize case:i03-cli \
  --participant participant:model --capability speech_to_text \
  --turn "$AUDIO_TURN" --json 2>&1)"
MALFORMED_EXIT=$?
set -e
[[ "$MALFORMED_EXIT" -ne 0 ]]
grep -Fq 'ResponseInvalid' <<<"$MALFORMED_OUTPUT"

"$YAI_BIN" case cognitive bind case:i03-cli --participant participant:model \
  --role auxiliary --capability speech_to_text --target "$EMPTY_TARGET" \
  --evidence "$EMPTY_EVIDENCE" --replace >/dev/null
# I05 preserves possible prior execution even across an explicit target
# replacement. ResponseInvalid is not retry-safe in the existing taxonomy.
set +e
UNSAFE_REPLACEMENT="$("$YAI_BIN" case cognitive realize case:i03-cli \
  --participant participant:model --capability speech_to_text \
  --turn "$AUDIO_TURN" --json 2>&1)"
UNSAFE_REPLACEMENT_EXIT=$?
set -e
[[ "$UNSAFE_REPLACEMENT_EXIT" -ne 0 ]]
grep -Fq 'cognitive_realization_prior_delivery_indeterminate_requires_resolution' <<<"$UNSAFE_REPLACEMENT"
[[ "$(grep -c '"synthetic":false' "$RUN_ROOT/empty.log" || true)" -eq 0 ]]

# Test normalization of independent newly submitted work, rather than bypass
# uncertain delivery of the malformed-result request by switching providers.
"$YAI_BIN" case conversation draft create case:i03-cli normalization-source \
  --participant participant:model >/dev/null
"$YAI_BIN" case conversation draft import case:i03-cli normalization-source \
  "$RUN_ROOT/source.wav" --type audio --mime audio/wav >/dev/null
NORMALIZATION_SEND="$("$YAI_BIN" case conversation draft send case:i03-cli normalization-source)"
AUDIO_TURN="$(sed -n 's/^turn_id: //p' <<<"$NORMALIZATION_SEND")"
set +e
EMPTY_OUTPUT="$("$YAI_BIN" case cognitive realize case:i03-cli \
  --participant participant:model --capability speech_to_text \
  --turn "$AUDIO_TURN" --json 2>&1)"
EMPTY_EXIT=$?
set -e
[[ "$EMPTY_EXIT" -ne 0 ]]
grep -Fq 'conversation_provider_derived_text_invalid' <<<"$EMPTY_OUTPUT"

"$YAI_BIN" case cognitive bind case:i03-cli --participant participant:model \
  --role auxiliary --capability speech_to_text --target "$DROP_TARGET" \
  --evidence "$DROP_EVIDENCE" --replace >/dev/null
set +e
DROP_OUTPUT="$("$YAI_BIN" case cognitive realize case:i03-cli \
  --participant participant:model --capability speech_to_text \
  --turn "$AUDIO_TURN" --json 2>&1)"
DROP_EXIT=$?
set -e
[[ "$DROP_EXIT" -ne 0 ]]
grep -Fq 'DeliveryIndeterminate' <<<"$DROP_OUTPUT"
DROP_DISPATCHES="$(grep -c '"synthetic":false' "$RUN_ROOT/drop.log")"
set +e
DROP_RETRY_OUTPUT="$("$YAI_BIN" case cognitive realize case:i03-cli \
  --participant participant:model --capability speech_to_text \
  --turn "$AUDIO_TURN" --json 2>&1)"
DROP_RETRY_EXIT=$?
set -e
[[ "$DROP_RETRY_EXIT" -ne 0 ]]
grep -Fq 'existing_attempt_not_retry_safe:DeliveryIndeterminate' <<<"$DROP_RETRY_OUTPUT"
[[ "$DROP_DISPATCHES" -eq "$(grep -c '"synthetic":false' "$RUN_ROOT/drop.log")" ]]

DERIVED_FINAL="$("$YAI_BIN" case cognitive derived show case:i03-cli \
  --participant participant:model --json)"
[[ "$(grep -o 'conversation-derived:' <<<"$DERIVED_FINAL" | wc -l)" -eq 2 ]]
"$YAI_BIN" case show case:i03-cli --json >/dev/null
"$YAI_BIN" case cognitive show case:i03-cli --participant participant:model --json >/dev/null

printf '%s\n' "$NATIVE_RESULT"
printf '%s\n' "$RECOVERED"
printf '%s\n' "$IMAGE_RESULT"
printf '%s\n' "$NO_WIRE_OUTPUT"
printf '%s\n' "$MALFORMED_OUTPUT"
printf '%s\n' "$UNSAFE_REPLACEMENT"
printf '%s\n' "$EMPTY_OUTPUT"
printf '%s\n' "$DROP_OUTPUT"
printf '%s\n' "$DROP_RETRY_OUTPUT"
printf '%s\n' "$OTHER_REALIZE"
printf 'i03_provider_dispatch: native=1 auxiliary=2 exact_target=true\n'
printf 'i03_crash_recovery: result_recorded=true reinvocation=false turn_immutable=true\n'
printf 'i03_fail_closed: missing_wire_evidence=true malformed=true normalization=true delivery_indeterminate_no_retry=true participant_isolation=true derived_count=2\n'
