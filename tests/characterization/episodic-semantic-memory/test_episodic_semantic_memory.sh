#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
YAI_BIN="$ROOT/target/debug/yai"
RUN_ROOT="$(mktemp -d)"
CHAT_PID=""
EMBED_PID=""

cleanup() {
  for pid in "$CHAT_PID" "$EMBED_PID"; do
    if [[ -n "$pid" ]]; then
      kill "$pid" 2>/dev/null || true
      wait "$pid" 2>/dev/null || true
    fi
  done
  rm -rf "$RUN_ROOT"
}
trap cleanup EXIT

python3 "$ROOT/tests/fixtures/provider_governance_server.py" \
  --mode memory_w20 --model memory-w20-cognition-fixture --requests 96 \
  >"$RUN_ROOT/chat.out" 2>"$RUN_ROOT/chat.err" &
CHAT_PID=$!
python3 "$ROOT/tests/fixtures/memory_embedding_server.py" \
  --model memory-w20-encoder --count-file "$RUN_ROOT/embed.count" \
  >"$RUN_ROOT/embed.out" 2>"$RUN_ROOT/embed.err" &
EMBED_PID=$!
for _ in $(seq 1 100); do
  [[ -s "$RUN_ROOT/chat.out" && -s "$RUN_ROOT/embed.out" ]] && break
  sleep 0.05
done
CHAT_PORT="$(head -1 "$RUN_ROOT/chat.out")"
EMBED_PORT="$(head -1 "$RUN_ROOT/embed.out")"
[[ "$CHAT_PORT" =~ ^[0-9]+$ && "$EMBED_PORT" =~ ^[0-9]+$ ]]

export YAI_HOME="$RUN_ROOT/yai-home"
"$YAI_BIN" init --tenant tenant:w20-smoke --organization organization:cli-product >/dev/null

"$YAI_BIN" provider add \
  --tenant tenant:w20-smoke --provider-key memory-w20-cognition \
  --endpoint "http://127.0.0.1:$CHAT_PORT" --model memory-w20-cognition-fixture \
  --credential-env YAI_W20_OPTIONAL_CHAT_KEY --locality loopback >/dev/null
"$YAI_BIN" provider qualify --tenant tenant:w20-smoke \
  --provider-key memory-w20-cognition >/dev/null
"$YAI_BIN" provider trust approve --tenant tenant:w20-smoke \
  --provider-key memory-w20-cognition >/dev/null

"$YAI_BIN" provider add \
  --tenant tenant:w20-smoke --provider-key memory-w20-encoder \
  --endpoint "http://127.0.0.1:$EMBED_PORT" --model memory-w20-encoder \
  --credential-env YAI_W20_OPTIONAL_ENCODER_KEY --locality loopback >/dev/null
"$YAI_BIN" provider qualify --tenant tenant:w20-smoke \
  --provider-key memory-w20-encoder --embedding | grep -Fq 'TextEmbedding'
"$YAI_BIN" provider trust approve --tenant tenant:w20-smoke \
  --provider-key memory-w20-encoder >/dev/null

"$YAI_BIN" case create case:w20-memory --tenant tenant:w20-smoke >/dev/null
"$YAI_BIN" case participant role add case:w20-memory \
  --participant participant:model --role model-executor >/dev/null
"$YAI_BIN" case participant link-principal case:w20-memory \
  --principal self --participant participant:model >/dev/null
"$YAI_BIN" case participant role add case:w20-memory \
  --participant participant:model --role operation-proposer >/dev/null
"$YAI_BIN" case participant view admit case:w20-memory \
  --participant participant:model --consumer model --view model_context >/dev/null
"$YAI_BIN" case provider bind case:w20-memory \
  --participant participant:model --provider-key memory-w20-cognition \
  --failover safe_only --max-attempts 1 >/dev/null
mkdir -p "$RUN_ROOT/resource/allowed"
"$YAI_BIN" case resource attach filesystem case:w20-memory \
  --resource resource:w20-memory --root "$RUN_ROOT/resource" \
  --allow-prefix allowed --policy-owner participant:model --max-bytes 4096 >/dev/null

"$YAI_BIN" policy ingest "$ROOT/tests/fixtures/cli-product-policy.json" \
  --tenant tenant:w20-smoke --validate --publish \
  --reason 'W20 fixture validation and publication' >/dev/null
"$YAI_BIN" case policy bind case:w20-memory --policy-key cli.product.governed \
  --reason 'W20 episodic semantic memory fixture' >/dev/null

denied="$("$YAI_BIN" case run case:w20-memory \
  --participant participant:model --resource resource:w20-memory \
  --prompt 'denied attempt: preserve the denied path exactly' \
  --max-invocations 1 --max-operations 1 --stop-on-deny --max-runtime-ms 5000)"
