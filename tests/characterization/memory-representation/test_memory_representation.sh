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
  --mode memory --model memory-cognition-fixture --requests 32 \
  >"$RUN_ROOT/chat.out" 2>"$RUN_ROOT/chat.err" &
CHAT_PID=$!
python3 "$ROOT/tests/fixtures/memory_embedding_server.py" \
  --model memory-fixture-encoder --count-file "$RUN_ROOT/embed.count" --delay-ms 25 \
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
"$YAI_BIN" init --tenant tenant:w19-smoke --organization organization:cli-product >/dev/null

chat_add=$("$YAI_BIN" provider add \
  --tenant tenant:w19-smoke --provider-key memory-cognition \
  --endpoint "http://127.0.0.1:$CHAT_PORT" --model memory-cognition-fixture \
  --locality loopback)
CHAT_TARGET=$(sed -n 's/^target_id: //p' <<<"$chat_add")
[[ "$CHAT_TARGET" == provider-target:* ]]
"$YAI_BIN" provider qualify "$CHAT_TARGET" >/dev/null
"$YAI_BIN" provider trust approve "$CHAT_TARGET" >/dev/null

encoder_add=$("$YAI_BIN" provider add \
  --tenant tenant:w19-smoke --provider-key memory-encoder \
  --endpoint "http://127.0.0.1:$EMBED_PORT" --model memory-fixture-encoder \
  --locality loopback)
ENCODER_TARGET=$(sed -n 's/^target_id: //p' <<<"$encoder_add")
[[ "$ENCODER_TARGET" == provider-target:* ]]
encoder_qualification=$("$YAI_BIN" provider qualify "$ENCODER_TARGET" --embedding)
grep -Fq 'TextEmbedding' <<<"$encoder_qualification"
grep -Fq 'embedding_dimension: 4' <<<"$encoder_qualification"
"$YAI_BIN" provider trust approve "$ENCODER_TARGET" >/dev/null
QUALIFICATION_ENCODER_REQUESTS=$(<"$RUN_ROOT/embed.count")

"$YAI_BIN" case create case:w19-memory --tenant tenant:w19-smoke >/dev/null
"$YAI_BIN" case participant role add case:w19-memory \
  --participant participant:model --role model-executor >/dev/null
"$YAI_BIN" case participant role add case:w19-memory \
  --participant participant:model --role operation-proposer >/dev/null
"$YAI_BIN" case participant view admit case:w19-memory \
  --participant participant:model --consumer model --view model_context >/dev/null
"$YAI_BIN" case provider bind case:w19-memory \
  --participant participant:model --target "$CHAT_TARGET" \
  --failover safe_only --max-attempts 2 >/dev/null
mkdir -p "$RUN_ROOT/resource/allowed"
"$YAI_BIN" case resource attach filesystem case:w19-memory \
  --resource resource:w19-memory --root "$RUN_ROOT/resource" \
  --allow-prefix allowed --policy-owner participant:model --max-bytes 1024 >/dev/null
policy_ingest=$("$YAI_BIN" policy ingest \
  "$ROOT/tests/fixtures/cli-product-policy.json" --tenant tenant:w19-smoke)
POLICY_ID=$(sed -n 's/^artifact_id: //p' <<<"$policy_ingest" | head -1)
[[ "$POLICY_ID" == policy-artifact:* ]]
"$YAI_BIN" policy validate "$POLICY_ID" --reason 'W19 fixture validation' >/dev/null
"$YAI_BIN" policy publish "$POLICY_ID" --reason 'W19 fixture publication' >/dev/null
"$YAI_BIN" case policy bind case:w19-memory --artifact "$POLICY_ID" \
  --reason 'W19 memory representation fixture' >/dev/null
run_output=$("$YAI_BIN" case run case:w19-memory \
  --participant participant:model --resource resource:w19-memory \
  --prompt 'complete the fixed W19 memory fixture turn' \
  --max-invocations 2 --max-operations 2 --max-runtime-ms 5000)
