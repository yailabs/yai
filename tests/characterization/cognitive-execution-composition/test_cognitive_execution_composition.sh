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

start_provider primary full whisper-name-with-text-wire-only
start_provider auxiliary full general-transformer-not-whisper
start_provider native full text-name-with-native-audio-wire
start_provider drop drop_realization speech-name-with-indeterminate-delivery
start_provider malformed malformed_realization vision-name-with-malformed-audio-result
start_provider empty empty_realization speech-name-with-empty-normalization

for _ in $(seq 1 100); do
  [[ -s "$RUN_ROOT/primary.port" \
    && -s "$RUN_ROOT/auxiliary.port" \
    && -s "$RUN_ROOT/native.port" \
    && -s "$RUN_ROOT/drop.port" \
    && -s "$RUN_ROOT/malformed.port" \
    && -s "$RUN_ROOT/empty.port" ]] && break
  sleep 0.05
done

PRIMARY_PORT="$(head -1 "$RUN_ROOT/primary.port")"
AUXILIARY_PORT="$(head -1 "$RUN_ROOT/auxiliary.port")"
NATIVE_PORT="$(head -1 "$RUN_ROOT/native.port")"
DROP_PORT="$(head -1 "$RUN_ROOT/drop.port")"
MALFORMED_PORT="$(head -1 "$RUN_ROOT/malformed.port")"
EMPTY_PORT="$(head -1 "$RUN_ROOT/empty.port")"
for port in "$PRIMARY_PORT" "$AUXILIARY_PORT" "$NATIVE_PORT" "$DROP_PORT" "$MALFORMED_PORT" "$EMPTY_PORT"; do
  [[ "$port" =~ ^[0-9]+$ ]]
done

base64 -d "$ROOT/tests/fixtures/conversation/i03-audio.wav.base64" >"$RUN_ROOT/source.wav"
base64 -d "$ROOT/tests/fixtures/conversation/i03-image.png.base64" >"$RUN_ROOT/source.png"

export YAI_HOME="$RUN_ROOT/yai-home"
"$YAI_BIN" init --tenant tenant:i04-cli --organization organization:i04 >/dev/null

add_target() {
  local key="$1"
  local port="$2"
  local model="$3"
  "$YAI_BIN" provider add --tenant tenant:i04-cli \
    --provider-key "$key" --endpoint "http://127.0.0.1:$port" \
    --model "$model" --locality loopback \
    | sed -n 's/^target_id: //p'
}

PRIMARY_TARGET="$(add_target misleading-primary "$PRIMARY_PORT" whisper-name-with-text-wire-only)"
AUXILIARY_TARGET="$(add_target generic-auxiliary "$AUXILIARY_PORT" general-transformer-not-whisper)"
NATIVE_TARGET="$(add_target native-audio-primary "$NATIVE_PORT" text-name-with-native-audio-wire)"
DROP_TARGET="$(add_target misleading-drop "$DROP_PORT" speech-name-with-indeterminate-delivery)"
MALFORMED_TARGET="$(add_target misleading-malformed "$MALFORMED_PORT" vision-name-with-malformed-audio-result)"
EMPTY_TARGET="$(add_target misleading-empty "$EMPTY_PORT" speech-name-with-empty-normalization)"

"$YAI_BIN" provider qualify "$PRIMARY_TARGET" --realization-shape text_to_text >/dev/null
"$YAI_BIN" provider qualify "$AUXILIARY_TARGET" \
  --realization-shape audio_wav_to_text \
  --realization-shape ordered_png_text_to_text >/dev/null
"$YAI_BIN" provider qualify "$NATIVE_TARGET" \
  --realization-shape text_to_text --realization-shape audio_wav_to_text >/dev/null
