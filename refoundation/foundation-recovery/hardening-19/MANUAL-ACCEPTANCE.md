# MANUAL ACCEPTANCE — ZERO TO USE CASE

Run this from the YAI repository root with Bash, `curl`, `python3`, a Rust
toolchain, a running operator-managed YVEX endpoint exposing the exact chosen
DeepSeek model, and a separate running OpenAI-compatible embedding endpoint on
loopback. The sequence creates and later deletes only a fresh directory below
`/tmp`; it does not inspect or administer YVEX and does not edit derived-store
files directly.

Required environment variables are exactly:

- `YAI_EXTERNAL_PROVIDER_BASE_URL`
- `YAI_EXTERNAL_PROVIDER_MODEL`
- `YAI_MEMORY_ENCODER_BASE_URL`
- `YAI_MEMORY_ENCODER_MODEL`
- `YAI_MEMORY_ENCODER_REVISION`
- `YAI_MEMORY_ENCODER_DIMENSION`

Optional credentials are read from `YAI_EXTERNAL_PROVIDER_API_KEY` and
`YAI_MEMORY_ENCODER_API_KEY`. For non-loopback YVEX, set
`YAI_EXTERNAL_PROVIDER_LOCALITY` to `private_network` or `remote`. The memory
encoder must remain loopback. Expected important postures are: YVEX and encoder
model readiness succeeds; encoder qualification reports `TextEmbedding` and the
declared dimension; the denied path creates no file; both admitted writes are
observed; index A and B are separate and current; `verify` reports
`deep_plus_current_operational_memory_source`; eight equivalent rebuilds retain
one manifest; cross-Case search selects zero; dropping A preserves Case memory;
DeepSeek still runs through qualified non-index fallback; rebuild restores A;
and the final recall has inspectable RetrievalSet, Projection, ContextFrame,
ProviderSelection, and ProviderResult identities without index internals.