grep -Fq 'runtime_status: Completed' <<<"$run_output"
grep -Fq 'operations: 1' <<<"$run_output"
grep -Fq 'project codename ORCHID-731' "$RUN_ROOT/resource/allowed/codename.txt"

before_show=$("$YAI_BIN" case memory show case:w19-memory)
grep -Fq 'canonical_source_generation:' <<<"$before_show"
grep -Eq 'operational_memory_entries: [1-9][0-9]*' <<<"$before_show"

build_output=$("$YAI_BIN" case memory index build case:w19-memory \
  --encoder-target "$ENCODER_TARGET" \
  --encoder-revision test-only-controlled-v1 --dimension 4)
PROFILE_ID=$(sed -n 's/^representation_profile_id: //p' <<<"$build_output")
INDEX_ID=$(sed -n 's/^index_manifest_id: //p' <<<"$build_output")
[[ "$PROFILE_ID" == memory-profile:* && "$INDEX_ID" == memory-index:* ]]
grep -Fq 'canonical_transition_mutated: no' <<<"$build_output"
POST_BUILD_ENCODER_REQUESTS=$(<"$RUN_ROOT/embed.count")
[[ "$POST_BUILD_ENCODER_REQUESTS" -eq $((QUALIFICATION_ENCODER_REQUESTS + 1)) ]]

status_output=$("$YAI_BIN" case memory index status case:w19-memory)
grep -Fq "index: $INDEX_ID" <<<"$status_output"
grep -Fq 'posture:current' <<<"$status_output"
grep -Fq 'format:yai.derived_memory_store.v2' <<<"$status_output"
verify_output=$("$YAI_BIN" case memory index verify case:w19-memory --profile "$PROFILE_ID")
grep -Fq 'posture: current' <<<"$verify_output"
grep -Fq 'validation: deep_plus_current_operational_memory_source' <<<"$verify_output"

# The profile lock and equivalent-build recheck happen before external work.
# Thirty-two independent rebuild callers therefore reuse the sealed build and
# must not issue another full-corpus embedding request.
REBUILD_PIDS=()
for worker in $(seq 1 32); do
  "$YAI_BIN" case memory index rebuild case:w19-memory \
    --encoder-target "$ENCODER_TARGET" \
    --encoder-revision test-only-controlled-v1 --dimension 4 \
    >"$RUN_ROOT/rebuild-$worker.out" 2>"$RUN_ROOT/rebuild-$worker.err" &
  REBUILD_PIDS+=("$!")
done
for pid in "${REBUILD_PIDS[@]}"; do
  wait "$pid"
done
for worker in $(seq 1 32); do
  grep -Fq 'memory_index_rebuild: existing_equivalent' "$RUN_ROOT/rebuild-$worker.out"
  grep -Fq "index_manifest_id: $INDEX_ID" "$RUN_ROOT/rebuild-$worker.out"
done
POST_STAMPEDE_ENCODER_REQUESTS=$(<"$RUN_ROOT/embed.count")
[[ "$POST_STAMPEDE_ENCODER_REQUESTS" -eq "$POST_BUILD_ENCODER_REQUESTS" ]]

search_output=$("$YAI_BIN" case memory search case:w19-memory \
  --participant participant:model --query 'provider model complete' \
  --profile "$PROFILE_ID" --limit 8)
grep -Fq 'plane: lexical_bm25 available:true' <<<"$search_output"
grep -Fq 'plane: vector_exact_cosine available:true' <<<"$search_output"
grep -Eq '^1[[:space:]]+provider_claim[[:space:]]+' <<<"$search_output"
"$YAI_BIN" case memory retrieval show case:w19-memory \
  --profile "$PROFILE_ID" --json | grep -Fq 'yai.retrieval_set.v2'