grep -Fq 'runtime_status: Denied' <<<"$denied"
test ! -e "$RUN_ROOT/resource/denied/blocked.txt"

first_write="$("$YAI_BIN" case run case:w20-memory \
  --participant participant:model --resource resource:w20-memory \
  --prompt 'initial write 4187 for ORCHID-W20' \
  --max-invocations 2 --max-operations 2 --max-runtime-ms 5000)"
grep -Fq 'runtime_status: Completed' <<<"$first_write"
grep -Fq 'numeric fact 4187' "$RUN_ROOT/resource/allowed/codename.txt"

replacement="$("$YAI_BIN" case run case:w20-memory \
  --participant participant:model --resource resource:w20-memory \
  --prompt 'replacement 4188 for ORCHID-W20' \
  --max-invocations 2 --max-operations 2 --max-runtime-ms 5000)"
grep -Fq 'runtime_status: Completed' <<<"$replacement"
grep -Fq 'numeric fact 4188' "$RUN_ROOT/resource/allowed/codename.txt"

set +e
provider_claim="$("$YAI_BIN" case run case:w20-memory \
  --participant participant:model --resource resource:w20-memory \
  --prompt '9999 provider-only contradictory statement; do not create an Operation' \
  --max-invocations 1 --max-operations 1 --max-runtime-ms 5000 2>&1)"
provider_claim_exit=$?
set -e
[[ "$provider_claim_exit" -eq 0 || "$provider_claim_exit" -eq 1 ]]

episodes_before="$("$YAI_BIN" case memory episodes show case:w20-memory \
  --participant participant:model --json)"
grep -Fq 'yai.memory_episode.v1' <<<"$episodes_before"
grep -Fq 'denied' <<<"$episodes_before"
grep -Fq 'completed' <<<"$episodes_before"
semantic_before="$("$YAI_BIN" case memory semantic show case:w20-memory \
  --participant participant:model --include-historical --json)"
grep -Fq 'provider_originated_claim' <<<"$semantic_before"
grep -Fq '9999' <<<"$semantic_before"

hierarchy_before="$("$YAI_BIN" case memory hierarchy show case:w20-memory \
  --participant participant:model --json)"
HIERARCHY_BEFORE="$(python3 -c 'import json,sys; print(json.load(sys.stdin)["data"]["value"]["hierarchy_id"])' <<<"$hierarchy_before")"

consolidation="$("$YAI_BIN" case memory consolidate case:w20-memory \
  --participant participant:model --json)"
grep -Fq 'yai.memory_consolidation_result.v1' <<<"$consolidation"
grep -Fq 'rebuild_requires_reinference' <<<"$consolidation"
CONSOLIDATION_INPUT="$(python3 -c 'import json,sys; print(json.load(sys.stdin)["data"]["value"]["consolidation_input_id"])' <<<"$consolidation")"
PROVIDER_RESULT="$(python3 -c 'import json,sys; print(json.load(sys.stdin)["data"]["value"]["provider_result_id"])' <<<"$consolidation")"

semantic_after="$("$YAI_BIN" case memory semantic show case:w20-memory \
  --participant participant:model --include-historical --json)"
grep -Fq 'evidence_bound_inference' <<<"$semantic_after"
grep -Fq '4188' <<<"$semantic_after"
grep -Fq '9999' <<<"$semantic_after"
contradictions="$("$YAI_BIN" case memory contradictions case:w20-memory \
  --participant participant:model --json)"
grep -Fq 'structural_value_conflict' <<<"$contradictions"
grep -Fq 'unresolved' <<<"$contradictions"

hierarchy_after="$("$YAI_BIN" case memory hierarchy show case:w20-memory \
  --participant participant:model --json)"
HIERARCHY_AFTER="$(python3 -c 'import json,sys; print(json.load(sys.stdin)["data"]["value"]["hierarchy_id"])' <<<"$hierarchy_after")"
rebuild_hierarchy="$("$YAI_BIN" case memory hierarchy rebuild case:w20-memory \
  --participant participant:model --json)"
HIERARCHY_REBUILT="$(python3 -c 'import json,sys; print(json.load(sys.stdin)["data"]["value"]["hierarchy_id"])' <<<"$rebuild_hierarchy")"
[[ "$HIERARCHY_AFTER" == "$HIERARCHY_REBUILT" ]]
grep -Fq '"canonical_transition_mutated":false' <<<"$rebuild_hierarchy"

build_output="$("$YAI_BIN" case memory index build case:w20-memory \
  --encoder-provider-key memory-w20-encoder \
  --encoder-revision test-only-w20-v1 --dimension 4)"
