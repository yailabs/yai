# GOVERNANCE.COGNITIVE.CONTEXT.0

Authority: bounded implementation and qualification evidence. ROADMAP owns live
project control. Baseline `7c5687effc4ffcd48cde93d83bab3667316e3e1b`, reconciled
equal to origin/master and remote master in the existing physical master tree.
Intended commit: `feat: project effective policy into cognitive Case context`.
Pre-publication: **IN_PROGRESS; implementation/local qualification PASS,
Real Live FAIL at the first model response deadline**. The isolated commit/push
publishes earned implementation/evidence, not completion of the live milestone.
No completed boundary or human acceptance is claimed.

Unrelated pre-existing changes to `docs/provider-governance.md`,
`tests/characterization/case-resource-access/test_reference_free.py` and
`refoundation/validation/external-golden-closure/` are preserved and excluded
from this boundary's ownership. README, REPLAI, YVEX and their pins are untouched.

## Archaeology and ownership

Read the current roadmap, executable architecture, semantic-state target,
governance reference, H8, Wave 9/10, Semantic State and Golden evidence and the
actual governance/materializer/admission/compiler/provider/product seams.
The source is authoritative where older architecture prose still said input v3:
the baseline executable already accepted input/source v4 and artifact v5.

Read-only `yai-dev` recovery inspection:

| Historical source | Strongest useful property | Reuse / rejection |
|---|---|---|
| `45c36bd0c:tools/gen/deterministic_governance_ingestion.py` | Explicit structured rules through document representations, source coordinates and unresolved facts | Recover the representation/interpretation split and location discipline; reject silently skipped prose, mutable registry files and historical owner trees |
| `45c36bd0c:tests/integration/governance/test_ingestion_pipeline.sh` | Parse → normalize → candidate → validation/review posture | Current immutable source/catalog lifecycle is stronger; retain fail-closed candidate semantics, not a parallel engine |
| Later `69e4aa6a3` epoch policy source/effective-contract anchors | Ownership comments without an executable replacement mechanism | No recovered executable capability claimed from comment-only anchors |

The public YVEX roadmap was inspected read-only for program ownership. Its
default-branch V010 DeepSeek runtime optimization remains producer work; an older
public roadmap snapshot is not deployment truth. No producer code, engine,
profile, load workflow or private protocol was inspected/administered. No YVEX
consumer registration existed under `.boundary/consumers/`; none was invented.

## Implemented contract

Before: strict JSON source compiler; W carries policy binding references but
not the effective rules that admission enforces.

After:

```text
JSON / explicit Markdown block / bounded text-PDF policy sheet
  → deterministic extraction + original bytes/digest/coordinates
  → existing strict JSON facts / IR / unresolved candidate
  → explicit validation / publication / Case binding
  → the existing EffectivePolicy materializer
       ├── deterministic DecisionBasis / review / Grant / effect admission
       └── mandatory scoped EffectiveAuthority in S → W → context compatibility
```

`governance/document.rs` is a representation adapter inside the existing owner,
not another policy engine. Original source bytes, digest, declared origin and
version remain in the existing immutable source catalog. Markdown locations are
block line ranges plus exact JSON field coordinates; PDF locations are page and
extracted-line ranges plus JSON coordinates, not fabricated visual glyph spans.
PDF uses pinned lopdf 0.44.0 (MIT), strict parsing and bounded text extraction.
Only the documented text policy-sheet profile is supported. No OCR, natural-
language interpreter, model interpretation, training or new store is introduced.

All nonblank document text outside the explicit block remains unresolved.
Pure prose cannot be published or bound; a structured block plus prose produces
a blocked candidate. Ambiguity is resolved by an explicit reviewed structured
source, never by silently accepting guessed semantics. Unsupported/malformed,
encrypted, blank-text, annotation/drawing/form/image PDF profiles refuse.

`policy extract FILE` and `/policy extract FILE` inspect without authority.
Guided `/policy publish` previews extracted candidate/validation, requires
explicit consent and checks original digest again before publishing/binding.
The original-byte catalog recompilation defect exposed by the actual Markdown
product test was repaired: recompile original bytes, not extracted JSON.