"$YAI_BIN" provider qualify "$DROP_TARGET" --realization-shape audio_wav_to_text >/dev/null
"$YAI_BIN" provider qualify "$MALFORMED_TARGET" --realization-shape audio_wav_to_text >/dev/null
"$YAI_BIN" provider qualify "$EMPTY_TARGET" --realization-shape audio_wav_to_text >/dev/null
for target in "$PRIMARY_TARGET" "$AUXILIARY_TARGET" "$NATIVE_TARGET" "$DROP_TARGET" "$MALFORMED_TARGET" "$EMPTY_TARGET"; do
  "$YAI_BIN" provider trust approve "$target" >/dev/null
done

record_evidence() {
  local target="$1"
  local capability="$2"
  local run="$3"
  "$YAI_BIN" provider suitability record "$target" \
    --capability "$capability" --suite fixture:i04 \
    --run "$run" --evidence-ref "evidence:$run" \
    | sed -n 's/^evidence_id: //p'
}

PRIMARY_EVIDENCE="$(record_evidence "$PRIMARY_TARGET" primary_conversation primary)"
AUXILIARY_EVIDENCE="$(record_evidence "$AUXILIARY_TARGET" speech_to_text auxiliary)"
AUXILIARY_IMAGE_EVIDENCE="$(record_evidence "$AUXILIARY_TARGET" image_understanding auxiliary-image)"
NATIVE_EVIDENCE="$(record_evidence "$NATIVE_TARGET" primary_conversation native)"
DROP_EVIDENCE="$(record_evidence "$DROP_TARGET" speech_to_text drop)"
MALFORMED_EVIDENCE="$(record_evidence "$MALFORMED_TARGET" speech_to_text malformed)"
EMPTY_EVIDENCE="$(record_evidence "$EMPTY_TARGET" speech_to_text empty)"

prepare_case() {
  local case_id="$1"
  "$YAI_BIN" case create "$case_id" --tenant tenant:i04-cli >/dev/null
  "$YAI_BIN" case participant role add "$case_id" \
    --participant participant:model --role model-executor >/dev/null
  "$YAI_BIN" case participant link-principal "$case_id" \
    --principal self --participant participant:model >/dev/null
  "$YAI_BIN" case participant view admit "$case_id" \
    --participant participant:model --consumer model --view model_context >/dev/null
}

make_audio_turn() {
  local case_id="$1"
  local draft_id="$2"
  "$YAI_BIN" case conversation draft create "$case_id" "$draft_id" \
    --participant participant:model >/dev/null
  "$YAI_BIN" case conversation draft add-text "$case_id" "$draft_id" \
    --text 'before exact audio source' >/dev/null
  "$YAI_BIN" case conversation draft import "$case_id" "$draft_id" \
    "$RUN_ROOT/source.wav" --type audio --mime audio/wav >/dev/null
  "$YAI_BIN" case conversation draft add-text "$case_id" "$draft_id" \
    --text 'after exact audio source' >/dev/null
  "$YAI_BIN" case conversation draft send "$case_id" "$draft_id" \
    | sed -n 's/^turn_id: //p'
}

audio_part_id() {
  local case_id="$1"
  local turn_id="$2"
  local value
  value="$("$YAI_BIN" case conversation turn show "$case_id" "$turn_id" \
    --participant participant:model --json)"
  python3 -c 'import json,sys; root=json.loads(sys.argv[1]); value=root.get("data",{}).get("value",root); print(next(part["part_id"] for part in value["turn"]["ordered_parts"] if part["object"]["modality"]=="audio"))' "$value"
}

prepare_case case:i04-composed
"$YAI_BIN" case provider bind case:i04-composed --participant participant:model \
  --target "$PRIMARY_TARGET" --target "$AUXILIARY_TARGET" --target "$DROP_TARGET" \
  --failover safe_only --max-attempts 3 >/dev/null
"$YAI_BIN" case cognitive bind case:i04-composed --participant participant:model \
  --role primary --capability primary_conversation --target "$PRIMARY_TARGET" \
  --evidence "$PRIMARY_EVIDENCE" >/dev/null
