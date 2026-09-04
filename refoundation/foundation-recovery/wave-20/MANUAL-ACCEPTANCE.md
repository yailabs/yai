# MANUAL ACCEPTANCE — ZERO TO USE CASE

This is an operator walkthrough, not a Bash test harness. Run the commands in
order from the YAI repository root. The setup and cleanup blocks contain the
only shell housekeeping. Every YAI action and inspection between them uses a
registry-backed `yai` command. No generated ID is parsed or copied into
a later command.

## Prerequisites

- Rust/Cargo and `make`;
- an operator-managed YVEX endpoint exposing the exact selected DeepSeek model;
- a separate OpenAI-compatible embedding endpoint on loopback;
- these variables already exported with the operator's real values:
  `YAI_EXTERNAL_PROVIDER_BASE_URL`, `YAI_EXTERNAL_PROVIDER_MODEL`,
  `YAI_MEMORY_ENCODER_BASE_URL`, `YAI_MEMORY_ENCODER_MODEL`,
  `YAI_MEMORY_ENCODER_REVISION`, and `YAI_MEMORY_ENCODER_DIMENSION`;
- optionally, `YAI_EXTERNAL_PROVIDER_API_KEY` and
  `YAI_MEMORY_ENCODER_API_KEY`. YAI uses each only when it is non-empty and
  persists only the environment-variable reference;
- the YVEX and encoder URLs must already include the operator's intended API
  base. Both endpoints in this acceptance are declared `loopback` and must
  resolve accordingly.

The CLI rejects a missing endpoint/model, a non-loopback endpoint, an encoder
dimension outside `1..=4096`, a model mismatch, or an invalid provider
envelope. `yai provider qualify` is the readiness check; no separate `curl` or
JSON parser is required.

## 1. Clean isolated setup

These are the only preparatory shell commands. The path is deliberately fixed,
disposable, and outside operator production state.

```bash
rm -rf /tmp/yai-w20-natural-acceptance
mkdir -p /tmp/yai-w20-natural-acceptance/resource/allowed
export YAI_HOME=/tmp/yai-w20-natural-acceptance/yai-home
export NO_COLOR=1
make build-rust
export PATH="$PWD/target/debug:$PATH"
```

## 2. Initialize YAI and the Tenant

```bash
yai init --tenant tenant:memory-w20-acceptance --organization organization:cli-product
yai doctor
yai identity whoami
```

Important posture: `doctor` reports the isolated home as ready and `whoami`
shows an authenticated local Principal. No Principal ID needs to be copied;
`--principal self` resolves it inside YAI.

## 3. Register, qualify, and trust YVEX + DeepSeek

```bash
yai provider add --tenant tenant:memory-w20-acceptance --provider-key yvex-deepseek-w20 --endpoint "$YAI_EXTERNAL_PROVIDER_BASE_URL" --model "$YAI_EXTERNAL_PROVIDER_MODEL" --credential-env YAI_EXTERNAL_PROVIDER_API_KEY --locality loopback
yai provider qualify --tenant tenant:memory-w20-acceptance --provider-key yvex-deepseek-w20
yai provider trust approve --tenant tenant:memory-w20-acceptance --provider-key yvex-deepseek-w20
yai provider show --tenant tenant:memory-w20-acceptance --provider-key yvex-deepseek-w20
```

Important posture: qualification records `ChatText` and
`StructuredJsonObject` for the exact configured model; trust is `Approved`.
The target remains addressable by its stable Tenant-scoped provider key.

## 4. Register, qualify, and trust the loopback encoder

```bash
yai provider add --tenant tenant:memory-w20-acceptance --provider-key loopback-memory-encoder-w20 --endpoint "$YAI_MEMORY_ENCODER_BASE_URL" --model "$YAI_MEMORY_ENCODER_MODEL" --credential-env YAI_MEMORY_ENCODER_API_KEY --locality loopback
yai provider qualify --tenant tenant:memory-w20-acceptance --provider-key loopback-memory-encoder-w20 --embedding
yai provider trust approve --tenant tenant:memory-w20-acceptance --provider-key loopback-memory-encoder-w20
yai provider show --tenant tenant:memory-w20-acceptance --provider-key loopback-memory-encoder-w20
```