"$YAI_BIN" case create case:w19-isolated --tenant tenant:w19-smoke >/dev/null
"$YAI_BIN" case participant role add case:w19-isolated \
  --participant participant:model --role model-executor >/dev/null
"$YAI_BIN" case participant view admit case:w19-isolated \
  --participant participant:model --consumer model --view model_context >/dev/null
isolated=$("$YAI_BIN" case memory search case:w19-isolated \
  --participant participant:model --query 'provider model complete' --limit 8)
grep -Fq 'selected: 0' <<<"$isolated"
if grep -Fq "$INDEX_ID" <<<"$isolated"; then
  printf 'cross_case_index_leak: true\n' >&2
  exit 1
fi

drop_output=$("$YAI_BIN" case memory index drop case:w19-memory --profile "$PROFILE_ID")
grep -Fq 'semantic_continuity_preserved: yes' <<<"$drop_output"
post_drop=$("$YAI_BIN" case memory show case:w19-memory)
grep -Fq 'indexes: 0' <<<"$post_drop"
grep -Eq 'operational_memory_entries: [1-9][0-9]*' <<<"$post_drop"
missing_verify=$("$YAI_BIN" case memory index verify case:w19-memory --profile "$PROFILE_ID")
grep -Fq 'posture: missing' <<<"$missing_verify"

rebuild_output=$("$YAI_BIN" case memory index rebuild case:w19-memory \
  --encoder-target "$ENCODER_TARGET" \
  --encoder-revision test-only-controlled-v1 --dimension 4)
REBUILT_INDEX_ID=$(sed -n 's/^index_manifest_id: //p' <<<"$rebuild_output")
[[ "$REBUILT_INDEX_ID" == "$INDEX_ID" ]]
repeat=$("$YAI_BIN" case memory search case:w19-memory \
  --participant participant:model --query 'provider model complete' \
  --profile "$PROFILE_ID" --limit 8)
grep -Fq 'plane: vector_exact_cosine available:true' <<<"$repeat"

# Provider selection is a canonical Transition and therefore makes the just
# built derived index stale. The runtime may refresh that exact profile only
# through its separately qualified loopback encoder before compiling the
# existing Projection -> ContextFrame path.
export YAI_MEMORY_PROFILE_ID="$PROFILE_ID"
second_run=$("$YAI_BIN" case run case:w19-memory \
  --participant participant:model --resource resource:w19-memory \
  --prompt 'recall the prior provider result from qualified Case memory' \
  --max-invocations 1 --max-runtime-ms 5000)
grep -Fq 'runtime_status: Completed' <<<"$second_run"
runtime_retrieval=$("$YAI_BIN" case memory retrieval show case:w19-memory \
  --profile "$PROFILE_ID" --json)
grep -Fq '"schema":"yai.retrieval_set.v2"' <<<"$runtime_retrieval"
grep -Fq '"plane":"vector_exact_cosine"' <<<"$runtime_retrieval"
grep -Fq '"available":true' <<<"$runtime_retrieval"
grep -Fq '"index_manifest_id":"memory-index:' <<<"$runtime_retrieval"
grep -Fq 'finalized_observed_consequence' <<<"$runtime_retrieval"

printf 'memory_representation_characterization: pass\n'
printf 'corpus_profile_index: %s %s\n' "$PROFILE_ID" "$INDEX_ID"
printf 'qualified_planes: exact_operational lexical_bm25 vector_exact_cosine\n'
printf 'ann_posture: deferred_exact_scan_within_bound\n'
printf 'cross_case_isolation: true\n'
printf 'drop_preserved_case_truth: true\n'
printf 'content_identical_rebuild: true\n'
printf 'runtime_context_used_current_w19_index: true\n'
printf 'physical_store: yai.derived_memory_store.v2\n'
printf 'deep_source_verify: true\n'
printf 'concurrent_rebuilders: 32\n'
printf 'duplicate_embedding_requests: 0\n'