"$YAI_BIN" case cognitive bind case:i04-composed --participant participant:model \
  --role auxiliary --capability speech_to_text --target "$AUXILIARY_TARGET" \
  --evidence "$AUXILIARY_EVIDENCE" >/dev/null
"$YAI_BIN" case cognitive bind case:i04-composed --participant participant:model \
  --role auxiliary --capability image_understanding --target "$AUXILIARY_TARGET" \
  --evidence "$AUXILIARY_IMAGE_EVIDENCE" >/dev/null

COMPOSED_TURN="$(make_audio_turn case:i04-composed composed-audio)"
COMPOSED_AUDIO_PART="$(audio_part_id case:i04-composed "$COMPOSED_TURN")"
TURN_BEFORE="$("$YAI_BIN" case conversation turn show case:i04-composed "$COMPOSED_TURN" \
  --participant participant:model --json)"

PRIMARY_BEFORE="$(grep -c '"synthetic":false' "$RUN_ROOT/primary.log" || true)"
AUX_BEFORE="$(grep -c '"synthetic":false' "$RUN_ROOT/auxiliary.log" || true)"
set +e
FAILPOINT_OUTPUT="$("$YAI_BIN" case cognitive compose case:i04-composed \
  --participant participant:model --goal primary_conversation --turn "$COMPOSED_TURN" \
  --prerequisite speech_to_text --prerequisite-part "$COMPOSED_AUDIO_PART" \
  --failpoint after-prerequisite --json 2>&1)"
FAILPOINT_EXIT=$?
set -e
[[ "$FAILPOINT_EXIT" -ne 0 ]]
grep -Fq 'cognitive_composition_failpoint_after_prerequisite' <<<"$FAILPOINT_OUTPUT"
[[ "$PRIMARY_BEFORE" -eq "$(grep -c '"synthetic":false' "$RUN_ROOT/primary.log" || true)" ]]
[[ "$((AUX_BEFORE + 1))" -eq "$(grep -c '"synthetic":false' "$RUN_ROOT/auxiliary.log")" ]]

AUX_AFTER_PREREQUISITE="$(grep -c '"synthetic":false' "$RUN_ROOT/auxiliary.log")"
COMPOSED_RESULT="$("$YAI_BIN" case cognitive compose case:i04-composed \
  --participant participant:model --goal primary_conversation --turn "$COMPOSED_TURN" \
  --prerequisite speech_to_text --prerequisite-part "$COMPOSED_AUDIO_PART" --json)"
grep -Fq '"composition_route":"composed"' <<<"$COMPOSED_RESULT"
grep -Fq '"disposition":"replaced_by_derived"' <<<"$COMPOSED_RESULT"
grep -Fq '"composition_owner":"none_derived_control"' <<<"$COMPOSED_RESULT"
[[ "$AUX_AFTER_PREREQUISITE" -eq "$(grep -c '"synthetic":false' "$RUN_ROOT/auxiliary.log")" ]]
python3 -c 'import json,sys; root=json.loads(sys.argv[1]); value=root.get("data",{}).get("value",root); c=value["source_closure"]; assert c["delivery_count"]==3; assert [x["source_ordinal"] for x in c["entries"]]==[0,1,2]; assert [x["disposition"] for x in c["entries"]]==["retained_original","replaced_by_derived","retained_original"]; assert value["prerequisite"]["provider_execution_performed_now"] is False; assert value["primary"]["provider_execution_performed_now"] is True' "$COMPOSED_RESULT"
python3 -c 'import json,sys; rows=[json.loads(line) for line in open(sys.argv[1],encoding="utf-8")]; actual=[row for row in rows if not row["synthetic"]]; assert actual[-1]["typed_kinds"]==["text","text","text","text"]' "$RUN_ROOT/primary.log"

TURN_AFTER="$("$YAI_BIN" case conversation turn show case:i04-composed "$COMPOSED_TURN" \
  --participant participant:model --json)"