Important posture: qualification reports `TextEmbedding`, exact model
addressing, and the same dimension later supplied to the index build. This is
separate from YVEX; YVEX is not treated as an embedding service.

## 5. Create the Case, Participant, provider binding, and Resource

```bash
yai case create case:memory-w20-acceptance --tenant tenant:memory-w20-acceptance
yai case participant role add case:memory-w20-acceptance --participant participant:deepseek --role model-executor
yai case participant link-principal case:memory-w20-acceptance --principal self --participant participant:deepseek
yai case participant role add case:memory-w20-acceptance --participant participant:deepseek --role operation-proposer
yai case participant view admit case:memory-w20-acceptance --participant participant:deepseek --consumer model --view model_context
yai case participant list case:memory-w20-acceptance
yai case provider bind case:memory-w20-acceptance --participant participant:deepseek --provider-key yvex-deepseek-w20 --failover safe_only --max-attempts 1
yai case provider show case:memory-w20-acceptance
yai case resource attach filesystem case:memory-w20-acceptance --resource resource:memory-w20-acceptance --root /tmp/yai-w20-natural-acceptance/resource --allow-prefix allowed --policy-owner participant:deepseek --max-bytes 4096
yai case resource list case:memory-w20-acceptance
```

Important posture: the Case is Tenant-scoped, the current authenticated
Principal is linked to `participant:deepseek`, and the provider binding shows
the exact DeepSeek model selected through the governed target.

## 6. Ingest, validate, publish, and bind Policy

```bash
yai policy ingest tests/fixtures/cli-product-policy.json --tenant tenant:memory-w20-acceptance --validate --publish --reason "W20 natural manual acceptance"
yai case policy bind case:memory-w20-acceptance --policy-key cli.product.governed --reason "W20 evidence-bound memory acceptance"
yai case policy show case:memory-w20-acceptance
```

Important posture: ingest reports the immutable artifact, then `validated` and
`published`; Case binding resolves the current published artifact by its exact
policy key. No artifact hash is copied through shell text processing.

## 7. Produce heterogeneous Case history

The first command deliberately asks for a path outside the admitted prefix.

```bash
yai case run case:memory-w20-acceptance --participant participant:deepseek --resource resource:memory-w20-acceptance --prompt "Proponi esclusivamente filesystem.write sul path denied/blocked-w20.txt con contenuto DENIED-W20. Non cambiare il path. Dopo la Decision non inventare successo." --max-invocations 1 --max-operations 1 --stop-on-deny --max-runtime-ms 180000
yai case show case:memory-w20-acceptance
```

Important negative posture: the run reports `Denied` (or a bounded invocation
stop before any Effect), `operations: 0`, and no successful Effect.

Create the first observed value:

```bash
yai case run case:memory-w20-acceptance --participant participant:deepseek --resource resource:memory-w20-acceptance --prompt "Proponi filesystem.write sulla risorsa resource:memory-w20-acceptance al path allowed/orchid-w20.txt. Il contenuto esatto deve essere: Project codename ORCHID-W20. Numeric fact 4187. Dopo la conseguenza osservata termina con yai.case_runtime_turn.v1 outcome complete." --max-invocations 3 --max-operations 2 --max-runtime-ms 180000
yai case show case:memory-w20-acceptance
```

Important posture: `runtime_status: Completed` and the latest Effect outcome is
successful.

Replace it with the current observed value:

```bash
yai case run case:memory-w20-acceptance --participant participant:deepseek --resource resource:memory-w20-acceptance --prompt "Proponi filesystem.write sulla risorsa resource:memory-w20-acceptance allo stesso path allowed/orchid-w20.txt. Sostituisci il contenuto con: Project codename ORCHID-W20. Numeric fact 4188. Final outcome: replacement applied. Dopo la conseguenza osservata termina con yai.case_runtime_turn.v1 outcome complete." --max-invocations 3 --max-operations 2 --max-runtime-ms 180000
yai case show case:memory-w20-acceptance
```

Important posture: the second write is completed and its finalized Effect
supersedes the earlier resource state.

Create the contradictory provider-only claim without changing the Resource:

```bash
yai case run case:memory-w20-acceptance --participant participant:deepseek --resource resource:memory-w20-acceptance --prompt "Non proporre alcuna Operation e non modificare file. Produci yai.case_runtime_turn.v1 con outcome complete e una summary che afferma soltanto come claim del provider: ORCHID-W20 numeric fact 9999." --max-invocations 1 --max-operations 1 --max-runtime-ms 180000
yai case show case:memory-w20-acceptance
yai case memory show case:memory-w20-acceptance
```

Important posture: the turn is complete with no new successful write; `9999`
is provider-originated material, not an observed Effect.

## 8. Derive and inspect Episodes and pre-consolidation semantics

```bash
yai case memory episodes show case:memory-w20-acceptance --participant participant:deepseek
yai case memory episode show case:memory-w20-acceptance --participant participant:deepseek --episode latest
yai case memory semantic show case:memory-w20-acceptance --participant participant:deepseek --include-historical
yai case memory hierarchy show case:memory-w20-acceptance --participant participant:deepseek
```

Important posture: Episode output includes denied and completed structural
postures and exact generation ranges. Semantic output keeps the `9999` item as
`ProviderOriginatedClaim`.

## 9. Run explicit DeepSeek consolidation

```bash
yai case memory consolidate case:memory-w20-acceptance --participant participant:deepseek
yai case memory semantic show case:memory-w20-acceptance --participant participant:deepseek --include-historical
yai case memory contradictions case:memory-w20-acceptance --participant participant:deepseek
yai case memory hierarchy show case:memory-w20-acceptance --participant participant:deepseek
yai store record list --case case:memory-w20-acceptance --limit 200
```

Important posture: consolidation prints its input, ProviderSelection,
ProviderInvocation, ProviderResult, Projection, ContextFrame, assertion, and
hierarchy IDs. It reports `rebuild_requires_reinference: no`. Assertions derived
from validated support are `EvidenceBoundInference`; the provider claim remains
`ProviderOriginatedClaim`; structural conflict is explicit and unresolved where
no mechanical rule chooses a winner. The record listing is the before-rebuild
canonical invocation inventory.

## 10. Build, verify, and search the multi-family index

```bash
yai case memory index build case:memory-w20-acceptance --encoder-provider-key loopback-memory-encoder-w20 --encoder-revision "$YAI_MEMORY_ENCODER_REVISION" --dimension "$YAI_MEMORY_ENCODER_DIMENSION"
yai case memory index status case:memory-w20-acceptance
yai case memory index verify case:memory-w20-acceptance
yai case memory search case:memory-w20-acceptance --participant participant:deepseek --query "ORCHID-W20 numeric fact 4188 9999 denied previous value final outcome" --purpose inspection --limit 16
yai case memory retrieval show case:memory-w20-acceptance
```

Important posture: status and verify show one `current` sealed v2 physical
index. Search reports RetrievalSet v3 and distinct operational, episodic, and
semantic families with their epistemic/lifecycle posture. With one current
profile, `verify`, `search`, and `retrieval show` resolve it without a profile
hash.

## 11. Cross-Case negative path

```bash
yai case create case:memory-w20-isolation-negative --tenant tenant:memory-w20-acceptance
yai case participant role add case:memory-w20-isolation-negative --participant participant:deepseek --role model-executor
yai case participant link-principal case:memory-w20-isolation-negative --principal self --participant participant:deepseek
yai case participant view admit case:memory-w20-isolation-negative --participant participant:deepseek --consumer model --view model_context
yai case memory search case:memory-w20-isolation-negative --participant participant:deepseek --query "ORCHID-W20 4188 9999" --limit 16
```