The cognitive authority view carries exact EffectivePolicy identity/digest,
readiness, validity, rules for visible resource kinds, defaults and blockers.
Current Participant/roles and resource envelopes remain separately typed W
entries. Security-critical rules are mandatory; a budget that cannot fit them
refuses instead of dropping them. The view contains normalized semantics, not
raw source instructions, and never grants execution authority.

Catalog validity/revocation participates in S identity. Clock observation alone
does not; crossing a validity boundary does. W is recompiled and tied to the
exact Participant/output contract/Projection inside the invocation commit's
write transaction. Revocation can invalidate W without a Case generation change.
Historical W inspection against changed current catalog truth may refuse; it is
not a newly implemented as-of governance query.

Recent own `DecisionEvidence` is optional, scoped before selection and bounded
to six decisions by locality. It supplies DecisionBasis/rules, proposer,
resource, sequence/recording time, reviewer when present, Grant/receipt references.
It preserves historical outcomes after revoke without preserving permission.
This is not H, S12 or M06 Recall, and recording time is not invented event time.
The ordinary memory description now uses the actual Operation kind instead of
calling every decision `filesystem.write`.

## Schema and owner deltas

| Contract | Delta |
|---|---|
| Transition / CaseState | **v18 / v15 unchanged** |
| Policy input / ParsedPolicy / IR / artifact / binding / materializer / DecisionBasis | v4 / v2 / v2 / v5 / v2 / v3 / v3 unchanged |
| PolicySourceArtifact | Existing JSON v4 retained; document v5 adds original representation/provenance; old source readers retained |
| SemanticState / WorkingState / State Compiler | v1 → v2, identity-bearing normative input and new semantic value variants |
| SemanticDelta | v1 unchanged; policy-only same-generation changes require full compilation, not a fabricated forward delta |
| Projection / ContextFrame | v9 → v10; existing deterministic peers accept the new version without weakening semantic assertions |
| RenderedInput | v7 unchanged |
| Semantic / operational durable owners | **0 / 0** |
| LMDB | **delta 0; 37 named databases of configured 40** |

No E, State Read/Update, persistent deliberation, Agent, Studio, policy memory
database or generic model mutation authority exists as a result of this work.

## Retained executable evidence

All commands below ran from `/home/mothx/computer-science/projects/YAI/yai`.
Rust builds use `TMPDIR=/tmp CARGO_TARGET_DIR=target`; the ordinary temporary
cache location was not writable. Shell evidence directories are fresh test
homes, not operator canaries. Each product log records its own run ID/pre-state;
do not concatenate identifiers from different runs into one lifecycle.

| Run / command | Actual result and evidence |
|---|---|
| `cargo test --offline --manifest-path engine/Cargo.toml` (engine-regression-01, before final new history negative) | 326 passed, 4 ignored, exit 0; [raw](engine-regression-01.log). Supplemental, not the final publication claim |
| `cargo test --offline --manifest-path engine/Cargo.toml cognitive_decision_history -- --nocapture` | exit 0; [history-contract-01](history-contract-01.log): two exact decisions, other Participant zero, DENY no Grant/effect, revoke preserves history not permission |
| `cargo test --offline --manifest-path engine/Cargo.toml cognitive_authority_same -- --nocapture` | exit 0; [authority-contract-02](authority-contract-02.log): same materializer, mandatory budget refusal, scope refusal, cache rebuild equality, atomic stale/mismatched invocation refusal |
| `bash tests/characterization/governance-cognitive-context/test_policy_intake.sh` | exit 0; [intake-product-05](intake-product-05.log). Three actual product format pipelines, explicit publication, READY, exact normalized EffectivePolicy equivalence, rebuild/replay, ambiguous/premature refusal |
| `make test-golden-local` | exit 0 on final source; [golden-local-02](golden-local-02.log). Free and Workflow complete with actual executable/LMDB/adapters/PTY, **loopback model**, no external claim |
| `make check characterization` | exit 0 on final source; [publication-04](publication-04.log). Unit/component/contract/product/recovery/endurance and characterization union; no-provider and loopback-fixture axes, not live-model qualification |
| `make check-docs check-layout check-doc-links test-roadmap` | exit 0; [final documentation guards](docs-final-04.log). Generated maturity counts unchanged: 32 established / 23 partial / 12 open / 4 later, total 71 |