[[ "$TURN_BEFORE" == "$TURN_AFTER" ]]
PRIMARY_AFTER="$(grep -c '"synthetic":false' "$RUN_ROOT/primary.log")"
COMPOSED_REPEAT="$("$YAI_BIN" case cognitive compose case:i04-composed \
  --participant participant:model --goal primary_conversation --turn "$COMPOSED_TURN" \
  --prerequisite speech_to_text --prerequisite-part "$COMPOSED_AUDIO_PART" --json)"
[[ "$AUX_AFTER_PREREQUISITE" -eq "$(grep -c '"synthetic":false' "$RUN_ROOT/auxiliary.log")" ]]
[[ "$PRIMARY_AFTER" -eq "$(grep -c '"synthetic":false' "$RUN_ROOT/primary.log")" ]]
python3 -c 'import json,sys; one=json.loads(sys.argv[1]).get("data",{}).get("value",json.loads(sys.argv[1])); two=json.loads(sys.argv[2]).get("data",{}).get("value",json.loads(sys.argv[2])); assert one["composition_request"]["request_id"]==two["composition_request"]["request_id"]; assert one["source_closure"]["closure_id"]==two["source_closure"]["closure_id"]; assert two["primary"]["provider_execution_performed_now"] is False' "$COMPOSED_RESULT" "$COMPOSED_REPEAT"

AUX_BEFORE_REQUALIFY="$(grep -c '"synthetic":false' "$RUN_ROOT/auxiliary.log")"
PRIMARY_BEFORE_REQUALIFY="$(grep -c '"synthetic":false' "$RUN_ROOT/primary.log")"
"$YAI_BIN" provider qualify "$AUXILIARY_TARGET" \
  --realization-shape audio_wav_to_text \
  --realization-shape ordered_png_text_to_text >/dev/null
REQUALIFIED_RESULT="$("$YAI_BIN" case cognitive compose case:i04-composed \
  --participant participant:model --goal primary_conversation --turn "$COMPOSED_TURN" \
  --prerequisite speech_to_text --prerequisite-part "$COMPOSED_AUDIO_PART" --json)"
[[ "$((AUX_BEFORE_REQUALIFY + 1))" -eq "$(grep -c '"synthetic":false' "$RUN_ROOT/auxiliary.log")" ]]
[[ "$((PRIMARY_BEFORE_REQUALIFY + 1))" -eq "$(grep -c '"synthetic":false' "$RUN_ROOT/primary.log")" ]]
python3 -c 'import json,sys; one=json.loads(sys.argv[1]).get("data",{}).get("value",json.loads(sys.argv[1])); two=json.loads(sys.argv[2]).get("data",{}).get("value",json.loads(sys.argv[2])); assert one["composition_request"]["request_id"]==two["composition_request"]["request_id"]; assert one["source_closure"]["closure_id"]!=two["source_closure"]["closure_id"]; assert one["prerequisite"]["derived_content_id"]!=two["prerequisite"]["derived_content_id"]' "$COMPOSED_RESULT" "$REQUALIFIED_RESULT"

"$YAI_BIN" case conversation draft create case:i04-composed image-composition \
  --participant participant:model >/dev/null
"$YAI_BIN" case conversation draft add-text case:i04-composed image-composition \
  --text 'before exact images' >/dev/null
"$YAI_BIN" case conversation draft import case:i04-composed image-composition \
  "$RUN_ROOT/source.png" --type image --mime image/png >/dev/null
"$YAI_BIN" case conversation draft import case:i04-composed image-composition \
  "$RUN_ROOT/source.png" --type image --mime image/png >/dev/null
"$YAI_BIN" case conversation draft add-text case:i04-composed image-composition \
  --text 'after exact images' >/dev/null
IMAGE_SEND="$("$YAI_BIN" case conversation draft send case:i04-composed image-composition)"
IMAGE_TURN="$(sed -n 's/^turn_id: //p' <<<"$IMAGE_SEND")"
IMAGE_SHOW="$("$YAI_BIN" case conversation turn show case:i04-composed "$IMAGE_TURN" \
  --participant participant:model)"