PROFILE_ID="$(sed -n 's/^representation_profile_id: //p' <<<"$build_output")"
INDEX_ID="$(sed -n 's/^index_manifest_id: //p' <<<"$build_output")"
[[ "$PROFILE_ID" == memory-profile:* && "$INDEX_ID" == memory-index:* ]]
verify="$("$YAI_BIN" case memory index verify case:w20-memory)"
grep -Fq 'posture: current' <<<"$verify"
grep -Fq 'deep_plus_current_memory_hierarchy_source' <<<"$verify"

search="$("$YAI_BIN" case memory search case:w20-memory \
  --participant participant:model \
  --query 'ORCHID-W20 numeric fact 4188 9999 denied final outcome' \
  --limit 16 --json)"
grep -Fq 'yai.retrieval_set.v3' <<<"$search"
grep -Fq '"memory_family":"episodic"' <<<"$search"
grep -Fq '"memory_family":"semantic"' <<<"$search"
grep -Fq 'evidence_bound_inference' <<<"$search"
grep -Fq "$INDEX_ID" <<<"$search"

"$YAI_BIN" case create case:w20-isolated --tenant tenant:w20-smoke >/dev/null
"$YAI_BIN" case participant role add case:w20-isolated \
  --participant participant:model --role model-executor >/dev/null
"$YAI_BIN" case participant view admit case:w20-isolated \
  --participant participant:model --consumer model --view model_context >/dev/null
isolated="$("$YAI_BIN" case memory search case:w20-isolated \
  --participant participant:model --query 'ORCHID-W20 4188 9999' --limit 8)"
grep -Fq 'selected: 0' <<<"$isolated"

drop="$("$YAI_BIN" case memory index drop case:w20-memory)"
grep -Fq 'semantic_continuity_preserved: yes' <<<"$drop"
fallback="$("$YAI_BIN" case memory search case:w20-memory \
  --participant participant:model --query 'ORCHID-W20 4188' \
  --limit 8)"
grep -Fq 'plane: lexical_bm25 available:false' <<<"$fallback"
grep -Fq 'plane: vector_exact_cosine available:false' <<<"$fallback"
grep -Eq 'selected: [1-9][0-9]*' <<<"$fallback"

hierarchy_drop="$("$YAI_BIN" case memory hierarchy drop case:w20-memory \
  --participant participant:model --json)"
grep -Fq '"rebuild_requires_provider_invocation":false' <<<"$hierarchy_drop"
semantic_rebuilt="$("$YAI_BIN" case memory semantic show case:w20-memory \
  --participant participant:model --include-historical --json)"
grep -Fq "$PROVIDER_RESULT" <<<"$semantic_rebuilt"

rebuilt="$("$YAI_BIN" case memory index rebuild case:w20-memory \
  --encoder-provider-key memory-w20-encoder \
  --encoder-revision test-only-w20-v1 --dimension 4)"
REBUILT_INDEX="$(sed -n 's/^index_manifest_id: //p' <<<"$rebuilt")"
[[ "$INDEX_ID" == "$REBUILT_INDEX" ]]

final_turn="$("$YAI_BIN" case run case:w20-memory \
  --participant participant:model --resource resource:w20-memory \
  --prompt 'Qual è il valore operativo finale, quale valore precedente è stato sostituito, quale claim contraddittorio è apparso e su quali evidenze si basa questa distinzione?' \
  --max-invocations 1 --max-runtime-ms 5000)"
grep -Fq 'runtime_status: Completed' <<<"$final_turn"
"$YAI_BIN" case context show case:w20-memory --kind projection | \
  grep -Fq 'artifact_kind: projection'
"$YAI_BIN" case context show case:w20-memory --kind context-frame | \
  grep -Fq 'artifact_kind: context_frame'
retrieval="$("$YAI_BIN" case memory retrieval show case:w20-memory \
  --json)"
grep -Fq 'yai.retrieval_set.v3' <<<"$retrieval"
grep -Fq 'memory-hierarchy:' <<<"$retrieval"

printf 'episodic_semantic_memory: pass\n'
printf 'episode_schema: yai.memory_episode.v1\n'
printf 'semantic_schema: yai.semantic_memory_assertion.v1\n'
printf 'retrieval_schema: yai.retrieval_set.v3\n'
printf 'consolidation_input: %s\n' "$CONSOLIDATION_INPUT"
printf 'consolidation_provider_result: %s\n' "$PROVIDER_RESULT"
printf 'hierarchy_before_consolidation: %s\n' "$HIERARCHY_BEFORE"
printf 'hierarchy_after_consolidation: %s\n' "$HIERARCHY_AFTER"
printf 'hierarchy_rebuild_exact: true\n'
printf 'provider_reinference_on_rebuild: zero\n'
printf 'cross_case_isolation: true\n'
printf 'index_drop_preserved_hierarchy: true\n'
