# MANUAL ACCEPTANCE — ZERO TO USE CASE

This acceptance starts from an isolated disposable `YAI_HOME`, uses the exact
operator-supplied YVEX endpoint and DeepSeek model as the cognitive provider,
and uses a separate qualified loopback embedding endpoint/model. It never
inspects or administers YVEX. Run from the YAI repository root with Bash,
`curl`, `python3`, a Rust toolchain, and both provider endpoints already
running. Do not point `YAI_HOME` at production data.

Required environment variables:

- `YAI_EXTERNAL_PROVIDER_BASE_URL`: exact running YVEX OpenAI-compatible base.
- `YAI_EXTERNAL_PROVIDER_MODEL`: exact DeepSeek model exposed by that endpoint.
- `YAI_MEMORY_ENCODER_BASE_URL`: exact running loopback OpenAI-compatible
  embedding base.
- `YAI_MEMORY_ENCODER_MODEL`: exact model exposed by that encoder.
- `YAI_MEMORY_ENCODER_REVISION`: operator-declared immutable model/profile
  revision.
- `YAI_MEMORY_ENCODER_DIMENSION`: exact positive embedding dimension.

Optional environment variables are `YAI_EXTERNAL_PROVIDER_API_KEY` and
`YAI_MEMORY_ENCODER_API_KEY`. If the YVEX URL is not loopback, also set
`YAI_EXTERNAL_PROVIDER_LOCALITY` to exactly `private_network` or `remote`.
The encoder is always required to be loopback in W19.

Expected postures are: both qualifications succeed; the negative turn is
denied and creates no file outside `allowed`; two later turns finalize writes;
the index reports `current` immediately after build; hybrid search exposes
exact/BM25/exact-cosine planes and a finalized observed consequence; the final
DeepSeek turn emits inspectable Projection/ContextFrame/ProviderResult IDs;
drop preserves the Case and OperationalMemory; rebuild restores the profile;
the isolated Case selects zero memories.

Copy and run the complete block in order:

```bash
set -euo pipefail

: "${YAI_EXTERNAL_PROVIDER_BASE_URL:?set this to the running YVEX OpenAI-compatible base URL}"
: "${YAI_EXTERNAL_PROVIDER_MODEL:?set this to the exact DeepSeek model exposed by YVEX}"
: "${YAI_MEMORY_ENCODER_BASE_URL:?set this to a running loopback OpenAI-compatible embedding base URL}"
: "${YAI_MEMORY_ENCODER_MODEL:?set this to the exact embedding model ID}"
: "${YAI_MEMORY_ENCODER_REVISION:?set this to the operator-declared encoder revision/profile}"
: "${YAI_MEMORY_ENCODER_DIMENSION:?set this to the exact embedding dimension}"

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
    : "${YAI_EXTERNAL_PROVIDER_LOCALITY:?set to private_network or remote for a non-loopback YVEX URL}"
    case "$YAI_EXTERNAL_PROVIDER_LOCALITY" in
      private_network|remote) ;;
      *) echo 'YAI_EXTERNAL_PROVIDER_LOCALITY must be private_network or remote' >&2; exit 2 ;;
    esac
    ;;
esac

python3 - "$YAI_MEMORY_ENCODER_BASE_URL" <<'PY'
import ipaddress
import sys
from urllib.parse import urlparse
host = urlparse(sys.argv[1]).hostname
if not host:
    raise SystemExit("YAI_MEMORY_ENCODER_BASE_URL has no host")
if host == "localhost":
    raise SystemExit(0)
try:
    if ipaddress.ip_address(host).is_loopback:
        raise SystemExit(0)
except ValueError:
    pass
raise SystemExit("W19 memory encoder must use a loopback host")
PY

make build-rust
YAI_BIN="$PWD/target/debug/yai"
test -x "$YAI_BIN"

YAI_W19_RUN_ROOT="$(mktemp -d /tmp/yai-w19-manual.XXXXXX)"
export YAI_HOME="$YAI_W19_RUN_ROOT/yai-home"
RESOURCE_ROOT="$YAI_W19_RUN_ROOT/resource"
POLICY_SOURCE="$YAI_W19_RUN_ROOT/memory-policy.json"
mkdir -p "$RESOURCE_ROOT/allowed"

cleanup_w19() {
  case "$YAI_W19_RUN_ROOT" in
    /tmp/yai-w19-manual.*) rm -rf "$YAI_W19_RUN_ROOT" ;;
    *) echo "refusing cleanup outside bounded W19 temp root: $YAI_W19_RUN_ROOT" >&2; return 1 ;;
  esac
}
trap cleanup_w19 EXIT

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

YVEX_MODELS="$YAI_W19_RUN_ROOT/yvex-models.json"
ENCODER_MODELS="$YAI_W19_RUN_ROOT/encoder-models.json"
if ! curl -fsS "${COGNITION_CURL_HEADERS[@]}" \
  "$(api_url "$YAI_EXTERNAL_PROVIDER_BASE_URL" models)" -o "$YVEX_MODELS"; then
  echo 'YVEX readiness failed: start the operator-managed endpoint and expose the configured DeepSeek model' >&2
  exit 3
fi
python3 - "$YVEX_MODELS" "$YAI_EXTERNAL_PROVIDER_MODEL" <<'PY'
import json, sys
data = json.load(open(sys.argv[1], encoding="utf-8"))
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
import json, sys
data = json.load(open(sys.argv[1], encoding="utf-8"))
ids = [item.get("id") for item in data.get("data", []) if isinstance(item, dict)]
if sys.argv[2] not in ids:
    raise SystemExit(f"configured embedding model is not exposed: {sys.argv[2]}")
print(f"embedding_model_ready: {sys.argv[2]}")
PY

"$YAI_BIN" init \
  --tenant tenant:memory-acceptance \
  --organization organization:memory-acceptance
"$YAI_BIN" doctor
"$YAI_BIN" identity whoami
PRINCIPAL_ID="$("$YAI_BIN" identity whoami | sed -n 's/^principal_id: //p' | head -1)"
test -n "$PRINCIPAL_ID"

COGNITION_ADD="$("$YAI_BIN" provider add \
  --tenant tenant:memory-acceptance \
  --provider-key yvex-deepseek-memory-acceptance \
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
  --tenant tenant:memory-acceptance \
  --provider-key loopback-memory-encoder \
  --endpoint "$YAI_MEMORY_ENCODER_BASE_URL" \
  --model "$YAI_MEMORY_ENCODER_MODEL" \
  --locality loopback \
  "${ENCODER_CREDENTIAL_ARGS[@]}")"
printf '%s\n' "$ENCODER_ADD"
ENCODER_TARGET="$(sed -n 's/^target_id: //p' <<<"$ENCODER_ADD")"
test -n "$ENCODER_TARGET"
ENCODER_QUALIFICATION="$("$YAI_BIN" provider qualify "$ENCODER_TARGET" --embedding)"
printf '%s\n' "$ENCODER_QUALIFICATION"
grep -F 'text_embedding: true' <<<"$ENCODER_QUALIFICATION"
grep -F "embedding_dimension: $YAI_MEMORY_ENCODER_DIMENSION" <<<"$ENCODER_QUALIFICATION"
"$YAI_BIN" provider trust approve "$ENCODER_TARGET"

"$YAI_BIN" case create case:memory-acceptance --tenant tenant:memory-acceptance
"$YAI_BIN" case participant link-principal case:memory-acceptance \
  --principal "$PRINCIPAL_ID" --participant participant:deepseek
"$YAI_BIN" case participant role add case:memory-acceptance \
  --participant participant:deepseek --role model-executor
"$YAI_BIN" case participant role add case:memory-acceptance \
  --participant participant:deepseek --role operation-proposer
"$YAI_BIN" case participant view admit case:memory-acceptance \
  --participant participant:deepseek --consumer model --view model_context
"$YAI_BIN" case participant list case:memory-acceptance
"$YAI_BIN" case provider bind case:memory-acceptance \
  --participant participant:deepseek --target "$COGNITION_TARGET" \
  --failover safe_only --max-attempts 1
"$YAI_BIN" case provider show case:memory-acceptance

"$YAI_BIN" case resource attach filesystem case:memory-acceptance \
  --resource resource:memory-acceptance \
  --root "$RESOURCE_ROOT" \
  --allow-prefix allowed \
  --policy-owner participant:deepseek \
  --max-bytes 4096

cat >"$POLICY_SOURCE" <<'POLICY'
{
  "schema": "yai.policy_source_input.v4",
  "policy_key": "manual.memory.acceptance",
  "source_version": "1",
  "owner_ref": "organization:memory-acceptance",
  "source_origin": {
    "source_system": "manual_acceptance",
    "source_uri": "manual://wave19/memory-acceptance"
  },
  "validity": {"mode": "unbounded"},
  "rules": [
    {
      "kind": "operation_restriction",
      "rule_id": "filesystem-posture",
      "operation_kind": "filesystem.write",
      "resource_kind": "filesystem",
      "effect": "allow",
      "reason": "Wave19 manual bounded filesystem write"
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

POLICY_INGEST="$("$YAI_BIN" policy ingest "$POLICY_SOURCE" --tenant tenant:memory-acceptance)"
printf '%s\n' "$POLICY_INGEST"
POLICY_ID="$(sed -n 's/^artifact_id: //p' <<<"$POLICY_INGEST" | head -1)"
test -n "$POLICY_ID"
"$YAI_BIN" policy validate "$POLICY_ID" --reason 'W19 manual validation'
"$YAI_BIN" policy publish "$POLICY_ID" --reason 'W19 manual publication'
"$YAI_BIN" case policy bind case:memory-acceptance \
  --artifact "$POLICY_ID" --reason 'W19 manual acceptance binding'
"$YAI_BIN" case policy show case:memory-acceptance

set +e
DENIED_OUTPUT="$("$YAI_BIN" case run case:memory-acceptance \
  --participant participant:deepseek \
  --resource resource:memory-acceptance \
  --prompt 'Proponi esclusivamente filesystem.write sulla risorsa resource:memory-acceptance al path denied/blocked.txt con contenuto DENIED-W19. Non cambiare path. Dopo la Decision, non inventare successo.' \
  --max-invocations 1 --max-operations 1 --stop-on-deny --max-runtime-ms 120000 2>&1)"
DENIED_EXIT=$?
set -e
printf '%s\n' "$DENIED_OUTPUT"
test "$DENIED_EXIT" -eq 0
grep -E 'runtime_status: (Denied|InvocationBudgetExhausted)' <<<"$DENIED_OUTPUT"
test ! -e "$RESOURCE_ROOT/denied/blocked.txt"

FIRST_WRITE="$("$YAI_BIN" case run case:memory-acceptance \
  --participant participant:deepseek \
  --resource resource:memory-acceptance \
  --prompt 'Proponi filesystem.write sulla risorsa resource:memory-acceptance al path allowed/orchid.txt. Il contenuto esatto deve essere: Project codename ORCHID-731. Numeric fact 4187. Dopo aver visto la conseguenza osservata, termina con yai.case_runtime_turn.v1 outcome complete.' \
  --max-invocations 3 --max-operations 2 --max-runtime-ms 180000)"
printf '%s\n' "$FIRST_WRITE"
grep -F 'runtime_status: Completed' <<<"$FIRST_WRITE"
grep -F 'effect_outcome: Applied' <<<"$FIRST_WRITE"
grep -F 'Project codename ORCHID-731. Numeric fact 4187.' "$RESOURCE_ROOT/allowed/orchid.txt"

REPLACEMENT="$("$YAI_BIN" case run case:memory-acceptance \
  --participant participant:deepseek \
  --resource resource:memory-acceptance \
  --prompt 'Proponi filesystem.write sulla risorsa resource:memory-acceptance allo stesso path allowed/orchid.txt. Sostituisci il contenuto con: Project codename ORCHID-731. Numeric fact 4188. Final outcome: replacement applied. Dopo la conseguenza osservata, termina con yai.case_runtime_turn.v1 outcome complete.' \
  --max-invocations 3 --max-operations 2 --max-runtime-ms 180000)"
printf '%s\n' "$REPLACEMENT"
grep -F 'runtime_status: Completed' <<<"$REPLACEMENT"
grep -F 'Project codename ORCHID-731. Numeric fact 4188. Final outcome: replacement applied.' "$RESOURCE_ROOT/allowed/orchid.txt"

"$YAI_BIN" case show case:memory-acceptance --json
"$YAI_BIN" case memory show case:memory-acceptance --json
INDEX_BUILD="$("$YAI_BIN" case memory index build case:memory-acceptance \
  --encoder-target "$ENCODER_TARGET" \
  --encoder-revision "$YAI_MEMORY_ENCODER_REVISION" \
  --dimension "$YAI_MEMORY_ENCODER_DIMENSION")"
printf '%s\n' "$INDEX_BUILD"
PROFILE_ID="$(sed -n 's/^representation_profile_id: //p' <<<"$INDEX_BUILD")"
INDEX_ID="$(sed -n 's/^index_manifest_id: //p' <<<"$INDEX_BUILD")"
test -n "$PROFILE_ID"
test -n "$INDEX_ID"
export YAI_MEMORY_PROFILE_ID="$PROFILE_ID"

"$YAI_BIN" case memory index status case:memory-acceptance --json
SEARCH_BEFORE="$("$YAI_BIN" case memory search case:memory-acceptance \
  --participant participant:deepseek \
  --query 'ORCHID-731 4188 replacement applied denied blocked attempt' \
  --purpose inspection \
  --resource resource:memory-acceptance \
  --profile "$PROFILE_ID" --limit 12 --json)"
printf '%s\n' "$SEARCH_BEFORE"
grep -F 'yai.retrieval_set.v2' <<<"$SEARCH_BEFORE"
grep -F 'lexical_bm25' <<<"$SEARCH_BEFORE"
grep -F 'vector_exact_cosine' <<<"$SEARCH_BEFORE"
grep -F 'finalized_observed_consequence' <<<"$SEARCH_BEFORE"

FINAL_RECALL="$("$YAI_BIN" case run case:memory-acceptance \
  --participant participant:deepseek \
  --resource resource:memory-acceptance \
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

"$YAI_BIN" case memory retrieval show case:memory-acceptance \
  --profile "$PROFILE_ID" --json
PROVIDER_SELECTION="$($YAI_BIN case provider show case:memory-acceptance)"
printf '%s\n' "$PROVIDER_SELECTION"
grep -F "last_selected_target: $COGNITION_TARGET" <<<"$PROVIDER_SELECTION"
grep -F "last_selected_model: $YAI_EXTERNAL_PROVIDER_MODEL" <<<"$PROVIDER_SELECTION"
PROJECTION_INSPECTION="$($YAI_BIN context inspect --id "$PROJECTION_ID")"
FRAME_INSPECTION="$($YAI_BIN context inspect --id "$CONTEXT_FRAME_ID")"
printf '%s\n' "$PROJECTION_INSPECTION"
printf '%s\n' "$FRAME_INSPECTION"
if grep -E '(vector_digest|embedding_id|lexical_checksum|bundle\.json|derived-memory|hnsw)' \
  <<<"$FRAME_INSPECTION"; then
  echo 'ContextFrame leaked derived index internals' >&2
  exit 1
fi
"$YAI_BIN" case show case:memory-acceptance --json
PROVIDER_RESULT_INSPECTION="$($YAI_BIN case memory search case:memory-acceptance \
  --participant participant:deepseek \
  --query "$PROVIDER_RESULT_ID ORCHID-731 4188 denied final outcome" \
  --purpose inspection --limit 12 --json)"
printf '%s\n' "$PROVIDER_RESULT_INSPECTION"
grep -F "$PROVIDER_RESULT_ID" <<<"$PROVIDER_RESULT_INSPECTION"

"$YAI_BIN" case create case:memory-isolation-negative --tenant tenant:memory-acceptance
"$YAI_BIN" case participant link-principal case:memory-isolation-negative \
  --principal "$PRINCIPAL_ID" --participant participant:deepseek
"$YAI_BIN" case participant role add case:memory-isolation-negative \
  --participant participant:deepseek --role model-executor
"$YAI_BIN" case participant view admit case:memory-isolation-negative \
  --participant participant:deepseek --consumer model --view model_context
ISOLATED_SEARCH="$("$YAI_BIN" case memory search case:memory-isolation-negative \
  --participant participant:deepseek \
  --query 'ORCHID-731 4188 replacement applied' --limit 12 --json)"
printf '%s\n' "$ISOLATED_SEARCH"
python3 - "$ISOLATED_SEARCH" <<'PY'
import json, sys
value = json.loads(sys.argv[1])["data"]["value"]
assert value["case_id"] == "case:memory-isolation-negative"
assert value["selected_count"] == 0
print("cross_case_isolation: pass")
PY

"$YAI_BIN" case memory index drop case:memory-acceptance \
  --profile "$PROFILE_ID" --json
"$YAI_BIN" case show case:memory-acceptance --json
"$YAI_BIN" case memory show case:memory-acceptance --json
REBUILT="$("$YAI_BIN" case memory index rebuild case:memory-acceptance \
  --encoder-target "$ENCODER_TARGET" \
  --encoder-revision "$YAI_MEMORY_ENCODER_REVISION" \
  --dimension "$YAI_MEMORY_ENCODER_DIMENSION")"
printf '%s\n' "$REBUILT"
REBUILT_PROFILE_ID="$(sed -n 's/^representation_profile_id: //p' <<<"$REBUILT")"
test "$REBUILT_PROFILE_ID" = "$PROFILE_ID"
"$YAI_BIN" case memory index status case:memory-acceptance --json
"$YAI_BIN" case memory search case:memory-acceptance \
  --participant participant:deepseek \
  --query 'ORCHID-731 4188 replacement applied' \
  --purpose inspection \
  --resource resource:memory-acceptance \
  --profile "$PROFILE_ID" --limit 12 --json

trap - EXIT
cleanup_w19
printf 'W19 manual acceptance completed; disposable state removed\n'
```

The final recall passes when the RetrievalSet and ContextFrame contain the
same-Resource observed consequence/Decision with exact provenance, the
ProviderSelection names the configured YVEX/DeepSeek target, and the final
ProviderResult summary is grounded in those items. Eloquence is not graded.
Raw vectors, BM25 postings, index paths, and credentials must be absent from
the ContextFrame and provider result inspection.