mapfile -t IMAGE_PARTS < <(sed -n 's/^part: .* id=\([^ ]*\) type=image .*/\1/p' <<<"$IMAGE_SHOW")
[[ "${#IMAGE_PARTS[@]}" -eq 2 ]]
IMAGE_RESULT="$("$YAI_BIN" case cognitive compose case:i04-composed \
  --participant participant:model --goal primary_conversation --turn "$IMAGE_TURN" \
  --prerequisite image_understanding \
  --prerequisite-part "${IMAGE_PARTS[0]}" --prerequisite-part "${IMAGE_PARTS[1]}" --json)"
grep -Fq '"composition_route":"composed"' <<<"$IMAGE_RESULT"
grep -Fq '"disposition":"consumed_by_derivation"' <<<"$IMAGE_RESULT"
python3 -c 'import json,sys; root=json.loads(sys.argv[1]); value=root.get("data",{}).get("value",root); entries=value["source_closure"]["entries"]; assert [x["disposition"] for x in entries]==["retained_original","replaced_by_derived","consumed_by_derivation","retained_original"]; assert value["source_closure"]["delivery_count"]==3' "$IMAGE_RESULT"

set +e
NO_INTENT_OUTPUT="$("$YAI_BIN" case cognitive compose case:i04-composed \
  --participant participant:model --goal primary_conversation --turn "$COMPOSED_TURN" --json 2>&1)"
NO_INTENT_EXIT=$?
set -e
[[ "$NO_INTENT_EXIT" -ne 0 ]]
grep -Fq 'cognitive_composition_prerequisite_required' <<<"$NO_INTENT_OUTPUT"

prepare_case case:i04-direct
"$YAI_BIN" case provider bind case:i04-direct --participant participant:model \
  --target "$NATIVE_TARGET" --failover safe_only --max-attempts 1 >/dev/null
"$YAI_BIN" case cognitive bind case:i04-direct --participant participant:model \
  --role primary --capability primary_conversation --target "$NATIVE_TARGET" \
  --evidence "$NATIVE_EVIDENCE" >/dev/null
DIRECT_TURN="$(make_audio_turn case:i04-direct direct-audio)"
DIRECT_AUDIO_PART="$(audio_part_id case:i04-direct "$DIRECT_TURN")"
NATIVE_BEFORE="$(grep -c '"synthetic":false' "$RUN_ROOT/native.log" || true)"
AUX_BEFORE_DIRECT="$(grep -c '"synthetic":false' "$RUN_ROOT/auxiliary.log")"
DIRECT_RESULT="$("$YAI_BIN" case cognitive compose case:i04-direct \
  --participant participant:model --goal primary_conversation --turn "$DIRECT_TURN" \
  --prerequisite speech_to_text --prerequisite-part "$DIRECT_AUDIO_PART" --json)"
grep -Fq '"composition_route":"direct"' <<<"$DIRECT_RESULT"
grep -Fq '"prerequisite":null' <<<"$DIRECT_RESULT"
[[ "$((NATIVE_BEFORE + 1))" -eq "$(grep -c '"synthetic":false' "$RUN_ROOT/native.log")" ]]
[[ "$AUX_BEFORE_DIRECT" -eq "$(grep -c '"synthetic":false' "$RUN_ROOT/auxiliary.log")" ]]

# Rebind the same prerequisite source to a different target/evidence. Historical
# derived content for the old binding must not satisfy the new composition.
"$YAI_BIN" case cognitive bind case:i04-composed --participant participant:model \
  --role auxiliary --capability speech_to_text --target "$DROP_TARGET" \
  --evidence "$DROP_EVIDENCE" --replace >/dev/null