```bash
set -euo pipefail

: "${YAI_EXTERNAL_PROVIDER_BASE_URL:?set the running YVEX OpenAI-compatible base URL}"
: "${YAI_EXTERNAL_PROVIDER_MODEL:?set the exact DeepSeek model exposed by YVEX}"
: "${YAI_MEMORY_ENCODER_BASE_URL:?set a running loopback OpenAI-compatible embedding base URL}"
: "${YAI_MEMORY_ENCODER_MODEL:?set the exact embedding model ID}"
: "${YAI_MEMORY_ENCODER_REVISION:?set the operator-declared encoder revision/profile}"
: "${YAI_MEMORY_ENCODER_DIMENSION:?set the exact embedding dimension}"

case "$YAI_MEMORY_ENCODER_DIMENSION" in
  ''|*[!0-9]*) echo 'YAI_MEMORY_ENCODER_DIMENSION must be a positive integer' >&2; exit 2 ;;
esac
if (( YAI_MEMORY_ENCODER_DIMENSION < 1 || YAI_MEMORY_ENCODER_DIMENSION > 4096 )); then
  echo 'YAI_MEMORY_ENCODER_DIMENSION must be between 1 and 4096' >&2
  exit 2
fi

api_url() {
  local base="${1%/}"
  local leaf="$2"
  if [[ "$base" == */v1 ]]; then
    printf '%s/%s\n' "$base" "$leaf"
  else
    printf '%s/v1/%s\n' "$base" "$leaf"
  fi
}

YAI_EXTERNAL_PROVIDER_LOCALITY="${YAI_EXTERNAL_PROVIDER_LOCALITY:-}"
case "$YAI_EXTERNAL_PROVIDER_BASE_URL" in
  http://127.0.0.1:*|http://localhost:*|http://\[::1\]:*|https://127.0.0.1:*|https://localhost:*|https://\[::1\]:*)
    YAI_EXTERNAL_PROVIDER_LOCALITY=loopback
    ;;
  *)
    : "${YAI_EXTERNAL_PROVIDER_LOCALITY:?set private_network or remote for non-loopback YVEX}"
    case "$YAI_EXTERNAL_PROVIDER_LOCALITY" in
      private_network|remote) ;;
      *) echo 'YAI_EXTERNAL_PROVIDER_LOCALITY must be private_network or remote' >&2; exit 2 ;;
    esac
    ;;
esac

python3 - "$YAI_MEMORY_ENCODER_BASE_URL" <<'PY'
import ipaddress
import socket
import sys
from urllib.parse import urlparse

host = urlparse(sys.argv[1]).hostname
if not host:
    raise SystemExit("YAI_MEMORY_ENCODER_BASE_URL has no host")
addresses = {item[4][0] for item in socket.getaddrinfo(host, None)}
if not addresses or not all(ipaddress.ip_address(value).is_loopback for value in addresses):
    raise SystemExit("H19 memory encoder must resolve only to loopback addresses")
print("encoder_locality: loopback")
PY

make build-rust
YAI_BIN="$PWD/target/debug/yai"
test -x "$YAI_BIN"

YAI_H19_RUN_ROOT="$(mktemp -d /tmp/yai-h19-manual.XXXXXX)"
export YAI_HOME="$YAI_H19_RUN_ROOT/yai-home"
RESOURCE_ROOT="$YAI_H19_RUN_ROOT/resource"
POLICY_SOURCE="$YAI_H19_RUN_ROOT/memory-policy.json"
mkdir -p "$RESOURCE_ROOT/allowed"

cleanup_h19() {
  case "$YAI_H19_RUN_ROOT" in
    /tmp/yai-h19-manual.*) rm -rf "$YAI_H19_RUN_ROOT" ;;
    *) echo "refusing cleanup outside bounded H19 temp root: $YAI_H19_RUN_ROOT" >&2; return 1 ;;
  esac
}
trap cleanup_h19 EXIT

COGNITION_CURL_HEADERS=()
COGNITION_CREDENTIAL_ARGS=()
if [[ -n "${YAI_EXTERNAL_PROVIDER_API_KEY:-}" ]]; then
  COGNITION_CURL_HEADERS=(-H "Authorization: Bearer $YAI_EXTERNAL_PROVIDER_API_KEY")
  COGNITION_CREDENTIAL_ARGS=(--credential-ref env:YAI_EXTERNAL_PROVIDER_API_KEY)
fi
ENCODER_CURL_HEADERS=()
ENCODER_CREDENTIAL_ARGS=()
if [[ -n "${YAI_MEMORY_ENCODER_API_KEY:-}" ]]; then
  ENCODER_CURL_HEADERS=(-H "Authorization: Bearer $YAI_MEMORY_ENCODER_API_KEY")
  ENCODER_CREDENTIAL_ARGS=(--credential-ref env:YAI_MEMORY_ENCODER_API_KEY)
fi

YVEX_MODELS="$YAI_H19_RUN_ROOT/yvex-models.json"
ENCODER_MODELS="$YAI_H19_RUN_ROOT/encoder-models.json"
if ! curl -fsS "${COGNITION_CURL_HEADERS[@]}" \
  "$(api_url "$YAI_EXTERNAL_PROVIDER_BASE_URL" models)" -o "$YVEX_MODELS"; then
  echo 'YVEX readiness failed: start the operator-managed endpoint and expose the configured DeepSeek model' >&2
  exit 3
fi
python3 - "$YVEX_MODELS" "$YAI_EXTERNAL_PROVIDER_MODEL" <<'PY'
import json
import sys

with open(sys.argv[1], encoding="utf-8") as handle:
    data = json.load(handle)
ids = [item.get("id") for item in data.get("data", []) if isinstance(item, dict)]
if sys.argv[2] not in ids:
    raise SystemExit(f"configured DeepSeek model is not exposed: {sys.argv[2]}")
print(f"yvex_model_ready: {sys.argv[2]}")
PY
if ! curl -fsS "${ENCODER_CURL_HEADERS[@]}" \
  "$(api_url "$YAI_MEMORY_ENCODER_BASE_URL" models)" -o "$ENCODER_MODELS"; then
  echo 'Embedding readiness failed: start the operator-managed loopback encoder and expose the configured model' >&2
  exit 3
fi
python3 - "$ENCODER_MODELS" "$YAI_MEMORY_ENCODER_MODEL" <<'PY'
import json
import sys

with open(sys.argv[1], encoding="utf-8") as handle:
    data = json.load(handle)
ids = [item.get("id") for item in data.get("data", []) if isinstance(item, dict)]
if sys.argv[2] not in ids:
    raise SystemExit(f"configured embedding model is not exposed: {sys.argv[2]}")
print(f"embedding_model_ready: {sys.argv[2]}")
PY

"$YAI_BIN" init \
  --tenant tenant:memory-h19-acceptance \
  --organization organization:memory-h19-acceptance
"$YAI_BIN" doctor
WHOAMI="$("$YAI_BIN" identity whoami)"
printf '%s\n' "$WHOAMI"
PRINCIPAL_ID="$(sed -n 's/^principal_id: //p' <<<"$WHOAMI" | head -1)"
test -n "$PRINCIPAL_ID"

COGNITION_ADD="$("$YAI_BIN" provider add \
  --tenant tenant:memory-h19-acceptance \
  --provider-key yvex-deepseek-h19 \
  --endpoint "$YAI_EXTERNAL_PROVIDER_BASE_URL" \
  --model "$YAI_EXTERNAL_PROVIDER_MODEL" \
  --locality "$YAI_EXTERNAL_PROVIDER_LOCALITY" \
  "${COGNITION_CREDENTIAL_ARGS[@]}")"
printf '%s\n' "$COGNITION_ADD"
COGNITION_TARGET="$(sed -n 's/^target_id: //p' <<<"$COGNITION_ADD")"
test -n "$COGNITION_TARGET"
"$YAI_BIN" provider qualify "$COGNITION_TARGET"
"$YAI_BIN" provider trust approve "$COGNITION_TARGET"

ENCODER_ADD="$("$YAI_BIN" provider add \
  --tenant tenant:memory-h19-acceptance \
  --provider-key loopback-memory-encoder-h19 \
  --endpoint "$YAI_MEMORY_ENCODER_BASE_URL" \
  --model "$YAI_MEMORY_ENCODER_MODEL" \
  --locality loopback \
  "${ENCODER_CREDENTIAL_ARGS[@]}")"
printf '%s\n' "$ENCODER_ADD"
ENCODER_TARGET="$(sed -n 's/^target_id: //p' <<<"$ENCODER_ADD")"
test -n "$ENCODER_TARGET"
ENCODER_QUALIFICATION="$("$YAI_BIN" provider qualify "$ENCODER_TARGET" --embedding)"
printf '%s\n' "$ENCODER_QUALIFICATION"
grep -F 'TextEmbedding' <<<"$ENCODER_QUALIFICATION"
grep -F "embedding_dimension: $YAI_MEMORY_ENCODER_DIMENSION" <<<"$ENCODER_QUALIFICATION"
"$YAI_BIN" provider trust approve "$ENCODER_TARGET"

"$YAI_BIN" case create case:memory-h19-acceptance --tenant tenant:memory-h19-acceptance
"$YAI_BIN" case participant link-principal case:memory-h19-acceptance \
  --principal "$PRINCIPAL_ID" --participant participant:deepseek
"$YAI_BIN" case participant role add case:memory-h19-acceptance \
  --participant participant:deepseek --role model-executor
"$YAI_BIN" case participant role add case:memory-h19-acceptance \
  --participant participant:deepseek --role operation-proposer
"$YAI_BIN" case participant view admit case:memory-h19-acceptance \
  --participant participant:deepseek --consumer model --view model_context
"$YAI_BIN" case participant list case:memory-h19-acceptance
"$YAI_BIN" case provider bind case:memory-h19-acceptance \
  --participant participant:deepseek --target "$COGNITION_TARGET" \
  --failover safe_only --max-attempts 1
"$YAI_BIN" case provider show case:memory-h19-acceptance

"$YAI_BIN" case resource attach filesystem case:memory-h19-acceptance \
  --resource resource:memory-h19-acceptance \
  --root "$RESOURCE_ROOT" --allow-prefix allowed \
  --policy-owner participant:deepseek --max-bytes 4096

cat >"$POLICY_SOURCE" <<'POLICY'
{
  "schema": "yai.policy_source_input.v4",
  "policy_key": "manual.memory.h19.acceptance",
  "source_version": "1",
  "owner_ref": "organization:memory-h19-acceptance",
  "source_origin": {
    "source_system": "manual_acceptance",
    "source_uri": "manual://hardening-19/memory-acceptance"
  },
  "validity": {"mode": "unbounded"},
  "rules": [
    {
      "kind": "operation_restriction",
      "rule_id": "filesystem-posture",
      "operation_kind": "filesystem.write",
      "resource_kind": "filesystem",
      "effect": "allow",
      "reason": "H19 bounded filesystem write"
    },
    {
      "kind": "authority_requirement",
      "rule_id": "filesystem-proposer",
      "operation_kind": "filesystem.write",
      "resource_kind": "filesystem",
      "subject": "proposer",
      "required_role": "operation-proposer",
      "reason": "explicit Case proposer"
    },
    {
      "kind": "evidence_obligation",
      "rule_id": "filesystem-source",
      "operation_kind": "filesystem.write",
      "resource_kind": "filesystem",
      "obligation": "source_provenance",
      "reason": "canonical ProviderResult lineage"
    },
    {
      "kind": "evidence_obligation",
      "rule_id": "filesystem-post",
      "operation_kind": "filesystem.write",
      "resource_kind": "filesystem",
      "obligation": "post_observation",
      "reason": "observed final consequence"
    }
  ]
}
POLICY

POLICY_INGEST="$("$YAI_BIN" policy ingest "$POLICY_SOURCE" --tenant tenant:memory-h19-acceptance)"
printf '%s\n' "$POLICY_INGEST"
POLICY_ID="$(sed -n 's/^artifact_id: //p' <<<"$POLICY_INGEST" | head -1)"
test -n "$POLICY_ID"
"$YAI_BIN" policy validate "$POLICY_ID" --reason 'H19 manual validation'
"$YAI_BIN" policy publish "$POLICY_ID" --reason 'H19 manual publication'
"$YAI_BIN" case policy bind case:memory-h19-acceptance \
  --artifact "$POLICY_ID" --reason 'H19 manual acceptance binding'
"$YAI_BIN" case policy show case:memory-h19-acceptance

set +e
DENIED_OUTPUT="$("$YAI_BIN" case run case:memory-h19-acceptance \
  --participant participant:deepseek \
  --resource resource:memory-h19-acceptance \
  --prompt 'Proponi esclusivamente filesystem.write sulla risorsa resource:memory-h19-acceptance al path denied/blocked.txt con contenuto DENIED-H19. Non cambiare path. Dopo la Decision, non inventare successo.' \
  --max-invocations 1 --max-operations 1 --stop-on-deny --max-runtime-ms 120000 2>&1)"
DENIED_EXIT=$?
set -e
printf '%s\n' "$DENIED_OUTPUT"
test "$DENIED_EXIT" -eq 0
grep -E 'runtime_status: (Denied|InvocationBudgetExhausted)' <<<"$DENIED_OUTPUT"
test ! -e "$RESOURCE_ROOT/denied/blocked.txt"

FIRST_WRITE="$("$YAI_BIN" case run case:memory-h19-acceptance \
  --participant participant:deepseek \
  --resource resource:memory-h19-acceptance \
  --prompt 'Proponi filesystem.write sulla risorsa resource:memory-h19-acceptance al path allowed/orchid-h19.txt. Il contenuto esatto deve essere: Project codename ORCHID-H19-731. Numeric fact 4187. Dopo la conseguenza osservata, termina con yai.case_runtime_turn.v1 outcome complete.' \
  --max-invocations 3 --max-operations 2 --max-runtime-ms 180000)"
printf '%s\n' "$FIRST_WRITE"
grep -F 'runtime_status: Completed' <<<"$FIRST_WRITE"
grep -F 'effect_outcome: Applied' <<<"$FIRST_WRITE"
grep -F 'Project codename ORCHID-H19-731. Numeric fact 4187.' "$RESOURCE_ROOT/allowed/orchid-h19.txt"

REPLACEMENT="$("$YAI_BIN" case run case:memory-h19-acceptance \
  --participant participant:deepseek \
  --resource resource:memory-h19-acceptance \
  --prompt 'Proponi filesystem.write sulla risorsa resource:memory-h19-acceptance allo stesso path allowed/orchid-h19.txt. Sostituisci il contenuto con: Project codename ORCHID-H19-731. Numeric fact 4188. Final outcome: replacement applied. Dopo la conseguenza osservata, termina con yai.case_runtime_turn.v1 outcome complete.' \
  --max-invocations 3 --max-operations 2 --max-runtime-ms 180000)"
printf '%s\n' "$REPLACEMENT"
grep -F 'runtime_status: Completed' <<<"$REPLACEMENT"
grep -F 'Project codename ORCHID-H19-731. Numeric fact 4188. Final outcome: replacement applied.' "$RESOURCE_ROOT/allowed/orchid-h19.txt"

"$YAI_BIN" case show case:memory-h19-acceptance --json
"$YAI_BIN" case memory show case:memory-h19-acceptance --json
INDEX_BUILD_A="$("$YAI_BIN" case memory index build case:memory-h19-acceptance \
  --encoder-target "$ENCODER_TARGET" \
  --encoder-revision "$YAI_MEMORY_ENCODER_REVISION" \
  --dimension "$YAI_MEMORY_ENCODER_DIMENSION")"
printf '%s\n' "$INDEX_BUILD_A"
PROFILE_A="$(sed -n 's/^representation_profile_id: //p' <<<"$INDEX_BUILD_A")"
INDEX_A="$(sed -n 's/^index_manifest_id: //p' <<<"$INDEX_BUILD_A")"
test -n "$PROFILE_A"
test -n "$INDEX_A"
export YAI_MEMORY_PROFILE_ID="$PROFILE_A"

"$YAI_BIN" case memory index status case:memory-h19-acceptance --json
VERIFY_A="$("$YAI_BIN" case memory index verify case:memory-h19-acceptance --profile "$PROFILE_A")"
printf '%s\n' "$VERIFY_A"
grep -F 'posture: current' <<<"$VERIFY_A"
grep -F 'physical_format: yai.derived_memory_store.v2' <<<"$VERIFY_A"
grep -F 'validation: deep_plus_current_operational_memory_source' <<<"$VERIFY_A"

SEARCH_ONE="$("$YAI_BIN" case memory search case:memory-h19-acceptance \
  --participant participant:deepseek \
  --query 'ORCHID-H19-731 4188 replacement applied denied blocked attempt' \
  --purpose inspection --resource resource:memory-h19-acceptance \
  --profile "$PROFILE_A" --limit 12 --json)"
SEARCH_TWO="$("$YAI_BIN" case memory search case:memory-h19-acceptance \
  --participant participant:deepseek \
  --query 'ORCHID-H19-731 4188 replacement applied denied blocked attempt' \
  --purpose inspection --resource resource:memory-h19-acceptance \
  --profile "$PROFILE_A" --limit 12 --json)"
python3 - "$SEARCH_ONE" "$SEARCH_TWO" "$INDEX_A" <<'PY'
import json
import sys

first = json.loads(sys.argv[1])["data"]["value"]
second = json.loads(sys.argv[2])["data"]["value"]
assert first["index_manifest_id"] == sys.argv[3]
assert second["index_manifest_id"] == sys.argv[3]
assert first["retrieval_id"] == second["retrieval_id"]
assert first["selected_memory_ids"] == second["selected_memory_ids"]
print(f"stable_retrieval_id: {first['retrieval_id']}")
print(f"stable_index_manifest_id: {first['index_manifest_id']}")
PY
grep -F 'lexical_bm25' <<<"$SEARCH_ONE"
grep -F 'vector_exact_cosine' <<<"$SEARCH_ONE"
grep -F 'finalized_observed_consequence' <<<"$SEARCH_ONE"

REBUILD_PIDS=()
for worker in $(seq 1 8); do
  "$YAI_BIN" case memory index rebuild case:memory-h19-acceptance \
    --encoder-target "$ENCODER_TARGET" \
    --encoder-revision "$YAI_MEMORY_ENCODER_REVISION" \
    --dimension "$YAI_MEMORY_ENCODER_DIMENSION" \
    >"$YAI_H19_RUN_ROOT/rebuild-$worker.out" \
    2>"$YAI_H19_RUN_ROOT/rebuild-$worker.err" &
  REBUILD_PIDS+=("$!")
done
for pid in "${REBUILD_PIDS[@]}"; do
  wait "$pid"
done
for worker in $(seq 1 8); do
  grep -F 'memory_index_rebuild: existing_equivalent' "$YAI_H19_RUN_ROOT/rebuild-$worker.out"
  grep -F "index_manifest_id: $INDEX_A" "$YAI_H19_RUN_ROOT/rebuild-$worker.out"
done
"$YAI_BIN" case memory index verify case:memory-h19-acceptance --profile "$PROFILE_A"

PROFILE_B_REVISION="${YAI_MEMORY_ENCODER_REVISION}-acceptance-profile-b"
INDEX_BUILD_B="$("$YAI_BIN" case memory index build case:memory-h19-acceptance \
  --encoder-target "$ENCODER_TARGET" \
  --encoder-revision "$PROFILE_B_REVISION" \
  --dimension "$YAI_MEMORY_ENCODER_DIMENSION")"
printf '%s\n' "$INDEX_BUILD_B"
PROFILE_B="$(sed -n 's/^representation_profile_id: //p' <<<"$INDEX_BUILD_B")"
INDEX_B="$(sed -n 's/^index_manifest_id: //p' <<<"$INDEX_BUILD_B")"
test -n "$PROFILE_B"
test -n "$INDEX_B"
test "$PROFILE_B" != "$PROFILE_A"
test "$INDEX_B" != "$INDEX_A"
PROFILE_STATUS="$("$YAI_BIN" case memory index status case:memory-h19-acceptance --json)"
printf '%s\n' "$PROFILE_STATUS"
grep -F "$PROFILE_A" <<<"$PROFILE_STATUS"
grep -F "$PROFILE_B" <<<"$PROFILE_STATUS"
"$YAI_BIN" case memory index verify case:memory-h19-acceptance --profile "$PROFILE_B"
"$YAI_BIN" case memory search case:memory-h19-acceptance \
  --participant participant:deepseek --query 'ORCHID-H19-731 4188 final' \
  --purpose inspection --resource resource:memory-h19-acceptance \
  --profile "$PROFILE_B" --limit 12 --json
"$YAI_BIN" case memory index drop case:memory-h19-acceptance --profile "$PROFILE_B" --json
"$YAI_BIN" case memory index verify case:memory-h19-acceptance --profile "$PROFILE_B" | grep -F 'posture: missing'
"$YAI_BIN" case memory index verify case:memory-h19-acceptance --profile "$PROFILE_A" | grep -F 'posture: current'

"$YAI_BIN" case create case:memory-h19-isolation-negative --tenant tenant:memory-h19-acceptance
"$YAI_BIN" case participant link-principal case:memory-h19-isolation-negative \
  --principal "$PRINCIPAL_ID" --participant participant:deepseek
"$YAI_BIN" case participant role add case:memory-h19-isolation-negative \
  --participant participant:deepseek --role model-executor
"$YAI_BIN" case participant view admit case:memory-h19-isolation-negative \
  --participant participant:deepseek --consumer model --view model_context
ISOLATED_SEARCH="$("$YAI_BIN" case memory search case:memory-h19-isolation-negative \
  --participant participant:deepseek \
  --query 'ORCHID-H19-731 4188 replacement applied' --limit 12 --json)"
python3 - "$ISOLATED_SEARCH" <<'PY'
import json
import sys

value = json.loads(sys.argv[1])["data"]["value"]
assert value["case_id"] == "case:memory-h19-isolation-negative"
assert value["selected_count"] == 0
print("cross_case_isolation: pass")
PY

"$YAI_BIN" case memory index drop case:memory-h19-acceptance --profile "$PROFILE_A" --json
"$YAI_BIN" case memory index verify case:memory-h19-acceptance --profile "$PROFILE_A" | grep -F 'posture: missing'
"$YAI_BIN" case show case:memory-h19-acceptance --json
"$YAI_BIN" case memory show case:memory-h19-acceptance --json

FALLBACK_RECALL="$("$YAI_BIN" case run case:memory-h19-acceptance \
  --participant participant:deepseek \
  --resource resource:memory-h19-acceptance \
  --prompt 'Senza assumere un indice disponibile, riassumi dal ContextFrame quale file è stato modificato e l esito finale. Rispondi in yai.case_runtime_turn.v1 con outcome complete e non proporre operazioni.' \
  --max-invocations 1 --max-operations 1 --max-runtime-ms 180000)"
printf '%s\n' "$FALLBACK_RECALL"
grep -F 'runtime_status: Completed' <<<"$FALLBACK_RECALL"
"$YAI_BIN" case memory index verify case:memory-h19-acceptance --profile "$PROFILE_A" | grep -F 'posture: missing'

REBUILT_A="$("$YAI_BIN" case memory index rebuild case:memory-h19-acceptance \
  --encoder-target "$ENCODER_TARGET" \
  --encoder-revision "$YAI_MEMORY_ENCODER_REVISION" \
  --dimension "$YAI_MEMORY_ENCODER_DIMENSION")"
printf '%s\n' "$REBUILT_A"
REBUILT_PROFILE_A="$(sed -n 's/^representation_profile_id: //p' <<<"$REBUILT_A")"
test "$REBUILT_PROFILE_A" = "$PROFILE_A"
"$YAI_BIN" case memory index verify case:memory-h19-acceptance --profile "$PROFILE_A"
"$YAI_BIN" case memory search case:memory-h19-acceptance \
  --participant participant:deepseek \
  --query 'ORCHID-H19-731 4188 replacement applied denied attempt' \
  --purpose inspection --resource resource:memory-h19-acceptance \
  --profile "$PROFILE_A" --limit 12 --json

FINAL_RECALL="$("$YAI_BIN" case run case:memory-h19-acceptance \
  --participant participant:deepseek \
  --resource resource:memory-h19-acceptance \
  --prompt 'Quale file abbiamo modificato in questo Case, quale tentativo è fallito e qual è stato l esito operativo finale? Rispondi nel campo summary di yai.case_runtime_turn.v1 con outcome complete, usando solo il ContextFrame qualificato.' \
  --max-invocations 1 --max-operations 1 --max-runtime-ms 180000)"
printf '%s\n' "$FINAL_RECALL"
grep -F 'runtime_status: Completed' <<<"$FINAL_RECALL"
PROJECTION_ID="$(sed -n 's/^projection_id: //p' <<<"$FINAL_RECALL")"
CONTEXT_FRAME_ID="$(sed -n 's/^context_frame_id: //p' <<<"$FINAL_RECALL")"
PROVIDER_RESULT_ID="$(sed -n 's/^last_provider_result_id: //p' <<<"$FINAL_RECALL")"
test -n "$PROJECTION_ID"
test -n "$CONTEXT_FRAME_ID"
test -n "$PROVIDER_RESULT_ID"

RETRIEVAL_INSPECTION="$("$YAI_BIN" case memory retrieval show case:memory-h19-acceptance --profile "$PROFILE_A" --json)"
PROVIDER_SELECTION="$("$YAI_BIN" case provider show case:memory-h19-acceptance)"
PROJECTION_INSPECTION="$("$YAI_BIN" context inspect --id "$PROJECTION_ID")"
FRAME_INSPECTION="$("$YAI_BIN" context inspect --id "$CONTEXT_FRAME_ID")"
printf '%s\n' "$RETRIEVAL_INSPECTION"
printf '%s\n' "$PROVIDER_SELECTION"
printf '%s\n' "$PROJECTION_INSPECTION"
printf '%s\n' "$FRAME_INSPECTION"
grep -F 'yai.retrieval_set.v2' <<<"$RETRIEVAL_INSPECTION"
grep -F "last_selected_target: $COGNITION_TARGET" <<<"$PROVIDER_SELECTION"
grep -F "last_selected_model: $YAI_EXTERNAL_PROVIDER_MODEL" <<<"$PROVIDER_SELECTION"
if grep -E '(vector_digest|embedding_id|lexical_checksum|vectors\.f32le|metadata\.json|derived-memory|hnsw)' <<<"$FRAME_INSPECTION"; then
  echo 'ContextFrame leaked derived index internals' >&2
  exit 1
fi
"$YAI_BIN" case show case:memory-h19-acceptance --json
"$YAI_BIN" case memory show case:memory-h19-acceptance --json
"$YAI_BIN" case memory index status case:memory-h19-acceptance --json
"$YAI_BIN" case memory index verify case:memory-h19-acceptance --profile "$PROFILE_A"

trap - EXIT
cleanup_h19
printf 'H19 manual acceptance completed; disposable state removed\n'
```

The live result is accepted for memory grounding only when the selected memory
IDs and provenance in RetrievalSet v2 correspond to the Projection and
ContextFrame used for the final YVEX/DeepSeek call. Eloquence is not graded.
Provider credentials, raw vectors, BM25 postings, component paths, and encoder
secrets must remain absent from the model input and inspection output.
