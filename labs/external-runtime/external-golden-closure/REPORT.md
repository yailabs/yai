# External Golden — retained public capacity and execution characterization

## Current publication classification

Reconciled for publication from YAI `9f3a56d0e9cda18d07a176d6674148a9c9893b24`.
This preserves pre-existing, non-reproducible external observations and their
raw records; it is not a new implementation wave or an external PASS.
**MEASURED_LIMITATION / external qualification incomplete. External Golden is
not a default implementation gate.** No live execution is required to publish
this characterization or to start unrelated Case semantic/security work.
The latest retained real run reached a 300-second terminal watchdog without a
ProviderResult; external Workflow was not reached. Capacity compatibility and
local PASS remain different claims. Human Golden stays PENDING_OPERATOR and
canary NOT_RUN. No W→E contract or I07 selection follows.

The sections below are historical observations and contemporaneous analysis of
the earlier `EXTERNAL.GOLDEN.CLOSURE.0` investigation, at their recorded SHAs.
Their BLOCKED/mandatory-lane/manual-handoff/next-wave language describes the
old investigation, not current project selection or a current publication gate.
That gating interpretation is superseded by [current validation doctrine](../../../docs/test-cases.md#external-yvex).
Commands, outputs, timings, identities and raw capture files remain unchanged;
historical test PASS records are not represented as fresh validation results.

## Historical investigation summary

Repair of **SEMANTIC.STATE.REFOUNDATION.0**, not another selected boundary.
Baseline/reconciled HEAD, origin/master and remote master:
`844a708aecda4a41ede552a2f7b329ac4c0338ed`.
Intended eventual completion message: `fix: qualify the semantic-state external Golden envelope`.
**Historical external qualification: BLOCKED — external Golden FAIL at the 300-second test wait.
No completion commit.** Latest generation 2 offers **16,384 tokens** and admits
the original Golden request (12,146 input / 4,238 output allowance). A fresh
actual product run then times out without receiving response bytes or recording
a ProviderResult; Workflow is not reached. Its fresh body is also capacity
compatible (12,055 / 4,329, measured postmortem). See the final generation-2
section and [new exact evidence](v3-generation2-external.jsonl). The subsequent
operator-supplied producer excerpt below locates the wait in prefill (918/12,055
tokens, zero generated), followed by cancellation and session cleanup; it does
not identify the internal bottleneck. Earlier v2,
endpoint-mismatch and 512-token failures remain historical. No fallback or retry.
Runtime construction was not changed during this investigation. Local proof
does not close the mandatory failed external lane. I07 remains UNSELECTED;
W→E, State Read/Update, YVEX implementation and REPLAI are outside this repair.

## Original finding and owning boundary (v2, before producer repair)

The real Golden request is rejected by the public YVEX adapter with:

```json
{"error":{"message":"token output capacity exceeded","type":"invalid_request_error","param":null,"code":"request_too_large"}}
```

HTTP is `413 Content Too Large`. **This is not evidence of a 40 KB HTTP-body
limit.** A separate synthetic request of exactly the same body size passes when
most bytes are legal JSON whitespace. A dense synthetic request still fails
with the same error when `max_tokens=1`. Therefore omission of a small explicit
output budget is not a sufficient explanation or demonstrated fix either.

The blocking owner is the **public YVEX executable-input/token envelope**:
it rejects this request but the inspected public catalog/health/error responses
provide neither the applicable capacity nor a pre-dispatch accounting contract.
The exact private implementation cause and numeric token threshold are UNKNOWN.
In particular, the phrase “token output capacity” must not be silently
reinterpreted as requested completion tokens, tokenizer-buffer internals, or
a published model context window. No private implementation was inspected.

Classification: **GENERIC_PROVIDER_CONTRACT_GAP**, with a **YVEX_CANDIDATE**
for the owning producer's investigation. YAI also has independently measured
compatibility amplification and no qualified target capacity propagation.
These are YAI-owned follow-up obligations once a truthful capacity contract is
available, not evidence that arbitrary truncation will close the Golden.

## Original public identity and BOUNDARY

- Endpoint: `http://127.0.0.1:18001`; operator-supplied public SSH-forwarded
  endpoint, not a deterministic model fixture.
- Exact exposed model: `deepseek-v4-flash-mixed-mxfp4-release-v1`.
- Header/model/health profile: `yvex.openai.compat.v2`.
- `/health`: `{"status":"ok","adapter":"ready","server":"ready","profile":"yvex.openai.compat.v2"}`.
- Public responses do not expose a deployment Git revision or capacity values.
  No revision, engine identity, tokenizer or model-family limit is guessed.
- `.boundary/consumers/` and `.boundary/` are absent in the reconciled YAI tree.
  YVEX is not registered here: resolver/receipt **NOT_APPLICABLE**. No producer
  registration or repin was created. This package is the cross-repository handoff.

See [public-health.jsonl](public-health.jsonl) and the first two catalog
exchanges in [wire.json](wire/wire.json). These observations do not establish
that no other public document exists; the producer must identify any normative
capacity contract it intends YAI to consume.

## Exact failing product request

Run `external-golden-forensics-20260909`, order 1, command
`make test-golden-external-yvex` under the non-mutating network tracer below.
Fresh nested run `yai-golden-free-hhbplh2h`; no operator canary was used.

```sh
env TMPDIR=/tmp CARGO_NET_OFFLINE=true CARGO_TARGET_DIR=target PATH=/home/mothx/computer-science/projects/YAI/yai/build/terminal-tests/bin:$PATH YAI_EXTERNAL_PROVIDER_BASE_URL=http://127.0.0.1:18001 YAI_EXTERNAL_PROVIDER_MODEL=deepseek-v4-flash-mixed-mxfp4-release-v1 YAI_EXTERNAL_PROVIDER_LOCALITY=loopback YAI_GOLDEN_RETAIN_RUN=1 YAI_GOLDEN_EVIDENCE_PREFIX=/tmp/yai-external-closure.3IQ0uM/golden python3 refoundation/validation/provider-connect-hardening/capture.py --output /tmp/yai-external-closure.3IQ0uM/run.jsonl --run external-golden-forensics-20260909 --order 1 --prestate 'HEAD 844a708; fresh Golden home; unchanged runtime and provider payload; only opt-in test home retention added; real endpoint catalog reachable' --mode external_yvex -- strace -f -s 131072 -xx -yy -e trace=network -o /tmp/yai-external-closure.3IQ0uM/network.trace make test-golden-external-yvex
```

CWD for all retained commands is
`/home/mothx/computer-science/projects/YAI/yai`. Exit **2**, elapsed **142.372 s**;
connection/shape probes pass, first real `/work` fails, external Workflow is
**NOT_REACHED**. [external.jsonl](external.jsonl) is a byte-identical retained
copy of the capture record, including bounded unedited stdout/stderr, exact
environment, command, baseline, and the failed `/details` output.

The opt-in retention switch changes only test cleanup, not runtime requests.
The fresh home remains at `/tmp/yai-golden-free-hhbplh2h/home` for inspection;
it is not a continuity canary and is not included in Git. The tracer observed
the actual executable's public socket writes/reads, not a substituted adapter.
No fixture provider ran in this external test.

### Causal pipeline and exact sizes

All JSON sizes below are **UTF-8 compact serialized bytes**, not estimates of
tokens. The analyzer compares the stored frame with the transmitted frame,
the stored W/Projection/frame entries for exact equality, both HTTP
Content-Length values, and compact serialization with observed wire bytes.
Read-only `mdb_dump` supplies the derived artifacts from the retained test home.

| Boundary / component | Exact bytes / result |
|---|---:|
| W serialized artifact (includes identity, request, decisions) | 14,800 |
| Selected semantic `entries` array alone | 9,769 |
| W selected / visible / omitted | 15 / 16 / 1 |
| Compiler units (not model tokens) | 2,446 selected; 16,384 budget |
| Projection serialized artifact | 10,519 |
| ContextFrame serialized artifact | 32,190 |
| Frame `output_contract` field, name + colon + value | 20,524 |
| Frame `semantic_instructions` field | 916 |
| Frame `model_independent_constraints` field | 369 |
| Frame `task` field | 362 |
| System message content, unescaped UTF-8 | 427 |
| User frame prefix + frame, unescaped UTF-8 | 32,214 |
| Current canonical user Turn text, separate native user message | 280 |
| These three content strings, raw UTF-8 total | 32,921 |
| These three JSON string values, including quotes/escaping | 35,656 |
| Native `tools` field (10 functions), name + colon + value | 4,408 |
| Entire serialized HTTP JSON body | **40,277** |
| Actual application HTTP headers including separator | **161** |
| Total application HTTP request bytes written | **40,438** |

The first Golden request has no prior model claims or native feedback exchange.
The 280-byte current human Turn is also present inside its typed semantic entry;
that entry is 1,627 bytes including ordered-part identity and provenance. The
task field is application control referencing that Turn, not another copy of
the human request. All 15 semantic entries survive W→Projection→Frame unchanged.

The body decomposes exactly into these disjoint top-level fields (field name,
colon and serialized value included): `messages=35751`, `model=50`,
`parallel_tool_calls=27`, `stream=14`, `tool_choice=20`, `tools=4408`, plus
**7 bytes** of outer braces/commas. Total **40,277**. The `messages` field's
structural overhead beyond its three JSON content values is **95 bytes**.
Frame fields, entry-by-entry sizes, and their additional JSON-string escaping
cost are retained in [wire.json](wire/wire.json), not inferred from character counts.
Within native tools, compact parameter schema objects total **1,520 bytes**;
unescaped descriptions total **1,619 bytes**, and function names **520 bytes**.
The remaining **749 bytes** include JSON field names, wrappers, separators and
string quoting. These measurements come from the exact captured tool objects;
none is a token estimate.

Exact identities from this run:

```text
Case generation: 26 (26 historical Transitions considered)
S: semantic-state:sha256:30c758af505e7fc26451e1296767395ae81beec43ec5652701324b81650b93f1
W: working-state:sha256:9df950b590e3dc7ae0bff18f42b8366d3aff41aef89688cdb73a95cd0de9369a
Projection: projection:826cf25cea9e9e46
ContextFrame: context-frame:384af8296207c902
Turn: conversation-turn:sha256:7a99744759292be2716df24491aa3d0d4a78dc05eafb19707af207e9690979ef
Body SHA256: aed2b961b08f85d812f33cefcdf192dd56528ca7f44a23f4b5966e5ba6750306
```

The public request shape is POST `/v1/chat/completions`, `stream:false`, exact
model, system + serialized typed ContextFrame + canonical text Turn messages,
10 native functions, `parallel_tool_calls:false`, `tool_choice:"auto"`; no
`max_tokens` field in this actual product request. See exact
[body](wire/09-body.json), [HTTP header/response](wire/09-http.json),
[W](wire/09-working.json), [Projection](wire/09-projection.json), and
[ContextFrame](wire/09-frame.json). These are only synthetic Golden Case data.
Do not replay historical indeterminate work as a retry after a repair: rerun
the complete fresh-home gate instead.

### Amplification and scaling ownership

| Material | Scales with | Finding |
|---|---|---|
| W semantic entries | current visible Case/task, exact requirements, bounded derived selection | 1 irrelevant candidate omitted; no budget-driven omission; not raw full history |
| Immutable Turn parts | admitted current/conversation content and modality | exact 280-byte current text occurs in typed entry and native text message |
| Frame capability view | resource operations and policy richness | 12 entries for 6 resources; MCP tool/resource functions await canonical catalog, so only 10 offered functions |
| Resource descriptors in view | operations per resource | 12 occurrences, 7,382 bytes; 6 unique objects, 3,646 bytes |
| Policy contribution objects in view | contributing exact rules/provenance | 18 objects, 7,555 bytes; **all 18 distinct**, not duplicate rules that can simply be dropped |
| Native function schemas | offered capabilities, argument schema richness | 4,408-byte field; required public function-call interface, not Case authority |
| JSON wrapping/escaping | all rendered strings/schema/control content | measured separately above; no token equivalence implied |

Repeated resource descriptors account for 3,736 bytes of repeated **raw object
occurrences**. This is not a net lossless-compaction saving: a shared-descriptor
encoding would need references and a qualified renderer contract. The view and
the native `tools` field serve different consumers; the view carries policy,
scope and provenance absent from function signatures. Deleting the entire
20,524-byte contract is not an equivalent lowering. Mandatory Case/policy
material must not be silently truncated. Lossless sharing is legitimate
YAI-owned pressure, but has not been implemented or qualified in this blocked
handoff and would not discover an unknown tokenizer limit.

No W selection, Projection/Frame schema, provider request construction,
qualification evidence, target, trust, or delivery classification was changed.
**Before/after product body: unchanged 40,277 bytes** for the reproduced run;
no fictitious size reduction or post-repair Golden PASS is claimed.

## Bounded public controls — separate runs, not retries

The successful connection's six synthetic POST bodies were respectively
230, 288, 379, 724, 958 and 340 bytes. All returned 200; the native function
exchange was genuinely executed. This does not establish a capacity ceiling.
The smallest observed passing POST in this run is 230 bytes, not a global minimum.

Two additional independent controls contain no Case data, functions, continuation
or resource operations. Each is one request, no fallback, no retry, 30-second
timeout, and `max_tokens=1`. The retained records' original diagnostic label
`contract_diagnostic` denotes **contract / qualification / external_yvex** here,
not a new TEST.TOPOLOGY proof class or a Golden gate. Commands actually run:

```sh
python3 refoundation/validation/external-golden-closure/probe_envelope.py --endpoint http://127.0.0.1:18001 --model deepseek-v4-flash-mixed-mxfp4-release-v1 --reference-body /tmp/yai-external-closure.3IQ0uM/pipeline/09-body.json --output /tmp/yai-external-closure.3IQ0uM/byte-control
python3 refoundation/validation/external-golden-closure/probe_envelope.py --endpoint http://127.0.0.1:18001 --model deepseek-v4-flash-mixed-mxfp4-release-v1 --reference-body /tmp/yai-external-closure.3IQ0uM/pipeline/09-body.json --density text --output /tmp/yai-external-closure.3IQ0uM/text-control
```

| Run (execution order after Golden failure) | Body | Public outcome | Exit / elapsed |
|---|---|---|---|
| byte-control (1) | 164 JSON bytes + 40,113 legal trailing whitespace bytes = 40,277 | HTTP 200; usage 12 prompt / 1 completion tokens | 0 / 3.761 s |
| text-control (2) | 40,277 JSON bytes; synthetic ` x` repetitions inside content, no padding | HTTP 413, `token output capacity exceeded`, no token usage disclosed | 1 / 2.444 s |

Unedited results: [byte-control.json](byte-control.json),
[text-control.json](text-control.json). Both have exact body hashes and baseline.
No threshold binary search or long inference benchmark was run. The observed
passing and failing sizes are **not ordered capacity bounds**: content density
and rendering matter. No model tokenizer was guessed or run locally.

## Original producer handoff and YAI capacity follow-through

The YVEX owner can investigate with only the exact public body, response and
control results here; no YAI internals are necessary. Required public resolution:

1. Identify which public input/token capacity rejects the supplied body. Publish
   its meaning and exact deployment/model/profile binding; do not relabel it an
   HTTP-byte ceiling or an output-token ceiling without evidence.
2. Make the real workload executable within the intended public envelope, or
   expose an exact typed incompatibility identifying actual/maximum capacity.
3. Supply a public, non-inference way to qualify/admit the **complete rendered
   request**, including chat template/native tool schemas, or an equivalently
   truthful consumer contract. A model-name context-window guess is insufficient.
4. Distinguish HTTP body bytes, input/tokenization capacity, model context,
   output reservation, and tool/schema constraints. Identify version/drift and
   whether rejection is definitely pre-execution; YAI will not infer retry safety
   just from status 413 after writing application request bytes.

Current YAI ProviderQualification records mechanical shapes, not an exact
request envelope. `ProviderRequirement.minimum_context_units`, if demanded,
fails closed because no honest capacity producer exists. Current W budget and
the complete-wire `body.len().div_ceil(4)` work-loop check are application bounds,
**not token counts or qualified remote capacity**. Product execution does not
currently demand a published remote capacity; this is the demonstrated gap.

After the producer clarifies its public contract, the same repair must align
YAI qualification and pre-dispatch admission with that exact contract. Optional
reduction must go through compiler decisions; mandatory incompatibility must be
typed, not silently removed. No guessed size, new capacity schema, blanket
trust approval or provider-brand workaround has been introduced in anticipation.

## Qualification, recovery and retained safety

Local regression is recorded in [regression.jsonl](regression.jsonl), run
`external-golden-repair-regression-20260909`, order 1, exact inner command
`make check characterization test-golden-local`, with offline Cargo, isolated
terminal-test venv PATH and TMPDIR=/tmp. This union covers compiler unit/recovery,
young/old locality, disclosure and mandatory references, full/delta equivalence,
replay/restart and derived drop/rebuild; Golden free and Workflow exercise the
actual executable and resource protocols with a loopback model. Its result is
independent of the external failure: **exit 0, PASS, 336.188 s**. The separate
[compiler run](compiler.jsonl), `cargo test --manifest-path engine/Cargo.toml
semantic_state::tests:: -- --nocapture`, is **exit 0, 1.129 s**. Its unedited
output includes:

```text
locality case=case:young transitions=8 selected=6 semantic_units=1056 omitted=3 required_retained=true
semantic_delta full_equals_recompiled=true stale_delta_refused=true restart_reconstruction=true mode=FullRecompilation
policy current_binding_retained=true task_switch_preserved=true superseded_ref_refused=true removal_delta_equals_full=true old_working_state_stale=true
locality case=case:old transitions=1206 selected=6 semantic_units=1054 omitted=2399 required_retained=true
```

These are separate unedited lines from that one run (intervening test-runner
lines remain in the capture), not combined output from different executions.
All eight compiler tests, including disclosure, mandatory-budget, tamper and
provider-independent lowering negatives, pass. No new locality limit is claimed.
[working-inspect.jsonl](working-inspect.jsonl) separately records exact historical
W re-compilation through `./yai context inspect --id ...`, exit 0, after the
external failure; it neither dispatches nor makes the old W current.

Independent final `make test-golden-local`: **exit 0, 41.941 s**, captured in
[golden-local.jsonl](golden-local.jsonl). Exact unedited first/final records
from each nested run are retained separately, not spliced into one Case:

| Local run / final order | Case / provider mode | Result and lineage |
|---|---|---|
| yai-golden-free-bot8qamu / 164 | free / loopback_fixture | [PASS](local-free-checkpoints.jsonl); 43 selected, 76 omitted; final Invocation `invocation:case:golden:free:model-prompt-44`; historical W recompiled equal after rebuild |
| yai-golden-free-o_410umk / 148 | Workflow / loopback_fixture | [PASS](local-workflow-checkpoints.jsonl); 46 selected, 93 omitted; final Invocation `invocation:case:golden:workflow:model-prompt-53`; historical W recompiled equal after rebuild |

Final documentation/layout/roadmap/topology guards: **exit 0, 3.618 s**, exact
command/environment in [docs.jsonl](docs.jsonl). Maturity counts remain
32 ESTABLISHED / 23 PARTIAL / 9 OPEN / 4 LATER; no maturity promotion. Only
the selected boundary's closure state changes to BLOCKED.

The external run dispatched one real Golden model request after successful
connection probes. It did not dispatch Workflow after failure. Fallback count
**0**. Hidden/cross-target retry count **0**. Delivery remains conservatively
**DeliveryIndeterminate**, with no repeated Turn execution or inferred external
effect success. Independent synthetic controls are not retries of that Turn.

## Original scope, project control and operator handoff

- Runtime semantic delta **0**; Transition **v18**, CaseState **v15**,
  Projection/ContextFrame **v9**, rendered input **v7** unchanged.
- Semantic owners **+0**, operational owners **+0**, LMDB **+0**, total **37/40**.
- YAI-only additions: opt-in fresh test-home retention; read-only forensic
  analyzer; explicit synthetic diagnostic support; exact evidence and capacity
  documentation. No load-bearing deletion or owner redesign.
- ROADMAP marks this same boundary **BLOCKED**, retains all earned maturity,
  and records the public capacity pressure. I07 **UNSELECTED**; no W→E selection.
- ZERO-TO-CURRENT **unchanged**: product commands/lifecycle are unchanged. This
  blocker must be resolved before preparing the requested human acceptance handoff.
- CONTINUITY_CANARY **NOT_RUN**; operator state was not accessed/reset.
- **HUMAN_GOLDEN_CASE = PENDING_OPERATOR**. No human PASS is inferred from PTY,
  synthetic controls, or deterministic tests.

Francesco's next action is to give this package to the YVEX owner for its public
capacity repair/clarification. After that repair, supply the exact public
deployment/model/contract identity and resume **this same** closure. From YAI's
repository, the required fresh complete external rerun is:

```sh
export YAI_EXTERNAL_PROVIDER_BASE_URL=http://127.0.0.1:18001
export YAI_EXTERNAL_PROVIDER_MODEL=deepseek-v4-flash-mixed-mxfp4-release-v1
export YAI_EXTERNAL_PROVIDER_LOCALITY=loopback
make test-golden-external-yvex
```

Do not run a manual “acceptance” sequence or relabel a small probe as the fix.
Only after **both external free and Workflow lifecycle PASS** will the normal
[ZERO-TO-CURRENT](../../../docs/zero-to-current.md) operator procedure be handed
off for separate human execution. There is no new final completion SHA in this
blocked handoff.

### Reproducibility material

[analysis.jsonl](analysis.jsonl) records the exact analyzer command, read-only
pre-state, exit and unedited output. Trace SHA256:
`3835f817eef2362994826f9150cc0a055faff47991a1cc38f6316ef68b208da9`;
full Golden action capture SHA256:
`3db2dbbe09604e6ee714a26f53be86e16fd9927d62fcd070e59c686e6910ed69`.
Those larger fresh-run traces remain under `/tmp/yai-external-closure.3IQ0uM/`;
the exact failing body/response and measured derived artifacts are retained here.
No private model, operator canary or credential bytes are part of this package.

## Historical resume: compat.v3 producer handoff / endpoint mismatch

This resumes the same closure, without discarding the original v2 failure or
its controls. On resume HEAD, origin/master and remote master remain
`844a708aecda4a41ede552a2f7b329ac4c0338ed`; existing README and evidence changes
are preserved. No runtime consumer adaptation has begun from guessed v3 fields.

Operator-supplied producer revisions (not independently exposed by this endpoint):

```text
capacity/public-contract repair: 5904df68b0304836caeca21df166eb7ff8fd0fc5
published YVEX models2 HEAD:     524efe7f67227f47a2fc749ca740f90d2294b937
expected public profile:        yvex.openai.compat.v3
expected protocol:              21
```

The producer finding supplied by the operator corrects the old diagnostic:
the original failure exhausted tokenizer output space while encoding **input**,
not requested completion capacity. Its reported 512-token test deployment is
not assumed to be the actual capacity of the endpoint observed in this resume.
Neither that value nor the producer's synthetic 12,004-token example is a
measurement of YAI's actual Golden request.

Observed public discovery run `external-golden-compat-v3-discovery-20260909`:
[compat-v3-discovery.jsonl](compat-v3-discovery.jsonl) retains exact commands,
CWD, relevant environment, pre-state, ordered outputs and exit statuses.

| Order | Public request | Observed result / command exit |
|---|---|---|
| 1 | GET `/v1/models` | HTTP 200; exact expected model, `yvex.openai.compat.v2`; no capacities / exit 0 |
| 2 | GET `/health` | HTTP 200; `profile: yvex.openai.compat.v2`; no protocol/deployment revision / exit 0 |
| 3 | POST `/v1/chat/completions/preflight` with original 40,277-byte Golden body | HTTP **404**, header still `yvex.openai.compat.v2` / curl exit **22** |

Exact preflight error body:

```json
{"error":{"message":"endpoint is outside the YVEX OpenAI profile","type":"invalid_request_error","param":null,"code":"model_not_found"}}
```

The original W identity, body SHA256 and byte count are unchanged from the
pipeline above and appear in order 3's pre-state. No synthetic tiny request
substituted for the Golden body. Only the newly specified **preflight** route
was called; `/v1/chat/completions` generation was not called in this resume.

Consequently:

- **EXTERNAL_GOLDEN = BLOCKED**, before capacity admission: the supplied
  endpoint does not yet expose the reported repaired public contract.
- Actual Golden input tokens, available input/sequence/output capacities,
  effective output budget, minimum required capacity and
  `execution_or_resources_qualified` are **UNKNOWN / NOT_RETURNED**.
- Protocol 21 and deployed revisions are **NOT_VERIFIED**. Publicly observed
  model remains `deepseek-v4-flash-mixed-mxfp4-release-v1`, profile **v2**.
- Do not label this `BLOCKED_BY_DEPLOYMENT_CAPACITY`: no actual capacity
  preflight was possible. The new deployment-discovery mismatch does not
  relabel the original real Golden 413 or erase its FAIL.
- No dispatch fallback, model substitution or blind retry. No runtime,
  Transition/CaseState, Projection/ContextFrame, owner or database delta.
- Previous deterministic/compiler/Golden-local PASS evidence remains intact;
  runtime regressions are not rerun solely for these read-only discovery calls.
- Human Golden remains **PENDING_OPERATOR**; canary **NOT_RUN**;
  ZERO-TO-CURRENT unchanged. No manual acceptance handoff before external PASS.

The operator must provide/correct the public endpoint or tunnel destination so
the actual YAI-side address exposes compat.v3 capacities and preflight. From
the host running YAI, check:

```sh
curl --max-time 10 -sS -i http://127.0.0.1:18001/v1/models
curl --max-time 10 -sS -i http://127.0.0.1:18001/health
```

Expected checkpoint is **compat.v3** and publicly qualified deployment/capacity
truth, not merely HTTP 200. Then resume discovery and actual-body preflight on
this same closure before adapting the consumer or running the external lifecycle.

## Historical resume: aligned compat.v3 / insufficient deployment capacity

**EXTERNAL_GOLDEN = BLOCKED_BY_DEPLOYMENT_CAPACITY.** The public contract is
now usable for diagnostic admission. It does not make the 512-token deployment
large enough. No semantic architecture, request material, target binding or
output budget was changed to fit it. No completion commit or next wave.

Run `external-golden-v3-deployment-20260910` is retained in
[v3-deployment-preflight.jsonl](v3-deployment-preflight.jsonl), with exact
commands, cwd, pre-state, YAI revision, execution order, exit and unedited
stdout/stderr. This is a **new preflight of the original product request**,
not a new successful Golden lifecycle. Its causal source remains the original
`yai-golden-free-hhbplh2h` product run; observations from the two runs are not
represented as one uninterrupted execution.

| Order | Proof / provider mode | Action and actual result |
|---|---|---|
| 1 | contract / external_yvex | GET `/v1/models`: HTTP 200, compat.v3, capacity facts; exit 0 |
| 2 | contract / external_yvex | GET `/health`: HTTP 200, compat.v3; exit 0 |
| 3 | contract / external_yvex | Exact 40,277-byte product body POST to the advertised preflight route: HTTP 200, **compatibility false**; exit 0, 0.028 s; no generation |
| 4 | recovery / no_provider | Product `context inspect` reconstructs original W from canonical history; `recompiled_from_canonical_history: true`; exit 0 |
| 5 | contract / no_provider | Recompiled W equals original; entries equal through Projection and ContextFrame; exact serialized frame is present in the transmitted body; body size/hash and preflight response hash verified; exit 0 |

Order 5's `omitted: []` is a filter over per-selected-item decisions, not a
zero-omission claim: the explicit compiler bounds report **one omission by
locality** (`no_current_intent_match`), zero omissions by budget. Complete W,
selected entries and decision provenance remain in [09-working.json](wire/09-working.json).
W was reconstructed using YAI's existing product inspection path, not a second
compiler. No new body was hand-assembled or replaced with a small synthetic
probe. Byte-identical captured product input avoids changing the Case/generation
while testing the repaired target; it does not reauthorize the historical W
for a fresh generation attempt.

### Exact semantic and request identity

| Identity / quantity | Observed value |
|---|---|
| YAI HEAD / origin/master / remote master | `844a708aecda4a41ede552a2f7b329ac4c0338ed` |
| Case / generation | `case:golden:free` / 26, original first work request |
| S identity | `semantic-state:sha256:30c758af505e7fc26451e1296767395ae81beec43ec5652701324b81650b93f1` |
| W identity | `working-state:sha256:9df950b590e3dc7ae0bff18f42b8366d3aff41aef89688cdb73a95cd0de9369a` |
| Selected / visible / omitted | 15 / 16 / 1; selected semantic units 2,446, not model tokens |
| HTTP body bytes / SHA-256 | 40,277 / `aed2b961b08f85d812f33cefcdf192dd56528ca7f44a23f4b5966e5ba6750306` |
| Public preflight rendered prompt bytes | 38,256, a different representation from HTTP JSON |
| Public tokenizer identity | `68f23b5e24f8ee3aa208a424041c306581435a0f9eaaf7da7e81d5527af0e50e` |
| Public prompt identity | `1bcda7893cd86ecdf5c4f4132cbfd44fdb3f1f5fb6fed34de9532df0e2415471` |
| Public provider request identity | `6a82fe703dca0f6e0974b00527ac445de68b435a5ef2053c950b5d7f8de9d59e` |
| Preflight JSON response bytes / SHA-256 | 1,890 / `89533f2c970f94d09bca326a79f779b85ab63f16f666d7e071534a0fb442c7f7` |

### Exact public deployment identity and capacities

Endpoint remains `http://127.0.0.1:18001`; catalog, health and preflight headers
and bodies expose **yvex.openai.compat.v3**. Model is exactly
`deepseek-v4-flash-mixed-mxfp4-release-v1`. Catalog and preflight agree on:

```text
engine_generation:       1
artifact_identity:       b669d80726cf83331c0d8016debbde44cf965a1503c33f605e92ea4e550ee87f
runtime_binding_identity: 9810f314aa1d9b80b3cddb72128cec8e9307071cbe698eb1888bfe8bbe36e0c5
runtime_model_identity:  e05d6574644e11235ced2f93c3fc1f455d8eee6e952bb53400505dd951ab04ce
specialization_identity: 7604985ea75253a9338877e3867d2f18daac3be036ffa66babe84aca8ae2554e
capacity_plan_identity:  53bc09cbd59d124c0a303e1649693ace1088a9d9933e38f9b109729e84ba88c2
```

Protocol **21**, deployed revision `524efe7f67227f47a2fc749ca740f90d2294b937`
and CUDA target-only are **operator-supplied**, not fields returned by these
three public responses. No private repository, process, CLI or model loader
was inspected. Public identities are opaque execution evidence, not S/W state.

| Capacity / preflight field | Exact public value |
|---|---|
| Capacity schema | `yvex.execution.capacity.v1` |
| `input_tokens` — GOLDEN_REQUIRED_INPUT_TOKENS | **12,146** |
| `runtime_input_tokens` | **512** |
| `runtime_sequence_tokens` | **512** |
| `requested_output_tokens` / `effective_output_tokens` | **0 / 0** |
| `maximum_requested_output_tokens` | 512, not an observed request budget |
| `token_capacity_compatible` | **false** |
| `input_capacity_exceeded` / `output_capacity_exceeded` | **true / false** |
| `full_requested_output_fits` | false |
| `execution_or_resources_qualified` | **false** |
| `scope` | `complete_stateless_chat_request` |
| `input_accounting` | `exact_tokenizer_including_template_and_tools` |
| `output_policy` | `ceiling_clamped_to_remaining_sequence` |
| `http_body_bytes` / `provider_wire_bytes` capacity | 1,048,576 / 1,048,576 |
| `architectural_context_tokens` | null; no model-architecture maximum inferred |
| `resource_reservation` | false; compatible preflight would not reserve execution |

The actual body **omits `max_tokens`**. Returned requested/effective 0/0 does
not establish a usable zero-token generation or an implicit 512-token request;
the input is already incompatible. No default output budget is guessed.
The exact first-input minimum is **12,146 tokens**. Arithmetic sequence lower
bounds are **12,147 for one output token**, or **12,658 for an explicitly chosen
512-token output allowance**, with input capacity at least 12,146 in both cases.
These are necessary bounds for this retained request, **not qualified sufficient
deployment sizes for the entire free/Workflow lifecycle**. Later work, feedback,
fresh identities or a changed output budget require their own exact preflight.
The producer exposes a combined tokenizer count, not an exact per-W/system/Turn/
tool breakdown; no token decomposition is estimated from the byte analysis.

### Stop condition and preservation

Current classification: **DEPLOYMENT_LIMITATION**, specifically
**BLOCKED_BY_DEPLOYMENT_CAPACITY**, proven before generation. This does not
relabel the original 413 FAIL or turn preflight's HTTP 200 into Golden PASS.
Complete external free/Workflow is **NOT RERUN / NOT QUALIFIED** on this resume.
Generation dispatches **0**, fallback **0**, blind/cross-target retries **0**.
The single preflight call is not a generation retry of the earlier indeterminate
Invocation. No effects were attempted.

No YAI runtime consumer adaptation was made: the exact public preflight was
consumed diagnostically using the retained production body. Automatic product
preflight is **not claimed implemented**. Execution admission must use this
truth independently from W sufficiency when that consumer is integrated; no
provider-brand semantic branch or guessed capacity was introduced. There is
no BOUNDARY registration here, so resolver/receipt remains NOT_APPLICABLE.

This resume changes evidence and directly related project-control documentation
only. Transition v18, CaseState v15, Projection/ContextFrame v9 and rendered
input v7 remain unchanged; runtime/schema/semantic-owner/operational-owner/LMDB
delta **0**, LMDB **37/40**. Prior deterministic publication, compiler/locality/
disclosure/delta/replay and Golden local free/Workflow PASS evidence above is
retained, not represented as a new full regression run. Historical W reconstruction
and pipeline equality were rechecked now. No maturity row is promoted.
ZERO-TO-CURRENT **unchanged** because product commands/lifecycle did not change;
continuity canary **NOT_RUN**; **HUMAN_GOLDEN_CASE = PENDING_OPERATOR**;
**I07 = UNSELECTED**. README and unrelated pending work are preserved.

Next action belongs to the deployment operator: supply an envelope admitting
the measured input **plus output**, then resume this same closure for public
rediscovery and exact-request admission before the full external free/Workflow
gate. Do not run `make test-golden-external-yvex` against the known-insufficient
512-token deployment. No YVEX operational command is invented by this consumer
session, and no human acceptance sequence is handed off before automated PASS.

Final documentation validation is retained separately in
[v3-deployment-validation.jsonl](v3-deployment-validation.jsonl), run
`external-golden-v3-deployment-validation-20260910`, order 1: **exit 0**,
5.876 s. `make check-docs check-layout check-roadmap test-roadmap
check-validation-topology`, `git diff --check`, and the no-runtime-source-diff
check pass. Maturity remains 32/23/9/4 across 68 rows; one selected boundary.
This is documentation/topology evidence, not a re-execution of all runtime
regressions. Final read-only reconciliation still finds HEAD == origin/master
== remote master at `844a708aecda4a41ede552a2f7b329ac4c0338ed`. The evidence and
documentation remain pending alongside preserved earlier edits; there is no
new published completion and the worktree is not claimed clean.

## Latest resume: generation 2 capacity admitted / real execution wait FAIL

**EXTERNAL_GOLDEN = FAIL; same-boundary external closure remains BLOCKED.**
The operator supplied an engine-only reload to 16,384 tokens, not a new YAI
implementation. HEAD/origin/master/remote remained `844a708aecda4a41ede552a2f7b329ac4c0338ed`.
The previous 512-token blocker is resolved for the first request. This does not
close the full product lifecycle or qualify another engineering boundary.

### Public rediscovery and preflight before the run

[v3-generation2-preflight.jsonl](v3-generation2-preflight.jsonl), run
`external-golden-v3-generation2-20260910`, records exact commands and raw public
responses. Order 1 GET `/v1/models` and order 2 POST of the **original retained
Golden body**, unchanged, both exit 0. Endpoint/model are unchanged:
`http://127.0.0.1:18001` /
`deepseek-v4-flash-mixed-mxfp4-release-v1`.

Public profile **yvex.openai.compat.v3**, **engine_generation 2**; capacity schema
`yvex.execution.capacity.v1`. Artifact/runtime binding/runtime model/specialization
identities match generation 1. New capacity-plan identity:
`41b1af867448aec04f3b2893ec73c9e3043cf640a4bb90039baf1e7696b475a6`.
Input, sequence and maximum requested output capacities are each **16,384**.
Protocol 21, CUDA target-only and revision
`524efe7f67227f47a2fc749ca740f90d2294b937` remain operator attestations, not fields
exposed by the observed public responses. No YVEX internal investigation.

Order 2 admits original 40,277-byte body `aed2b961…`:
**12,146 input**, requested output **0** (field omitted), effective output
**4,238**, `token_capacity_compatible=true`, both input/output exceeded flags
false, `full_requested_output_fits=false`, `execution_or_resources_qualified=false`.
Request identity `d50ee4a94e46c66ef5d2ef5f27d24188957a47d27d77bfd10516b72c4e0af591`.
No inference was performed by this preflight; no resource reservation was claimed.

### Fresh actual product run

[v3-generation2-external.jsonl](v3-generation2-external.jsonl), run
`external-golden-generation2-product-20260910`, order 1 retains the exact
environment and command (`strace ... make test-golden-external-yvex`).
It ran the actual unchanged YAI executable, S/W compiler and compatibility
lowering, with real reference resource peers and **no model fixture**.
Nested fresh run: **`yai-golden-free-xmf3p8vd`**; retained home
`/tmp/yai-golden-free-xmf3p8vd/home`, not an operator canary.

Actual exit **2**, elapsed **441.291 seconds**. The
[unaltered checkpoint stream](v3-generation2-free-checkpoints.jsonl) shows:

- Case initialization/Participants/resources/policy/issue admission reached.
- `/connect` reached `Connected to ...`, text/native functions/JSON qualified.
  The public trace contains two catalog GETs and six synthetic generation
  requests, all with HTTP 200; this is mechanical shape evidence, not Golden PASS.
- Order 63 submitted real `/work` and committed the Turn before dispatch.
- Order 64 records `expected terminal outcome absent`, expected
  `Human review required.`; output shows `Still waiting (300s)`.
- The harness raised at `test_reference_free.py`'s `wait()` deadline and its
  `finally` terminated the terminal process with SIGTERM. Order 65 records
  reference-resource cleanup. Make did not start the external Workflow run.

No HTTP response byte was received for that first real work request. This is
**not a returned 413**, an observed model refusal, or a proved server-side
generation failure. The harness watchdog is the direct failing mechanism:
external terminal wait is `max(timeout, 300)`, measured over the action; transport
also defaults to 300 seconds, beginning later at the HTTP request. The action
deadline can terminate the client before its transport outcome is surfaced and
recorded. Increasing `YAI_PROVIDER_RESPONSE_TIMEOUT_SECS` alone would not increase
this independent harness deadline. Neither budget was changed mid-run.

### Fresh request and postmortem, not a generation retry

[generation2-wire/wire.json](generation2-wire/wire.json) reconstructs exact public
socket writes and read-only stored W/Projection/Frame. New Case/source identities
are distinguished from the older body used for initial admission:

| Property | Exact fresh result |
|---|---|
| S | `semantic-state:sha256:f09c1eefd51b37d85f83779202aefb6c6f6ea43d931ffaabaff8bd4f172f7589` |
| W | `working-state:sha256:ea8d46f1b39c9fd41712bfcf8a3569e1be24649c24911112599a9e6ba2a0c373` |
| Projection / Frame | `projection:0ef53c8ab2faa007` / `context-frame:de0c06b3f82eeb87` |
| W / Projection / Frame bytes | 14,800 / 10,519 / 32,190 |
| Semantic selection | 15 selected / 16 visible / 1 locality omission; 2,446 semantic units; entries equal through all three stages |
| Body / complete HTTP request bytes | 40,277 / 40,438 |
| Body SHA-256 | `61a164eb3ae6f0c2683de6d0fdaa5dff5f1eb12bdef049c946c6c3d8d3fb248f` |
| Received response bytes | **0**; header/body recorded as null, not synthesized |
| Canonical Turn | `conversation-turn:sha256:5f57a707fca626728be613e8b4fa5c884668f2d8d0344e5669c153e342803922` |
| Canonical Invocation | `invocation:case:golden:free:model-prompt-2`, selected exact target `provider-target:47bd733c0bc90492037064be9c7ee454`, attempt 1 |

After the run failed, preflight **order 3** submitted this exact fresh body only
to `/v1/chat/completions/preflight`: HTTP 200, **12,055 input / 4,329 effective
output**, requested output 0, 16,384 sequence/input capacity, compatible true,
execution/resources qualified false. Rendered prompt 38,256 bytes; prompt identity
`07461dc9f47eea085068fb08ae584c3ec5bcb678174e7f39dd8dfb9c8feb543c` and request
identity `5ce7a28ed0c51a917422df1b52461f248fceb8ba49421388bdb8f453ace87db1`.
This is **postmortem accounting**, not claimed pre-dispatch admission of the new
body. Identical body byte counts with changed identities have different token
counts; neither count is substituted for the other. No token decomposition or
whole-lifecycle fit is inferred.

[Inspection evidence](v3-generation2-inspection.jsonl), order 1: reopening only
the history inspection command exits 0; **27 canonical Transitions**, one
ProviderInvocation started and **no ProviderResult**. That pending state plus
complete observed dispatch requires indeterminate-delivery precautions; it is
not a fabricated canonical terminal `DeliveryIndeterminate` record. No retry,
target change, model effect or blind resubmission occurred. These consumer-side
public observations alone did not establish whether time was spent queued,
prefilling, decoding, or elsewhere. The later operator report below supplies
the missing phase evidence without replacing the consumer trace.

Order 2 extracts the trace with explicit `--allow-no-response`; the forensic
analyzer now distinguishes zero response bytes from complete replies. Its default
still requires a complete response; partial replies remain rejected. Order 3
checks the prior complete trace/body unchanged and the current no-response trace
refused in strict mode, and preserves the checkpoint file byte-for-byte.
This is evidence tooling, not a transport or runtime change.

### Current qualification and remaining owner pressure

| Evidence / run | Actual result |
|---|---|
| `external-golden-generation2-regression-20260910` order 1 | [make check characterization test-golden-local](v3-generation2-regression.jsonl): **exit 0**, 351.968 s; no_provider + loopback_fixture, free and Workflow local gates pass |
| `external-golden-generation2-compiler-20260910` order 1 | [8 compiler contracts](v3-generation2-compiler.jsonl): **exit 0**, 1.169 s; locality 8 vs 1,206 transitions selects 6 items each (1,056 / 1,054 semantic units), mandatory constraint retained; disclosure/full-delta/replay checks pass |
| Actual fresh external free | **FAIL** at 300-second action watchdog; no ProviderResult |
| Actual fresh external Workflow | **NOT_REACHED**, no fallback execution |
| Human / canary | **PENDING_OPERATOR / NOT_RUN**, unchanged |

The new YAI-owned qualification pressure is the independent action/transport
deadline boundary: a long-running model request is interrupted by the test before
an application terminal delivery record can be inspected. This is not repaired
by mutating W or by assuming that a larger token envelope guarantees timely
execution. Observed finite-wait behavior is an **EXPECTED_LIMITATION** of the
current budget, with an independently demonstrated YAI harness deadline/diagnostic
gap. No YVEX defect or model-quality failure is established by zero response bytes.

At this stage, same-boundary work required an explicit acceptable external waiting
budget and alignment of the harness observation budget with application terminal
outcomes. Operator-side timing evidence was requested for the one request
(approximately 2026-09-10 14:36:25–14:41:25 Europe/Rome); the response is retained
below. This session does not inspect/administer the producer. Do not restart this unresolved
Turn as a test retry. A further independently identified qualification run needs
an explicit pre-state/budget and exact-request admission, not an automatic repeat
with a larger timeout. No request to redeploy a larger **token** envelope is
justified by this particular failure.

Runtime/Transition/CaseState/Projection/ContextFrame schema deltas **0**;
semantic/operational owners **+0**; LMDB **+0**, **37/40**. S/W and capabilities
were not truncated or rewritten. Production automatic preflight remains absent;
the public contract was consumed diagnostically. BOUNDARY not applicable (no
registered YVEX consumer manifest); REPLAI/YVEX source and pins unchanged.
ROADMAP retains the same BLOCKED boundary and all earned maturity, replacing the
current first-request capacity blocker with this observed execution/wait failure.
**I07 UNSELECTED**, W→E not started. ZERO-TO-CURRENT **unchanged** (no product
command/lifecycle change); no manual handoff before automated external PASS.
No completion commit or clean-worktree claim; earlier README and pending edits
are preserved separately from this closure's evidence.

Final [documentation/layout/roadmap/topology validation](v3-generation2-docs.jsonl),
run `external-golden-generation2-docs-20260910`, order 1, exits **0** in 3.314 s;
`git diff --check` and the runtime-source-diff check pass. Read-only final
reconciliation still reports HEAD == origin/master == remote master at
`844a708aecda4a41ede552a2f7b329ac4c0338ed`. Evidence remains pending, not published
as a completion. No further generation is scheduled by this handoff.

## Operator follow-up: prefill phase and cancellation correlated

**Source authority:** Francesco supplied this producer-side log excerpt in this
conversation on 2026-09-10, reporting actual service PID **3709423**. This is
**operator-supplied external_yvex execution evidence**, correlated with run
`external-golden-generation2-product-20260910` / `yai-golden-free-xmf3p8vd`;
it is not another test run, a new direct observation by this YAI session, or
human Golden acceptance. The operator omitted intermediate prefill updates.
The original source-file pathname, extraction command, source-file digest and
command exit status were not supplied; none is fabricated here. UTC timestamps
correspond to Europe/Rome plus two hours on this date. Supplied excerpt:

```text
12:36:26  HTTP      http-56 openai POST /v1/chat/completions peer=127.0.0.1:50052 received
12:36:26  SESSION   oa-000000000014 created active_sessions=1
12:36:27  REQUEST   oa-000..0014/r41 messages=3 prefix_tokens=0 max_tokens=16384
12:36:27  PROMPT    oa-000..0014/r41 tokens=12055 reused=0
12:36:27  PREFILL   oa-000..0014/r41 phase=prefill completed=0/12055 tokens
12:36:30  PREFILL   oa-000..0014/r41 phase=prefill completed=54/12055 tokens elapsed=3.13s | avg=17.26
tok/s
12:37:27  PREFILL   oa-000..0014/r41 phase=prefill completed=492/12055 tokens elapsed=59.73s | avg=8.24
tok/s
12:38:25  PREFILL   oa-000..0014/r41 phase=prefill completed=660/12055 tokens elapsed=118.16s | avg=5.59
tok/s
12:40:00  PREFILL   oa-000..0014/r41 phase=prefill completed=822/12055 tokens elapsed=212.81s | avg=3.86
tok/s
12:41:04  PREFILL   oa-000..0014/r41 phase=prefill completed=900/12055 tokens elapsed=276.53s | avg=3.25
tok/s
12:41:20  PREFILL   oa-000..0014/r41 phase=prefill completed=918/12055 tokens elapsed=293.21s | avg=3.13
tok/s
12:41:25  CANCELLED oa-000..0014/r41 generated=0 position=918 stop="cancelled" elapsed=298.4s
12:41:26  SESSION   oa-000000000014 closed active_sessions=0
12:41:26  HTTP      http-56 openai POST /v1/chat/completions peer=127.0.0.1:50052 closed status=499
outcome=cancelled error=YVEX_ERR_CANCELLED elapsed=300.543s session=oa-000000000014
```

The operator additionally reports no FIRST, DECODE or DONE for this request;
absence from this shortened excerpt alone would not prove that. Correlation
uses the matching UTC interval, three messages, **12,055 exact input tokens**,
no reused prefix and cancellation at the consumer watchdog interval. Producer
peer `127.0.0.1:50052` is tunnel-side, not the YAI-side socket identity; they are
not conflated. Request/session IDs above are producer evidence, not canonical
Case IDs.

| Boundary | Admitted conclusion / limit |
|---|---|
| Capacity | First request fits generation 2's 16,384-token envelope; no new 413 or input-capacity failure |
| Phase | Operator report localizes the wait to **prefill**, not completion decoding; last progress **918/12,055**, **293.21 s**, reported average **3.13 tok/s** |
| Output | **0 generated**, consistent with absent ProviderResult and zero received response bytes |
| Cancellation | Producer reports this request cancelled and session closed; its internally logged **499** was not received by YAI |
| Canonical truth | Invocation remains without ProviderResult; logs do not manufacture a terminal Transition or authorize redispatch |
| Diagnosis | Prefill throughput decreases across the supplied samples; no kernel, memory, hardware or algorithmic bottleneck is identified |

The server's `max_tokens=16384` log field does not rewrite YAI's exact body,
which omitted `max_tokens`, or preflight's requested/effective output values
**0/4,329**. Those are different observed representations; no additional
user-requested output budget or output-capacity defect is inferred.

**YVEX EXTERNAL FINDINGS: YVEX_CANDIDATE** — producer-owned investigation of
prefill execution latency for this admitted workload, not an established private
implementation defect. The test's action/transport deadline interaction remains
separate **YAI-owned qualification pressure**. Raising the deadline alone would
neither explain nor qualify this workload. Decreasing throughput does not justify
linear extrapolation to a completion time or choosing a timeout blindly; no
further token-capacity increase is supported by this evidence.

Next coordination is producer-owned diagnosis and bounded execution evidence for
the exact workload already supplied. YAI does not enter producer internals.
Then establish an evidence-based waiting budget and align YAI harness/transport
observation boundaries for a separately identified qualification run. No blind
generation retry, runtime mutation, remote admin, automatic canonical
reconciliation or BOUNDARY registration occurred. The reported producer `.1`
and QA remain operator-owned open work, not completed by this evidence update.

**EXTERNAL_GOLDEN remains FAIL / closure BLOCKED**; Workflow NOT_REACHED.
Previous local publication/Golden/compiler PASS remains scoped to its recorded
runs; no new runtime regression or external execution is claimed. Runtime/schema/
semantic-owner/operational-owner/LMDB deltas **0**; LMDB **37/40**.
ZERO-TO-CURRENT unchanged; canary NOT_RUN; HUMAN_GOLDEN_CASE PENDING_OPERATOR;
I07 UNSELECTED. Evidence/docs only, preserving prior edits; no completion commit.

[Documentation validation](operator-prefill-docs.jsonl), run
`external-golden-operator-prefill-evidence-20260910`, order 1: **exit 0**,
1.105 s. Documentation/layout/roadmap, static topology, whitespace and no-runtime
diff checks pass. This is no_provider documentation proof, not a new producer
run or a promotion of the operator excerpt into canonical execution truth.
YAI does not restart/administer YVEX or change the operator's SSH tunnel.