PRIMARY_BEFORE_FAILURE="$(grep -c '"synthetic":false' "$RUN_ROOT/primary.log")"
set +e
FAILURE_OUTPUT="$("$YAI_BIN" case cognitive compose case:i04-composed \
  --participant participant:model --goal primary_conversation --turn "$COMPOSED_TURN" \
  --prerequisite speech_to_text --prerequisite-part "$COMPOSED_AUDIO_PART" --json 2>&1)"
FAILURE_EXIT=$?
set -e
[[ "$FAILURE_EXIT" -ne 0 ]]
grep -Fq 'DeliveryIndeterminate' <<<"$FAILURE_OUTPUT"
[[ "$PRIMARY_BEFORE_FAILURE" -eq "$(grep -c '"synthetic":false' "$RUN_ROOT/primary.log")" ]]

prepare_case case:i04-malformed
"$YAI_BIN" case provider bind case:i04-malformed --participant participant:model \
  --target "$PRIMARY_TARGET" --target "$MALFORMED_TARGET" \
  --failover safe_only --max-attempts 2 >/dev/null
"$YAI_BIN" case cognitive bind case:i04-malformed --participant participant:model \
  --role primary --capability primary_conversation --target "$PRIMARY_TARGET" \
  --evidence "$PRIMARY_EVIDENCE" >/dev/null
"$YAI_BIN" case cognitive bind case:i04-malformed --participant participant:model \
  --role auxiliary --capability speech_to_text --target "$MALFORMED_TARGET" \
  --evidence "$MALFORMED_EVIDENCE" >/dev/null
MALFORMED_TURN="$(make_audio_turn case:i04-malformed malformed-audio)"
MALFORMED_PART="$(audio_part_id case:i04-malformed "$MALFORMED_TURN")"
PRIMARY_BEFORE_MALFORMED="$(grep -c '"synthetic":false' "$RUN_ROOT/primary.log")"
set +e
MALFORMED_OUTPUT="$("$YAI_BIN" case cognitive compose case:i04-malformed \
  --participant participant:model --goal primary_conversation --turn "$MALFORMED_TURN" \
  --prerequisite speech_to_text --prerequisite-part "$MALFORMED_PART" --json 2>&1)"
MALFORMED_EXIT=$?
set -e
[[ "$MALFORMED_EXIT" -ne 0 ]]
grep -Fq 'ResponseInvalid' <<<"$MALFORMED_OUTPUT"
[[ "$PRIMARY_BEFORE_MALFORMED" -eq "$(grep -c '"synthetic":false' "$RUN_ROOT/primary.log")" ]]
MALFORMED_DERIVED="$("$YAI_BIN" case cognitive derived show case:i04-malformed \
  --participant participant:model --json)"
[[ "$(grep -o 'conversation-derived:' <<<"$MALFORMED_DERIVED" | wc -l)" -eq 0 ]]

prepare_case case:i04-normalization
"$YAI_BIN" case provider bind case:i04-normalization --participant participant:model \
  --target "$PRIMARY_TARGET" --target "$EMPTY_TARGET" \
  --failover safe_only --max-attempts 2 >/dev/null
"$YAI_BIN" case cognitive bind case:i04-normalization --participant participant:model \
  --role primary --capability primary_conversation --target "$PRIMARY_TARGET" \
  --evidence "$PRIMARY_EVIDENCE" >/dev/null
"$YAI_BIN" case cognitive bind case:i04-normalization --participant participant:model \
  --role auxiliary --capability speech_to_text --target "$EMPTY_TARGET" \
  --evidence "$EMPTY_EVIDENCE" >/dev/null
EMPTY_TURN="$(make_audio_turn case:i04-normalization empty-audio)"
EMPTY_PART="$(audio_part_id case:i04-normalization "$EMPTY_TURN")"
PRIMARY_BEFORE_EMPTY="$(grep -c '"synthetic":false' "$RUN_ROOT/primary.log")"
set +e
EMPTY_OUTPUT="$("$YAI_BIN" case cognitive compose case:i04-normalization \
  --participant participant:model --goal primary_conversation --turn "$EMPTY_TURN" \
  --prerequisite speech_to_text --prerequisite-part "$EMPTY_PART" --json 2>&1)"