Final publication-union and Golden-local verdicts above are actual exits, not
inferred from aggregate counts. The QA environment is the temporary venv
`/tmp/yai-governance-qa-venv.Jf2vcj`, with the unchanged declared
`tests/requirements-terminal.txt` pins. Early attempts are retained separately:
the peer's old v9-only allowlist, missing pyte prerequisite and malformed xref in
the new PDF fixture were not accepted as passes. The parser was not loosened to
accept that fixture; its exact offsets/standard xref were repaired.

The concurrent Golden-driver modification only adds an explicit forensic
retention option; no requests/assertions/retry semantics change. It was present
during qualification and remains excluded from this commit. This work does not
claim to have qualified or published that independently owned option.
Local evidence attributes preserve raw PTY/HTTP CRLF, process trailing spaces
and blank lines rather than rewriting observed output to satisfy source-style
checks. Whitespace exceptions apply only to these captured logs, not code/docs.

Bounded unedited excerpts from independent runs:

```text
cognitive_authority exact_effective_policy=true relevant_rules=3 mandatory_budget_refusal=true absent_participant_refusal=true rebuild_equal=true revoke_without_case_generation_invalidates_W=true admission_refuses=true
```

```text
cognitive_history exact_basis=true exact_recorded_time=true own_decisions=2 other_participant_decisions=0 deny_grants=0 deny_effects=0 revocation_preserves_history_not_permission=true replay=true
```

```text
product_policy_intake=PASS formats=JSON,Markdown,PDF publish_before_validation=REFUSED ambiguous=REFUSED model_invocations=0 retained_home=/tmp/yai-policy-native.eYDxwE/home
```

The PDF format Case in that last run is `case:policy-pdf`, generation 2,
binding `case-policy-binding:074ec46c92cb96e9a847082dcf0718e7`, effective policy
`effective-policy:0d1afe9ed690a03b17ed2d7a04acb0f2`, READY. These identify a
format/publication proof, **not** a model-operated live Case.

## Real Live qualification — executed, no model result

Endpoint: `http://127.0.0.1:18001`; provider-exposed model:
`deepseek-v4-flash-mixed-mxfp4-release-v1`. No alternate model or fixture was
substituted. Early discovery exposed an empty catalog, then
[connection resets](provider-discovery-03.log). Those were not the final state:
[discovery 04](provider-discovery-04.log), at `2026-09-10T19:23:46Z`, exposed
compat.v3, the exact model, engine generation **1**, 16,384 input/sequence tokens
and the public preflight endpoint. Protocol number is not exposed in this HTTP
response; do not infer the currently deployed Git revision from the model name.

Public exact identities:

```text
artifact_identity=b669d80726cf83331c0d8016debbde44cf965a1503c33f605e92ea4e550ee87f
runtime_binding_identity=9810f314aa1d9b80b3cddb72128cec8e9307071cbe698eb1888bfe8bbe36e0c5
runtime_model_identity=e05d6574644e11235ced2f93c3fc1f455d8eee6e952bb53400505dd951ab04ce
specialization_identity=7604985ea75253a9338877e3867d2f18daac3be036ffa66babe84aca8ae2554e
capacity_plan_identity=fcba106cd99c97b9b41682a217c6006c7f9fa4ec526183bcfcad5d1542dba383
```

The actual product session is retained in [live-terminal.log](live-terminal.log),
captured by `script -q -f -c 'env YAI_HOME=/tmp/yai-governance-live.uOpHL7/home
./yai open policy-native' .../live-terminal.log`. This is native REPLAI/`./yai`,
not a model driver. [Bootstrap](live-bootstrap.log) records the fresh home and
tenant/Case creation. The Case contains one real filesystem with `src` admitted,
a protected sibling outside that envelope, and a six-rule Markdown policy:
read ALLOW, search DENY, write ALLOW + review + proposer-role + audit obligation.

