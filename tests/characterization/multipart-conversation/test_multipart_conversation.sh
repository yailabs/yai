#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
YAI_BIN="$ROOT/target/debug/yai"
RUN_ROOT="$(mktemp -d)"
trap 'rm -rf "$RUN_ROOT"' EXIT
export YAI_HOME="$RUN_ROOT/yai-home"

"$YAI_BIN" init --tenant tenant:i01-smoke --organization organization:cli-product >/dev/null
"$YAI_BIN" case create case:i01-smoke --tenant tenant:i01-smoke >/dev/null
"$YAI_BIN" case participant role add case:i01-smoke \
  --participant participant:operator --role model-executor >/dev/null
"$YAI_BIN" case participant role add case:i01-smoke \
  --participant participant:operator --role operation-proposer >/dev/null
"$YAI_BIN" case participant link-principal case:i01-smoke \
  --principal self --participant participant:operator >/dev/null
"$YAI_BIN" case participant view admit case:i01-smoke \
  --participant participant:operator --consumer model --view model_context >/dev/null

"$YAI_BIN" case conversation draft create case:i01-smoke draft-text \
  --participant participant:operator >/dev/null
"$YAI_BIN" case conversation draft add-text case:i01-smoke draft-text \
  --text 'I01 text-only canonical Turn' >/dev/null
text_send="$("$YAI_BIN" case conversation draft send case:i01-smoke draft-text)"
grep -Fq 'canonical: yes' <<<"$text_send"
grep -Fq 'provider_execution_started: no' <<<"$text_send"

"$YAI_BIN" case conversation draft create case:i01-smoke draft-multipart \
  --participant participant:operator >/dev/null
"$YAI_BIN" case conversation draft add-text case:i01-smoke draft-multipart \
  --text 'part zero' >/dev/null
"$YAI_BIN" case conversation draft import case:i01-smoke draft-multipart \
  "$ROOT/tests/fixtures/conversation/i01-image-one.svg" \
  --type image --mime image/svg+xml >/dev/null
"$YAI_BIN" case conversation draft import case:i01-smoke draft-multipart \
  "$ROOT/tests/fixtures/conversation/i01-image-one.svg" \
  --type image --mime image/svg+xml >/dev/null
"$YAI_BIN" case conversation draft import case:i01-smoke draft-multipart \
  "$ROOT/tests/fixtures/conversation/i01-audio.fixture" \
  --type audio --mime audio/x-yai-fixture >/dev/null
"$YAI_BIN" case conversation draft derive-text case:i01-smoke draft-multipart \
  --source-part 3 --kind speech-transcription \
  --producer-ref execution:fixture-stt \
  --text 'machine transcript ORCHID-I01 4188' >/dev/null
"$YAI_BIN" case conversation draft derive-text case:i01-smoke draft-multipart \
  --source-part 4 --kind human-edit \
  --text 'human edited transcript ORCHID-I01 4189' >/dev/null
multipart_send="$("$YAI_BIN" case conversation draft send case:i01-smoke draft-multipart)"
grep -Fq 'ordered_parts: 6' <<<"$multipart_send"

turn="$("$YAI_BIN" case conversation turn show case:i01-smoke latest \
  --participant participant:operator)"
grep -Fq 'content_integrity: verified' <<<"$turn"
grep -Fq 'part: 0' <<<"$turn"
grep -Fq 'part: 5' <<<"$turn"
grep -Fq 'type=image' <<<"$turn"
grep -Fq 'type=audio' <<<"$turn"
grep -Fq 'provenance=machine_or_deterministic_derived' <<<"$turn"
grep -Fq 'provenance=human_edited_derived' <<<"$turn"
image_objects="$(grep 'type=image' <<<"$turn" | sed -n 's/.* object=\([^ ]*\) storage=.*/\1/p')"
[[ "$(sort -u <<<"$image_objects" | wc -l)" -eq 1 ]]
image_parts="$(grep 'type=image' <<<"$turn" | sed -n 's/.*id=\([^ ]*\) type=.*/\1/p')"
[[ "$(sort -u <<<"$image_parts" | wc -l)" -eq 2 ]]

turn_after_restart="$("$YAI_BIN" case conversation turn show case:i01-smoke latest \
  --participant participant:operator)"
[[ "$turn" == "$turn_after_restart" ]]
list_json="$("$YAI_BIN" case conversation turn list case:i01-smoke \
  --participant participant:operator --json)"
grep -Fq 'yai.conversation_turn_list.v1' <<<"$list_json"
grep -Fq 'yai.conversation_turn.v1' <<<"$list_json"

mkdir -p "$RUN_ROOT/resource/allowed"
"$YAI_BIN" case resource attach filesystem case:i01-smoke \
  --resource resource:i01-smoke --root "$RUN_ROOT/resource" \
  --allow-prefix allowed --policy-owner participant:operator --max-bytes 4096 >/dev/null
"$YAI_BIN" policy ingest "$ROOT/tests/fixtures/cli-product-policy.json" \
  --tenant tenant:i01-smoke --validate --publish \
  --reason 'I01 provider failure qualification' >/dev/null
"$YAI_BIN" case policy bind case:i01-smoke --policy-key cli.product.governed \
  --reason 'I01 provider failure qualification' >/dev/null
"$YAI_BIN" case provider attach case:i01-smoke \
  --participant participant:operator --endpoint http://127.0.0.1:9/v1 \
  --model unavailable-i01 --provider provider:i01-unavailable >/dev/null

"$YAI_BIN" case conversation draft create case:i01-smoke draft-failure \
  --participant participant:operator >/dev/null
"$YAI_BIN" case conversation draft add-text case:i01-smoke draft-failure \
  --text 'This canonical Turn survives downstream provider failure.' >/dev/null
failure_send="$("$YAI_BIN" case conversation draft send case:i01-smoke draft-failure)"
failure_turn="$(sed -n 's/^turn_id: //p' <<<"$failure_send")"
set +e
runtime="$("$YAI_BIN" case run case:i01-smoke \
  --participant participant:operator --resource resource:i01-smoke \
  --input-turn latest --max-invocations 1 --max-operations 1 \
  --max-runtime-ms 2000 2>&1)"
runtime_exit=$?
set -e
[[ "$runtime_exit" -eq 0 || "$runtime_exit" -eq 1 ]]
after_failure="$("$YAI_BIN" case conversation turn show case:i01-smoke latest \
  --participant participant:operator)"
grep -Fq "turn_id: $failure_turn" <<<"$after_failure"
grep -Fq 'content_integrity: verified' <<<"$after_failure"

printf 'multipart_conversation: pass\n'
printf 'turn_commit_before_provider: pass\n'
printf 'original_derived_provenance: pass\n'
printf 'provider_failure_preserves_turn: pass\n'
