# MANUAL ACCEPTANCE — ZERO TO USE CASE

Run these commands from the YAI repository root with Bash. Execute them in
order, one block at a time, and stop if an expected posture does not match.
They use ordinary shell syntax with no functions or hidden setup script.

Prerequisites:

- Rust/Cargo, `make`, `curl`, `python3`, `sed`, `grep`, and `mktemp`;
- an operator-managed YVEX endpoint exposing the exact selected DeepSeek model;
- a separate OpenAI-compatible embedding endpoint whose hostname resolves only
  to loopback;
- the six required variables below; credentials are optional and are never
  printed;
- for the first-party YVEX profile v2, the normal locality is `loopback`. Set
  `YAI_EXTERNAL_PROVIDER_LOCALITY=private_network` or `remote` only when the
  operator has deliberately exposed YVEX through such a boundary.

The disposable Case writes only below a fresh `/tmp/yai-w20-manual.*`
directory. The final command removes that directory.

```bash
test -n "${YAI_EXTERNAL_PROVIDER_BASE_URL:-}"
test -n "${YAI_EXTERNAL_PROVIDER_MODEL:-}"
test -n "${YAI_MEMORY_ENCODER_BASE_URL:-}"
test -n "${YAI_MEMORY_ENCODER_MODEL:-}"
test -n "${YAI_MEMORY_ENCODER_REVISION:-}"
test -n "${YAI_MEMORY_ENCODER_DIMENSION:-}"

python3 -c 'import sys; value=int(sys.argv[1]); assert 1 <= value <= 4096; print("embedding_dimension:", value)' "$YAI_MEMORY_ENCODER_DIMENSION"
python3 -c 'import ipaddress,socket,sys; from urllib.parse import urlparse; host=urlparse(sys.argv[1]).hostname; assert host; addresses={item[4][0] for item in socket.getaddrinfo(host,None)}; assert addresses and all(ipaddress.ip_address(value).is_loopback for value in addresses); print("encoder_locality: loopback")' "$YAI_MEMORY_ENCODER_BASE_URL"

export YAI_EXTERNAL_PROVIDER_LOCALITY="${YAI_EXTERNAL_PROVIDER_LOCALITY:-loopback}"
export NO_COLOR=1

YVEX_API_BASE="${YAI_EXTERNAL_PROVIDER_BASE_URL%/}"
if test "${YVEX_API_BASE##*/}" != "v1"; then YVEX_API_BASE="$YVEX_API_BASE/v1"; fi

ENCODER_API_BASE="${YAI_MEMORY_ENCODER_BASE_URL%/}"
if test "${ENCODER_API_BASE##*/}" != "v1"; then ENCODER_API_BASE="$ENCODER_API_BASE/v1"; fi

YAI_W20_RUN_ROOT="$(mktemp -d /tmp/yai-w20-manual.XXXXXX)"
export YAI_HOME="$YAI_W20_RUN_ROOT/yai-home"
RESOURCE_ROOT="$YAI_W20_RUN_ROOT/resource"
POLICY_SOURCE="$YAI_W20_RUN_ROOT/policy.json"
mkdir -p "$RESOURCE_ROOT/allowed"
cp tests/fixtures/cli-product-policy.json "$POLICY_SOURCE"

if test -n "${YAI_EXTERNAL_PROVIDER_API_KEY:-}"; then COGNITION_CREDENTIAL_REF=env:YAI_EXTERNAL_PROVIDER_API_KEY; else COGNITION_CREDENTIAL_REF=none; fi
if test -n "${YAI_MEMORY_ENCODER_API_KEY:-}"; then ENCODER_CREDENTIAL_REF=env:YAI_MEMORY_ENCODER_API_KEY; else ENCODER_CREDENTIAL_REF=none; fi

if test -n "${YAI_EXTERNAL_PROVIDER_API_KEY:-}"; then curl -fsS -H "Authorization: Bearer $YAI_EXTERNAL_PROVIDER_API_KEY" "$YVEX_API_BASE/models" -o "$YAI_W20_RUN_ROOT/yvex-models.json"; else curl -fsS "$YVEX_API_BASE/models" -o "$YAI_W20_RUN_ROOT/yvex-models.json"; fi
python3 -c 'import json,sys; ids=[item.get("id") for item in json.load(open(sys.argv[1],encoding="utf-8")).get("data",[])]; assert sys.argv[2] in ids, (sys.argv[2],ids); print("yvex_deepseek_ready:",sys.argv[2])' "$YAI_W20_RUN_ROOT/yvex-models.json" "$YAI_EXTERNAL_PROVIDER_MODEL"

if test -n "${YAI_MEMORY_ENCODER_API_KEY:-}"; then curl -fsS -H "Authorization: Bearer $YAI_MEMORY_ENCODER_API_KEY" "$ENCODER_API_BASE/models" -o "$YAI_W20_RUN_ROOT/encoder-models.json"; else curl -fsS "$ENCODER_API_BASE/models" -o "$YAI_W20_RUN_ROOT/encoder-models.json"; fi
python3 -c 'import json,sys; ids=[item.get("id") for item in json.load(open(sys.argv[1],encoding="utf-8")).get("data",[])]; assert sys.argv[2] in ids, (sys.argv[2],ids); print("embedding_model_ready:",sys.argv[2])' "$YAI_W20_RUN_ROOT/encoder-models.json" "$YAI_MEMORY_ENCODER_MODEL"

make build-rust
YAI_BIN="$PWD/target/debug/yai"
test -x "$YAI_BIN"

"$YAI_BIN" init --tenant tenant:memory-w20-acceptance --organization organization:cli-product
"$YAI_BIN" doctor
WHOAMI="$("$YAI_BIN" identity whoami)"
printf '%s\n' "$WHOAMI"
PRINCIPAL_ID="$(printf '%s\n' "$WHOAMI" | sed -n 's/^principal_id: //p' | head -1)"
test -n "$PRINCIPAL_ID"

COGNITION_ADD="$("$YAI_BIN" provider add --tenant tenant:memory-w20-acceptance --provider-key yvex-deepseek-w20 --endpoint "$YAI_EXTERNAL_PROVIDER_BASE_URL" --model "$YAI_EXTERNAL_PROVIDER_MODEL" --credential-ref "$COGNITION_CREDENTIAL_REF" --locality "$YAI_EXTERNAL_PROVIDER_LOCALITY")"
printf '%s\n' "$COGNITION_ADD"
COGNITION_TARGET="$(printf '%s\n' "$COGNITION_ADD" | sed -n 's/^target_id: //p')"
test -n "$COGNITION_TARGET"
"$YAI_BIN" provider qualify "$COGNITION_TARGET"
"$YAI_BIN" provider trust approve "$COGNITION_TARGET"

ENCODER_ADD="$("$YAI_BIN" provider add --tenant tenant:memory-w20-acceptance --provider-key loopback-memory-encoder-w20 --endpoint "$YAI_MEMORY_ENCODER_BASE_URL" --model "$YAI_MEMORY_ENCODER_MODEL" --credential-ref "$ENCODER_CREDENTIAL_REF" --locality loopback)"
printf '%s\n' "$ENCODER_ADD"
ENCODER_TARGET="$(printf '%s\n' "$ENCODER_ADD" | sed -n 's/^target_id: //p')"
test -n "$ENCODER_TARGET"
ENCODER_QUALIFICATION="$("$YAI_BIN" provider qualify "$ENCODER_TARGET" --embedding)"
printf '%s\n' "$ENCODER_QUALIFICATION"
printf '%s\n' "$ENCODER_QUALIFICATION" | grep 'TextEmbedding'
printf '%s\n' "$ENCODER_QUALIFICATION" | grep "embedding_dimension: $YAI_MEMORY_ENCODER_DIMENSION"
"$YAI_BIN" provider trust approve "$ENCODER_TARGET"

"$YAI_BIN" case create case:memory-w20-acceptance --tenant tenant:memory-w20-acceptance
"$YAI_BIN" case participant link-principal case:memory-w20-acceptance --principal "$PRINCIPAL_ID" --participant participant:deepseek
"$YAI_BIN" case participant role add case:memory-w20-acceptance --participant participant:deepseek --role model-executor
"$YAI_BIN" case participant role add case:memory-w20-acceptance --participant participant:deepseek --role operation-proposer
"$YAI_BIN" case participant view admit case:memory-w20-acceptance --participant participant:deepseek --consumer model --view model_context
"$YAI_BIN" case participant list case:memory-w20-acceptance
"$YAI_BIN" case provider bind case:memory-w20-acceptance --participant participant:deepseek --target "$COGNITION_TARGET" --failover safe_only --max-attempts 1
"$YAI_BIN" case provider show case:memory-w20-acceptance

"$YAI_BIN" case resource attach filesystem case:memory-w20-acceptance --resource resource:memory-w20-acceptance --root "$RESOURCE_ROOT" --allow-prefix allowed --policy-owner participant:deepseek --max-bytes 4096

POLICY_INGEST="$("$YAI_BIN" policy ingest "$POLICY_SOURCE" --tenant tenant:memory-w20-acceptance)"
printf '%s\n' "$POLICY_INGEST"
POLICY_ID="$(printf '%s\n' "$POLICY_INGEST" | sed -n 's/^artifact_id: //p' | head -1)"
test -n "$POLICY_ID"
"$YAI_BIN" policy validate "$POLICY_ID" --reason 'W20 manual validation'
"$YAI_BIN" policy publish "$POLICY_ID" --reason 'W20 manual publication'
"$YAI_BIN" case policy bind case:memory-w20-acceptance --artifact "$POLICY_ID" --reason 'W20 evidence-bound memory acceptance'
"$YAI_BIN" case policy show case:memory-w20-acceptance

DENIED_OUTPUT="$("$YAI_BIN" case run case:memory-w20-acceptance --participant participant:deepseek --resource resource:memory-w20-acceptance --prompt 'Proponi esclusivamente filesystem.write sul path denied/blocked-w20.txt con contenuto DENIED-W20. Non cambiare il path. Dopo la Decision non inventare successo.' --max-invocations 1 --max-operations 1 --stop-on-deny --max-runtime-ms 180000 2>&1)"
printf '%s\n' "$DENIED_OUTPUT"
printf '%s\n' "$DENIED_OUTPUT" | grep -E 'runtime_status: (Denied|InvocationBudgetExhausted)'
test ! -e "$RESOURCE_ROOT/denied/blocked-w20.txt"

FIRST_WRITE="$("$YAI_BIN" case run case:memory-w20-acceptance --participant participant:deepseek --resource resource:memory-w20-acceptance --prompt 'Proponi filesystem.write sulla risorsa resource:memory-w20-acceptance al path allowed/orchid-w20.txt. Il contenuto esatto deve essere: Project codename ORCHID-W20. Numeric fact 4187. Dopo la conseguenza osservata termina con yai.case_runtime_turn.v1 outcome complete.' --max-invocations 3 --max-operations 2 --max-runtime-ms 180000)"
printf '%s\n' "$FIRST_WRITE"
printf '%s\n' "$FIRST_WRITE" | grep 'runtime_status: Completed'
grep 'Project codename ORCHID-W20. Numeric fact 4187.' "$RESOURCE_ROOT/allowed/orchid-w20.txt"

REPLACEMENT="$("$YAI_BIN" case run case:memory-w20-acceptance --participant participant:deepseek --resource resource:memory-w20-acceptance --prompt 'Proponi filesystem.write sulla risorsa resource:memory-w20-acceptance allo stesso path allowed/orchid-w20.txt. Sostituisci il contenuto con: Project codename ORCHID-W20. Numeric fact 4188. Final outcome: replacement applied. Dopo la conseguenza osservata termina con yai.case_runtime_turn.v1 outcome complete.' --max-invocations 3 --max-operations 2 --max-runtime-ms 180000)"
printf '%s\n' "$REPLACEMENT"
printf '%s\n' "$REPLACEMENT" | grep 'runtime_status: Completed'
grep 'Project codename ORCHID-W20. Numeric fact 4188. Final outcome: replacement applied.' "$RESOURCE_ROOT/allowed/orchid-w20.txt"

PROVIDER_CLAIM="$("$YAI_BIN" case run case:memory-w20-acceptance --participant participant:deepseek --resource resource:memory-w20-acceptance --prompt 'Non proporre alcuna Operation e non modificare file. Produci yai.case_runtime_turn.v1 con outcome complete e una summary che afferma soltanto come claim del provider: ORCHID-W20 numeric fact 9999.' --max-invocations 1 --max-operations 1 --max-runtime-ms 180000)"
printf '%s\n' "$PROVIDER_CLAIM"
printf '%s\n' "$PROVIDER_CLAIM" | grep 'runtime_status: Completed'
grep 'Numeric fact 4188' "$RESOURCE_ROOT/allowed/orchid-w20.txt"

"$YAI_BIN" case show case:memory-w20-acceptance --json
"$YAI_BIN" case memory show case:memory-w20-acceptance --json

EPISODES_BEFORE="$("$YAI_BIN" case memory episodes show case:memory-w20-acceptance --participant participant:deepseek --json)"
printf '%s\n' "$EPISODES_BEFORE"
printf '%s\n' "$EPISODES_BEFORE" | grep 'yai.memory_episode.v1'
printf '%s\n' "$EPISODES_BEFORE" | grep 'denied'
printf '%s\n' "$EPISODES_BEFORE" | grep 'completed'
EPISODE_ID="$(python3 -c 'import json,sys; root=json.loads(sys.argv[1]); value=root.get("data",{}).get("value",root); print(value["episodes"][0]["episode_id"])' "$EPISODES_BEFORE")"
"$YAI_BIN" case memory episode show case:memory-w20-acceptance --participant participant:deepseek --episode "$EPISODE_ID" --json

SEMANTIC_BEFORE="$("$YAI_BIN" case memory semantic show case:memory-w20-acceptance --participant participant:deepseek --include-historical --json)"
printf '%s\n' "$SEMANTIC_BEFORE"
printf '%s\n' "$SEMANTIC_BEFORE" | grep 'provider_originated_claim'
printf '%s\n' "$SEMANTIC_BEFORE" | grep '9999'

CONSOLIDATION="$("$YAI_BIN" case memory consolidate case:memory-w20-acceptance --participant participant:deepseek --json)"
printf '%s\n' "$CONSOLIDATION"
printf '%s\n' "$CONSOLIDATION" | grep 'yai.memory_consolidation_result.v1'
printf '%s\n' "$CONSOLIDATION" | grep '"rebuild_requires_reinference":false'
CONSOLIDATION_INPUT_ID="$(python3 -c 'import json,sys; root=json.loads(sys.argv[1]); value=root.get("data",{}).get("value",root); print(value["consolidation_input_id"])' "$CONSOLIDATION")"
CONSOLIDATION_RESULT_ID="$(python3 -c 'import json,sys; root=json.loads(sys.argv[1]); value=root.get("data",{}).get("value",root); print(value["provider_result_id"])' "$CONSOLIDATION")"
CONSOLIDATION_PROJECTION_ID="$(python3 -c 'import json,sys; root=json.loads(sys.argv[1]); value=root.get("data",{}).get("value",root); print(value["projection_id"])' "$CONSOLIDATION")"
CONSOLIDATION_FRAME_ID="$(python3 -c 'import json,sys; root=json.loads(sys.argv[1]); value=root.get("data",{}).get("value",root); print(value["context_frame_id"])' "$CONSOLIDATION")"
printf 'consolidation_input_id: %s\nprovider_result_id: %s\n' "$CONSOLIDATION_INPUT_ID" "$CONSOLIDATION_RESULT_ID"
"$YAI_BIN" context inspect --id "$CONSOLIDATION_PROJECTION_ID"
"$YAI_BIN" context inspect --id "$CONSOLIDATION_FRAME_ID"

SEMANTIC_AFTER="$("$YAI_BIN" case memory semantic show case:memory-w20-acceptance --participant participant:deepseek --include-historical --json)"
printf '%s\n' "$SEMANTIC_AFTER"
printf '%s\n' "$SEMANTIC_AFTER" | grep 'evidence_bound_inference'
printf '%s\n' "$SEMANTIC_AFTER" | grep 'provider_originated_claim'
printf '%s\n' "$SEMANTIC_AFTER" | grep '4188'
printf '%s\n' "$SEMANTIC_AFTER" | grep '9999'

CONTRADICTIONS="$("$YAI_BIN" case memory contradictions case:memory-w20-acceptance --participant participant:deepseek --json)"
printf '%s\n' "$CONTRADICTIONS"
printf '%s\n' "$CONTRADICTIONS" | grep 'structural_value_conflict'
printf '%s\n' "$CONTRADICTIONS" | grep 'unresolved'

HIERARCHY_AFTER="$("$YAI_BIN" case memory hierarchy show case:memory-w20-acceptance --participant participant:deepseek --json)"
printf '%s\n' "$HIERARCHY_AFTER"
HIERARCHY_ID_BEFORE_DROP="$(python3 -c 'import json,sys; root=json.loads(sys.argv[1]); value=root.get("data",{}).get("value",root); print(value["hierarchy_id"])' "$HIERARCHY_AFTER")"
SEMANTIC_IDS_BEFORE_DROP="$(python3 -c 'import json,sys; root=json.loads(sys.argv[1]); value=root.get("data",{}).get("value",root); print(" ".join(sorted(item["assertion_id"] for item in value["assertions"])))' "$SEMANTIC_AFTER")"

INDEX_BUILD="$("$YAI_BIN" case memory index build case:memory-w20-acceptance --encoder-target "$ENCODER_TARGET" --encoder-revision "$YAI_MEMORY_ENCODER_REVISION" --dimension "$YAI_MEMORY_ENCODER_DIMENSION")"
printf '%s\n' "$INDEX_BUILD"
PROFILE_ID="$(printf '%s\n' "$INDEX_BUILD" | sed -n 's/^representation_profile_id: //p')"
INDEX_ID="$(printf '%s\n' "$INDEX_BUILD" | sed -n 's/^index_manifest_id: //p')"
test -n "$PROFILE_ID"
test -n "$INDEX_ID"
export YAI_MEMORY_PROFILE_ID="$PROFILE_ID"

"$YAI_BIN" case memory index status case:memory-w20-acceptance --json
"$YAI_BIN" case memory index verify case:memory-w20-acceptance --profile "$PROFILE_ID"

SEARCH_ONE="$("$YAI_BIN" case memory search case:memory-w20-acceptance --participant participant:deepseek --query 'ORCHID-W20 numeric fact 4188 9999 denied previous value final outcome' --purpose inspection --profile "$PROFILE_ID" --limit 16 --json)"
printf '%s\n' "$SEARCH_ONE"
printf '%s\n' "$SEARCH_ONE" | grep 'yai.retrieval_set.v3'
printf '%s\n' "$SEARCH_ONE" | grep '"memory_family":"episodic"'
printf '%s\n' "$SEARCH_ONE" | grep '"memory_family":"semantic"'
printf '%s\n' "$SEARCH_ONE" | grep 'evidence_bound_inference'

"$YAI_BIN" case create case:memory-w20-isolation-negative --tenant tenant:memory-w20-acceptance
"$YAI_BIN" case participant link-principal case:memory-w20-isolation-negative --principal "$PRINCIPAL_ID" --participant participant:deepseek
"$YAI_BIN" case participant role add case:memory-w20-isolation-negative --participant participant:deepseek --role model-executor
"$YAI_BIN" case participant view admit case:memory-w20-isolation-negative --participant participant:deepseek --consumer model --view model_context
ISOLATED_SEARCH="$("$YAI_BIN" case memory search case:memory-w20-isolation-negative --participant participant:deepseek --query 'ORCHID-W20 4188 9999' --limit 16 --json)"
printf '%s\n' "$ISOLATED_SEARCH"
python3 -c 'import json,sys; root=json.loads(sys.argv[1]); value=root.get("data",{}).get("value",root); assert value["case_id"]=="case:memory-w20-isolation-negative"; assert value["selected_count"]==0; print("cross_case_isolation: pass")' "$ISOLATED_SEARCH"

PROVIDER_RECORD_COUNT_BEFORE="$("$YAI_BIN" store record list --case case:memory-w20-acceptance --limit 200 | grep -c 'record_id: model-output:')"
printf 'provider_result_record_count_before_rebuild: %s\n' "$PROVIDER_RECORD_COUNT_BEFORE"

"$YAI_BIN" case memory index drop case:memory-w20-acceptance --profile "$PROFILE_ID" --json
"$YAI_BIN" case memory index verify case:memory-w20-acceptance --profile "$PROFILE_ID"
"$YAI_BIN" case show case:memory-w20-acceptance --json
"$YAI_BIN" case memory show case:memory-w20-acceptance --json

FALLBACK_SEARCH="$("$YAI_BIN" case memory search case:memory-w20-acceptance --participant participant:deepseek --query 'ORCHID-W20 4188 final outcome' --profile "$PROFILE_ID" --limit 12 --json)"
printf '%s\n' "$FALLBACK_SEARCH"
printf '%s\n' "$FALLBACK_SEARCH" | grep '"plane":"lexical_bm25","available":false'
printf '%s\n' "$FALLBACK_SEARCH" | grep '"plane":"vector_exact_cosine","available":false'

"$YAI_BIN" case memory hierarchy drop case:memory-w20-acceptance --participant participant:deepseek --json
HIERARCHY_REBUILT="$("$YAI_BIN" case memory hierarchy rebuild case:memory-w20-acceptance --participant participant:deepseek --json)"
printf '%s\n' "$HIERARCHY_REBUILT"
HIERARCHY_ID_AFTER_REBUILD="$(python3 -c 'import json,sys; root=json.loads(sys.argv[1]); value=root.get("data",{}).get("value",root); print(value["hierarchy_id"])' "$HIERARCHY_REBUILT")"
test "$HIERARCHY_ID_BEFORE_DROP" = "$HIERARCHY_ID_AFTER_REBUILD"

SEMANTIC_REBUILT="$("$YAI_BIN" case memory semantic show case:memory-w20-acceptance --participant participant:deepseek --include-historical --json)"
SEMANTIC_IDS_AFTER_REBUILD="$(python3 -c 'import json,sys; root=json.loads(sys.argv[1]); value=root.get("data",{}).get("value",root); print(" ".join(sorted(item["assertion_id"] for item in value["assertions"])))' "$SEMANTIC_REBUILT")"
test "$SEMANTIC_IDS_BEFORE_DROP" = "$SEMANTIC_IDS_AFTER_REBUILD"
printf '%s\n' "$SEMANTIC_REBUILT" | grep "$CONSOLIDATION_RESULT_ID"

PROVIDER_RECORD_COUNT_AFTER="$("$YAI_BIN" store record list --case case:memory-w20-acceptance --limit 200 | grep -c 'record_id: model-output:')"
printf 'provider_result_record_count_after_rebuild: %s\n' "$PROVIDER_RECORD_COUNT_AFTER"
test "$PROVIDER_RECORD_COUNT_BEFORE" = "$PROVIDER_RECORD_COUNT_AFTER"

INDEX_REBUILD="$("$YAI_BIN" case memory index rebuild case:memory-w20-acceptance --encoder-target "$ENCODER_TARGET" --encoder-revision "$YAI_MEMORY_ENCODER_REVISION" --dimension "$YAI_MEMORY_ENCODER_DIMENSION")"
printf '%s\n' "$INDEX_REBUILD"
REBUILT_PROFILE_ID="$(printf '%s\n' "$INDEX_REBUILD" | sed -n 's/^representation_profile_id: //p')"
test "$PROFILE_ID" = "$REBUILT_PROFILE_ID"
"$YAI_BIN" case memory index verify case:memory-w20-acceptance --profile "$PROFILE_ID"
"$YAI_BIN" case memory search case:memory-w20-acceptance --participant participant:deepseek --query 'ORCHID-W20 4188 9999 denied final outcome' --purpose inspection --profile "$PROFILE_ID" --limit 16 --json

FINAL_RECALL="$("$YAI_BIN" case run case:memory-w20-acceptance --participant participant:deepseek --resource resource:memory-w20-acceptance --prompt 'Qual è il valore operativo finale, quale valore precedente è stato sostituito, quale claim contraddittorio è apparso e su quali evidenze si basa questa distinzione? Non proporre operazioni. Rispondi con yai.case_runtime_turn.v1 outcome complete usando solo il ContextFrame qualificato.' --max-invocations 1 --max-operations 1 --max-runtime-ms 180000)"
printf '%s\n' "$FINAL_RECALL"
printf '%s\n' "$FINAL_RECALL" | grep 'runtime_status: Completed'
FINAL_PROJECTION_ID="$(printf '%s\n' "$FINAL_RECALL" | sed -n 's/^projection_id: //p')"
FINAL_CONTEXT_FRAME_ID="$(printf '%s\n' "$FINAL_RECALL" | sed -n 's/^context_frame_id: //p')"
FINAL_PROVIDER_RESULT_ID="$(printf '%s\n' "$FINAL_RECALL" | sed -n 's/^last_provider_result_id: //p')"
test -n "$FINAL_PROJECTION_ID"
test -n "$FINAL_CONTEXT_FRAME_ID"
test -n "$FINAL_PROVIDER_RESULT_ID"

"$YAI_BIN" case memory retrieval show case:memory-w20-acceptance --profile "$PROFILE_ID" --json
"$YAI_BIN" case memory episodes show case:memory-w20-acceptance --participant participant:deepseek --json
"$YAI_BIN" case memory semantic show case:memory-w20-acceptance --participant participant:deepseek --include-historical --json
"$YAI_BIN" case memory contradictions case:memory-w20-acceptance --participant participant:deepseek --json
"$YAI_BIN" case provider show case:memory-w20-acceptance
"$YAI_BIN" context inspect --id "$FINAL_PROJECTION_ID"
"$YAI_BIN" context inspect --id "$FINAL_CONTEXT_FRAME_ID"
"$YAI_BIN" case memory index status case:memory-w20-acceptance --json
"$YAI_BIN" case memory index verify case:memory-w20-acceptance --profile "$PROFILE_ID"
grep 'Numeric fact 4188' "$RESOURCE_ROOT/allowed/orchid-w20.txt"

rm -rf "$YAI_W20_RUN_ROOT"
printf 'W20 manual acceptance completed; disposable state removed\n'
```

Expected important results are: the YVEX and encoder model IDs are listed by
their endpoints; qualification records ChatText/StructuredJsonObject and
TextEmbedding respectively; the denied file does not exist; the Resource moves
from 4187 to 4188 while 9999 exists only as ProviderOriginatedClaim material;
Episodes include denied and completed postures; consolidation returns a
content-addressed input ID and recorded ProviderResult; contradictions preserve
epistemic classes; RetrievalSet v3 contains typed episodic and semantic
families; the negative Case selects zero items; dropped indexes degrade to
qualified non-index retrieval; hierarchy/assertion IDs rebuild exactly with no
new provider record; and the final Projection/ContextFrame show v6 typed memory
without vectors, BM25 postings, paths, or credentials.