Required result: `selected: 0`. No count or content from the first Case appears.

## 12. Drop derived state and prove semantic continuity

```bash
yai case memory index drop case:memory-w20-acceptance
yai case memory index status case:memory-w20-acceptance
yai case show case:memory-w20-acceptance
yai case memory show case:memory-w20-acceptance
yai case memory search case:memory-w20-acceptance --participant participant:deepseek --query "ORCHID-W20 4188 final outcome" --limit 12
yai case memory hierarchy drop case:memory-w20-acceptance --participant participant:deepseek
yai case memory hierarchy rebuild case:memory-w20-acceptance --participant participant:deepseek
yai case memory episodes show case:memory-w20-acceptance --participant participant:deepseek
yai case memory semantic show case:memory-w20-acceptance --participant participant:deepseek --include-historical
yai case memory contradictions case:memory-w20-acceptance --participant participant:deepseek
yai store record list --case case:memory-w20-acceptance --limit 200
```

Required result: index status is absent and fuzzy planes are unavailable, while
qualified non-index memory remains. Hierarchy rebuild reports
`rebuild_requires_provider_invocation: no`; Episode, assertion, contradiction,
and hierarchy identities match the earlier inspection. The second record
listing contains no additional consolidation invocation or ProviderResult.

## 13. Rebuild and repeat retrieval

```bash
yai case memory index rebuild case:memory-w20-acceptance --encoder-provider-key loopback-memory-encoder-w20 --encoder-revision "$YAI_MEMORY_ENCODER_REVISION" --dimension "$YAI_MEMORY_ENCODER_DIMENSION"
yai case memory index verify case:memory-w20-acceptance
yai case memory search case:memory-w20-acceptance --participant participant:deepseek --query "ORCHID-W20 4188 9999 denied final outcome" --purpose inspection --limit 16
yai case memory retrieval show case:memory-w20-acceptance
```

Required result: the same representation profile is current again, with a
sealed manifest derived from current Case memory.

## 14. Final real DeepSeek recall and context inspection

```bash
yai case run case:memory-w20-acceptance --participant participant:deepseek --resource resource:memory-w20-acceptance --prompt "Qual è il valore operativo finale, quale valore precedente è stato sostituito, quale claim contraddittorio è apparso e su quali evidenze si basa questa distinzione? Non proporre operazioni. Rispondi con yai.case_runtime_turn.v1 outcome complete usando solo il ContextFrame qualificato." --max-invocations 1 --max-operations 1 --max-runtime-ms 180000
yai case show case:memory-w20-acceptance
yai case memory retrieval show case:memory-w20-acceptance
yai case context show case:memory-w20-acceptance --kind projection
yai case context show case:memory-w20-acceptance --kind context-frame
yai case memory episodes show case:memory-w20-acceptance --participant participant:deepseek
yai case memory semantic show case:memory-w20-acceptance --participant participant:deepseek --include-historical
yai case memory contradictions case:memory-w20-acceptance --participant participant:deepseek
yai case provider show case:memory-w20-acceptance
yai case memory index status case:memory-w20-acceptance
yai case memory index verify case:memory-w20-acceptance
```

Required result: the runtime completes; the latest Projection and ContextFrame
are v6, current for the Case generation used by that invocation, and reference
the exact RetrievalSet. The answer distinguishes final observed `4188`, prior
observed `4187`, and provider-only `9999` using qualified Case memory. Raw
vectors, BM25 postings, internal paths, and credentials are absent.

## 15. Cleanup

Only after inspection is complete:

```bash
rm -rf /tmp/yai-w20-natural-acceptance
```

This removes only the explicitly disposable acceptance home and Resource.