EMPTY_EXIT=$?
set -e
[[ "$EMPTY_EXIT" -ne 0 ]]
grep -Fq 'conversation_provider_derived_text_invalid' <<<"$EMPTY_OUTPUT"
[[ "$PRIMARY_BEFORE_EMPTY" -eq "$(grep -c '"synthetic":false' "$RUN_ROOT/primary.log")" ]]
EMPTY_DERIVED="$("$YAI_BIN" case cognitive derived show case:i04-normalization \
  --participant participant:model --json)"
[[ "$(grep -o 'conversation-derived:' <<<"$EMPTY_DERIVED" | wc -l)" -eq 0 ]]

prepare_case case:i04-unresolved
"$YAI_BIN" case provider bind case:i04-unresolved --participant participant:model \
  --target "$PRIMARY_TARGET" --failover safe_only --max-attempts 1 >/dev/null
"$YAI_BIN" case cognitive bind case:i04-unresolved --participant participant:model \
  --role primary --capability primary_conversation --target "$PRIMARY_TARGET" \
  --evidence "$PRIMARY_EVIDENCE" >/dev/null
UNRESOLVED_TURN="$(make_audio_turn case:i04-unresolved unresolved-audio)"
UNRESOLVED_PART="$(audio_part_id case:i04-unresolved "$UNRESOLVED_TURN")"
PRIMARY_BEFORE_UNRESOLVED="$(grep -c '"synthetic":false' "$RUN_ROOT/primary.log")"
set +e
UNRESOLVED_OUTPUT="$("$YAI_BIN" case cognitive compose case:i04-unresolved \
  --participant participant:model --goal primary_conversation --turn "$UNRESOLVED_TURN" \
  --prerequisite speech_to_text --prerequisite-part "$UNRESOLVED_PART" --json 2>&1)"
UNRESOLVED_EXIT=$?
set -e
[[ "$UNRESOLVED_EXIT" -ne 0 ]]
grep -Fq 'cognitive_composition_prerequisite_not_auxiliary:Unresolved' <<<"$UNRESOLVED_OUTPUT"
[[ "$PRIMARY_BEFORE_UNRESOLVED" -eq "$(grep -c '"synthetic":false' "$RUN_ROOT/primary.log")" ]]

DERIVED="$("$YAI_BIN" case cognitive derived show case:i04-composed \
  --participant participant:model --json)"
[[ "$(grep -o 'conversation-derived:' <<<"$DERIVED" | wc -l)" -eq 3 ]]

printf '%s\n' "$COMPOSED_RESULT"
printf '%s\n' "$COMPOSED_REPEAT"
printf '%s\n' "$REQUALIFIED_RESULT"
printf '%s\n' "$DIRECT_RESULT"
printf '%s\n' "$IMAGE_RESULT"
printf '%s\n' "$NO_INTENT_OUTPUT"
printf '%s\n' "$FAILURE_OUTPUT"
printf '%s\n' "$MALFORMED_OUTPUT"
printf '%s\n' "$EMPTY_OUTPUT"
printf '%s\n' "$UNRESOLVED_OUTPUT"
printf 'i04_composed: auxiliary_dispatch_once=true restart_resume=true primary_dispatch=true turn_immutable=true audio_and_image=true\n'
printf 'i04_direct: primary_native_audio=true auxiliary_bypassed=true explicit_intent_preserved=true\n'
printf 'i04_failure: delivery_indeterminate_blocks_primary=true malformed_blocks_primary=true normalization_blocks_primary=true unresolved_blocks_primary=true derived_duplicates=0\n'
printf 'i04_revalidation: binding_evidence_and_qualification_change_replans=true historical_derivation_preserved=true\n'