```text
CASE: case:policy-native
TENANT: tenant:policy-native
HUMAN/REVIEWER: participant:operator (Principal-linked)
MODEL: participant:model (model-executor / operation-proposer; no human Principal link)
RESOURCE: resource:workspace
POLICY SOURCE: policy-source:40cd96d6426b7a53536ec9e7db5f0fa6f202db2c0c8ca825f90ab22e5b7d2bab
POLICY ARTIFACT: policy-artifact:ad4cf21a110701ce4d50f58ab89754af4a30bca9833ef66a2019633019411b6f
BINDING: case-policy-binding:dadad3befc89b97fdea8cc38bc375d11
EFFECTIVE POLICY: effective-policy:7ad56d480cd4975ecec910e347bd1715
READINESS: ready / valid
```

Operator actions in the retained session: admit guided Participants; `/attach`
the single resource; `/policy publish` the Markdown document after inspecting
the actual normalized rules; `/connect` with explicit synthetic-probe/trust/
operator-attested suitability approval. Text, functions and JSON were truthfully
qualified; those probes do not establish Case execution. Their observed response
times were 5,825 / 9,345 / 6,965 / 42,446 / 27,805 / 12,367 ms, in transcript
order, after the 12 ms catalog probe.

The sole actual Case question, not a hand-fed policy summary, was:

```text
What can you do in this Case right now? What is forbidden, and what needs review? Answer briefly.
```

Generation 15 W: `working-state:sha256:8d6ee2ba61d17d04c5895b7741d45e724bdda8e643ec13dbeffbfdb77700d5e6`;
Projection `projection:0d43f88c288b4a84`; frame `context-frame:657bba65690eeee3`;
render `rendered-input:b5b4571c580e4b16`.
[W](live-working-a.txt) has 10 selected entries, zero omitted, 2,723 semantic
units (**not tokens**), including the exact six current effective rules.
The separate [frame](live-frame-a.txt), [Projection](live-projection-a.txt) and
[render metadata](live-rendered-a.txt) retain their identities.

Observed product output after the unchanged 300-second response deadline:

```text
Execution unresolved: delivery is uncertain. Your message is saved. No automatic retry or target substitution was performed. /details for evidence.
```

The actual `/details` diagnostic, unedited:

```text
cognitive_realization_failed:delivery=DeliveryIndeterminate:derived_content=false:provider_delivery_indeterminate:response_deadline:bytes=14987
```

There is **no ProviderResult and no ordinary model answer to quote**. Time to
first received result is unavailable, not zero; no HTTP response status was
received. The consumer observation does not locate the wait in prefill versus
decode/runtime/transport. No hidden reasoning was requested or exposed.
The [canonical history](live-history-after-timeout.json) records the Turn and
Invocation at `2026-09-10T19:29:44Z`, and the indeterminate outcome at
`2026-09-10T19:34:45Z`: 301 seconds between second-resolution recording times,
including product overhead, for the 300-second transport deadline.
Live fallback = 0; inference retries = 0. The original invocation remains
`invocation:case:policy-native:model-prompt-2`; no blind retry was performed.

### Exact public capacity check and ownership of remaining pressure

After failure, an explicitly **forensic, non-generation** reconstruction used
the same engine renderer and verified equality with the product's persisted
render metadata before rebuilding its three-message OpenAI request. This is not
a replacement product workflow or synthetic tiny probe. The [exact JSON request](live-request-a-exact.json)
and [identity/size check](live-request-a-forensics-03.log) are retained. The
temporary diagnostic source lives at `/tmp/yai-governance-live.uOpHL7/forensic`;
`cargo run --offline --quiet --manifest-path .../Cargo.toml` only reads the Case
and emits that request; it never invokes a model.

