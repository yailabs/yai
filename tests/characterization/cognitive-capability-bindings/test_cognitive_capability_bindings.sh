#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
YAI_BIN="$ROOT/yai"
RUN_ROOT="$(mktemp -d)"
PRIMARY_PID=""
AUXILIARY_PID=""

cleanup() {
  if [[ -n "$PRIMARY_PID" ]]; then
    kill "$PRIMARY_PID" 2>/dev/null || true
    wait "$PRIMARY_PID" 2>/dev/null || true
  fi
  if [[ -n "$AUXILIARY_PID" ]]; then
    kill "$AUXILIARY_PID" 2>/dev/null || true
    wait "$AUXILIARY_PID" 2>/dev/null || true
  fi
  rm -rf "$RUN_ROOT"
}
trap cleanup EXIT

python3 "$ROOT/tests/fixtures/provider_governance_server.py" \
  --mode full --model whisper-name-is-not-semantics --requests 16 \
  >"$RUN_ROOT/primary.out" 2>"$RUN_ROOT/primary.err" &
PRIMARY_PID=$!
python3 "$ROOT/tests/fixtures/provider_governance_server.py" \
  --mode full --model vision-name-is-not-semantics --requests 16 \
  >"$RUN_ROOT/auxiliary.out" 2>"$RUN_ROOT/auxiliary.err" &
AUXILIARY_PID=$!
for _ in $(seq 1 100); do
  [[ -s "$RUN_ROOT/primary.out" && -s "$RUN_ROOT/auxiliary.out" ]] && break
  sleep 0.05
done
PRIMARY_PORT="$(head -1 "$RUN_ROOT/primary.out")"
AUXILIARY_PORT="$(head -1 "$RUN_ROOT/auxiliary.out")"
[[ "$PRIMARY_PORT" =~ ^[0-9]+$ ]]
[[ "$AUXILIARY_PORT" =~ ^[0-9]+$ ]]

export YAI_HOME="$RUN_ROOT/yai-home"
"$YAI_BIN" init --tenant tenant:i02-cli --organization organization:i02 >/dev/null
"$YAI_BIN" case create case:i02-cli --tenant tenant:i02-cli >/dev/null
"$YAI_BIN" case participant role add case:i02-cli \
  --participant participant:model --role model-executor >/dev/null

primary_add=$("$YAI_BIN" provider add --tenant tenant:i02-cli \
  --provider-key misleading-whisper-primary \
  --endpoint "http://127.0.0.1:$PRIMARY_PORT" \
  --model whisper-name-is-not-semantics --locality loopback)
PRIMARY_TARGET=$(awk '/Target id/ {print $NF} /target_id:/ {print $2}' <<<"$primary_add" | tail -1)
auxiliary_add=$("$YAI_BIN" provider add --tenant tenant:i02-cli \
  --provider-key misleading-vision-auxiliary \
  --endpoint "http://127.0.0.1:$AUXILIARY_PORT" \
  --model vision-name-is-not-semantics --locality loopback)
AUXILIARY_TARGET=$(awk '/Target id/ {print $NF} /target_id:/ {print $2}' <<<"$auxiliary_add" | tail -1)
[[ "$PRIMARY_TARGET" == provider-target:* ]]
[[ "$AUXILIARY_TARGET" == provider-target:* ]]

"$YAI_BIN" provider qualify "$PRIMARY_TARGET" >/dev/null
"$YAI_BIN" provider qualify "$AUXILIARY_TARGET" >/dev/null
"$YAI_BIN" provider trust approve "$PRIMARY_TARGET" >/dev/null
"$YAI_BIN" provider trust approve "$AUXILIARY_TARGET" >/dev/null
"$YAI_BIN" case provider bind case:i02-cli --participant participant:model \
  --target "$PRIMARY_TARGET" --target "$AUXILIARY_TARGET" \
  --failover safe_only --max-attempts 2 >/dev/null

primary_evidence=$("$YAI_BIN" provider suitability record "$PRIMARY_TARGET" \
  --capability primary_conversation --suite operator-suite:i02 \
  --run operator-run:primary --evidence-ref evidence:primary-conversation)
PRIMARY_EVIDENCE=$(sed -n 's/^evidence_id: //p' <<<"$primary_evidence")
auxiliary_evidence=$("$YAI_BIN" provider suitability record "$AUXILIARY_TARGET" \
  --capability speech_to_text --suite operator-suite:i02 \
  --run operator-run:stt --evidence-ref evidence:speech-to-text)
AUXILIARY_EVIDENCE=$(sed -n 's/^evidence_id: //p' <<<"$auxiliary_evidence")
[[ "$PRIMARY_EVIDENCE" == semantic-suitability:* ]]
[[ "$AUXILIARY_EVIDENCE" == semantic-suitability:* ]]

"$YAI_BIN" case cognitive bind case:i02-cli --participant participant:model \
  --role primary --capability primary_conversation --target "$PRIMARY_TARGET" \
  --evidence "$PRIMARY_EVIDENCE" >/dev/null
"$YAI_BIN" case cognitive bind case:i02-cli --participant participant:model \
  --role auxiliary --capability speech_to_text --target "$AUXILIARY_TARGET" \
  --evidence "$AUXILIARY_EVIDENCE" >/dev/null

# Planning must remain successful after both provider endpoints disappear.
kill "$PRIMARY_PID" "$AUXILIARY_PID"
wait "$PRIMARY_PID" 2>/dev/null || true
wait "$AUXILIARY_PID" 2>/dev/null || true
PRIMARY_PID=""
AUXILIARY_PID=""

primary_plan=$("$YAI_BIN" case cognitive plan case:i02-cli \
  --participant participant:model --capability primary_conversation \
  --source turn:i02-text --json)
grep -Fq '"route":"native"' <<<"$primary_plan"
grep -Fq '"role":"primary"' <<<"$primary_plan"
grep -Fq '"provider_execution":"not_performed"' <<<"$primary_plan"

derived_plan=$("$YAI_BIN" case cognitive plan case:i02-cli \
  --participant participant:model --capability speech_to_text \
  --source turn:i02-audio --json)
grep -Fq '"route":"derived"' <<<"$derived_plan"
grep -Fq '"role":"auxiliary"' <<<"$derived_plan"
grep -Fq '"provider_execution":"not_performed"' <<<"$derived_plan"

unresolved_plan=$("$YAI_BIN" case cognitive plan case:i02-cli \
  --participant participant:model --capability image_understanding \
  --source turn:i02-image --json)
grep -Fq '"route":"unresolved"' <<<"$unresolved_plan"
grep -Fq '"auxiliary_binding_missing"' <<<"$unresolved_plan"

binding_json=$("$YAI_BIN" case cognitive show case:i02-cli \
  --participant participant:model --json)
grep -Fq '"count":2' <<<"$binding_json"
grep -Fq "$PRIMARY_TARGET" <<<"$binding_json"
grep -Fq "$AUXILIARY_TARGET" <<<"$binding_json"

printf '%s\n' "$primary_plan"
printf '%s\n' "$derived_plan"
printf '%s\n' "$unresolved_plan"
printf 'i02_cli: bindings=2 native=true derived=true unresolved=true provider_endpoints=stopped provider_dispatches=0\n'