| Exact dimension | Observation |
|---|---|
| Serialized HTTP body | **14,826 bytes**; digest `sha256:1c22fb78b5bd4f55d5d0ca25d018a72008fa371e44b924169a93861e21b37075` |
| Application bytes written including HTTP framing | **14,987** from actual transport failure |
| Rendered content | 13,617 characters: system 751 + frame 12,866; current Turn separately 97 bytes |
| Capability/function schema | **0 tools** in this ordinary conversation request |
| Tokenized input, public preflight | **4,448 tokens**, tokenizer/template-inclusive |
| Public rendered prompt | **13,782 bytes** |
| Output budget | omitted by product; public `requested_output_tokens=0` sentinel, effective allowance **11,936** |
| Runtime input / total sequence capacity | **16,384 / 16,384** |
| Preflight | HTTP **200**, `token_capacity_compatible=true`, no input/output capacity exceedance |
| Execution/resources qualification | **false**: a preflight PASS does not establish generation |

[Raw exact preflight](live-preflight-a-exact.log), run
`governance-live-preflight-a-exact` at `2026-09-10T19:45:24Z`, uses
`POST /v1/chat/completions/preflight` with that exact JSON. Its public request
identity is `d647344acbe4b90ed20644adc132429af3e058d07adcac1432b864aa7b68aadd`.
An earlier [diagnostic preflight](live-preflight-a.log) used the same JSON with
one extra trailing newline, so has a different request identity; it is not the
exact-byte evidence. Both are non-generation checks, not retries of
`chat/completions`. This consumer does not currently
perform this preflight automatically; the result above is explicit postmortem
public evidence, not a fabricated pre-dispatch check.

Classification: **YVEX_CANDIDATE** for admitted-request response latency/delivery;
not a demonstrated capacity defect and not proof of any producer-internal cause.
YAI's 14.8 KB context for one resource/six rules is also retained as compatibility
payload pressure, not dismissed: the effective-authority value is 4,299 serialized
characters and its binding 1,268; exact provenance is repeated in several current
compatibility fields. No mandatory state was truncated or timeout increased to
manufacture a pass. Producer phase/timing evidence is needed before another
execution attempt; do not diagnose or optimize YVEX internals from this session.

| Required live phase | Verdict |
|---|---|
| A: model enters the rules via Case → W | **FAIL / delivery indeterminate** after 300 s; W contains rules, but no model awareness answer was received |
| B: model-proposed allowed operation | NOT_RUN live; existing deterministic product path preserved |
| C: model-proposed forbidden operation and deterministic DENY | NOT_RUN live; no fixture response relabelled as model awareness |
| D: eligible review and admitted effect | NOT_RUN live; Golden-local review is independent evidence |
| E: real model explains who/what/where/when/how/why | NOT_RUN; cognitive quality cannot be inferred from canonical correctness |

After `/exit`, new product processes verified replay at generation 17 and rebuilt
the same current policy. [Restart evidence](live-restart.log) and
[recompiled W](live-working-a-restarted.txt) equal the original W byte-for-byte
(`cmp` exit 0), without inference. Canonical history survives the lost result;
that does not supply a missing cognitive answer or authorize a retry.

Codex must resume the remaining real phases after safe external reconciliation,
collect actual model outputs and compare against canonical evidence. It is not
Human Golden, not delegated manual acceptance, and not closed by the local PASS.

## Project control and INTERLOCK CHECK

The user explicitly selected this new boundary independently of the previous
large External Golden performance blockage. The semantic-state implementation
and local qualification remain earned; its full External Golden remains FAIL
with previous evidence preserved. Q03 is not promoted. Q04 stays
`HUMAN_GOLDEN_CASE = PENDING_OPERATOR`; Q05 canary stays NOT_RUN.
The cumulative `docs/zero-to-current.md` is updated only for extraction preview
and supported document inspection. Golden commands/deck are retained. README
does not become a report. A01 and C02 gain bounded intake/authority evidence,
without a maturity-state change or new row. No generic or cognitive-quality
promotion is claimed before the required Real Live qualification.

**INTERLOCK CHECK:** current governance catalog/materializer, semantic compiler,
admission and product owners can express this bounded integration without a new
independent owner or unresolved cross-owner authority split. No new Interlock is
selected; **I07 UNSELECTED**. E/Recall/producer optimization remains out of scope.
The milestone is not closed while the Real Live row is blocked.
