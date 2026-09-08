# Provider connection and buffered execution hardening

Engineering closure prompted by real operator failure; not I07.

Baseline: `953f3d85266359974f175f5a981825049e5737aa`
(`feat: simplify Case CLI with guided authoritative setup`). HEAD, origin/master
and remote master were equal before mutation. Intended commit:
`fix: harden provider onboarding against real buffered execution`.
This report describes baseline plus the isolated hardening diff; it does not
claim the containing commit's future SHA. Publication identity belongs in the
post-commit handoff. No YVEX source/admin, operator Case mutation, REPLAI pin
change, or new Interlock work is part of this closure.

## Observed failure and ownership decision

The operator's `/connect` failed mechanical qualification, so no trust or Case
cognitive binding was added. The later canonical `ciao` SEND could not dispatch:
`PrimaryBindingMissing` was missing configuration, not proof that YVEX was down.
The operator's qualification ID was
`provider-qualification:2ddc21d7ad72608272be064fb4c8dd20`; target
`provider-target:47bd733c0bc90492037064be9c7ee454`. Its stored evidence was inspected
read-only. It proved base text/JSON/exact addressing, not the requested typed
shapes/native functions. The supplied server log showed a 330-token function
probe prefill lasting 28 seconds, cancelled around the client's old 30-second
socket budget. This correlation motivated a transport fix; it did not prove a
model or CUDA defect.

Legacy archaeology: `../yai-dev` at
`5c1c7b9d099eea9f2947146cd821d6501c4a6ddf`, relevant historical commit
`dda93ee3a` (`Complete CORE.SPINE Series 1 root drain baseline`),
`src/runtime/provider/runtime_provider_transport_local.c`, already separated
connection timeout from model-response maximum time. Recover that distinction
in the current generic Rust transport. Do not restore its shell/curl execution,
provider registry, or historical ownership. Case/controller and provider
governance remain the existing owners.

## Implemented contracts

- Guided `/connect`: explicit conversation profile, qualifies base and ordered
  text. Guided `/connect workbench`: explicitly requires text, native function
  calls plus correlated function-result consumption, and JSON. The old complete
  `/connect ENDPOINT MODEL ...` form retains full-workbench requirements. No
  failed full-profile request silently downgrades. Explicit trust, operator
  attestation and replacement confirmation remain mandatory.
- Plain text lowers to distinct ordered string messages, including equal
  repeated parts. Context precedes exact source parts. Media retains ordered
  content arrays; no media support is inferred. Canonical source closure,
  original/derived identity, Turn and Invocation lineage are unchanged. Probe
  and production use the same lowering.
- Connection/write and ordinary resource budgets remain 30 seconds. Buffered
  provider response has a total deadline: default 300 seconds, operator setting
  `YAI_PROVIDER_RESPONSE_TIMEOUT_SECS=1..3600`. Invalid settings refuse before
  transport. The remaining deadline bounds reads; it does not reset per byte.
  Expiry after submission remains delivery-indeterminate, with no automatic
  retry or target substitution. This changes permitted waiting, not speed.
- Synthetic probes explicitly bound output to 96 tokens and disclose stage,
  elapsed time and five-second wait updates. Text-only setup does not run an
  unrelated JSON probe. Released-editor execution reports waiting; it does not
  introduce streaming, editable drafts, transport cancellation or a new editor.
- Qualification failure exposes required/proven shapes and bounded error
  categories. `provider show` exposes stored shape, failure and timing evidence.
  Provider HTTP errors preserve a bounded public code, not arbitrary remote
  message prose or credentials. Missing host binding reports
  `provider_unconfigured` rather than `provider_unavailable` (application result
  posture, not a canonical schema change).

No semantic or operational owner added; no database added. LMDB stays 37/40.
Transition v18, CaseState v15, ProviderQualification v5 and all canonical schemas
are unchanged. The changed synthetic typed-content suite identity is
`yai.openai_compatible.typed_content.synthetic.v2`, not a schema bump. No engine
source or canonical reducer is changed. I01–I06 remain complete; I07 unselected.

## Evidence authority and reproduction

All commands ran from `/home/mothx/computer-science/projects/YAI/yai`.
`capture.py` retains exact command/working directory, allowlisted environment,
baseline SHA, run/order/material pre-state, exit and bounded unedited output in
JSONL. Head/tail truncation is explicit. These are observations, not a PASS
cache. `product-external.jsonl` and `workbench-external.jsonl` retain ordered raw
PTY bytes as hex plus independent canonical inspection. Decode a field with
`bytes.fromhex(value).decode(errors="replace")`; no transcript was reconstructed.
Each `yai-connect-product-*` identity is an independent disposable home, not the
operator's Golden Case. Do not mix their Turn or invocation IDs.

Publication uses `TMPDIR=/tmp` and the existing `build/r4-venv/bin` on PATH;
loopback/socket execution requires the host's test permission. The inherited
cache TMPDIR was not writable in the sandbox. No dependencies were downloaded.
Provider modes follow TEST.TOPOLOGY.0, not aggregate counts.

| Evidence | Scope and actual outcome |
|---|---|
| `gates.jsonl`, `hardening-gates` order 1 | `make test-fast`, no_provider, exit 0 |
| order 3 | `make test-golden-local`, loopback_fixture, exit 0; free and Workflow reference resource lifecycle |
| order 7 | PTY terminal regression and strict text-provider connection, loopback_fixture, exit 0; 16.60s |
| order 8 | `make test-release characterization`, no_provider + loopback_fixture, exit 0; 270.24s, shared Make leaves executed once |
| order 9 | docs/layout/topology, no_provider, exit 0; historical coverage parity, zero duplicated composite leaves |
| orders 10–11 | Cargo fmt check and all-target Clippy with warnings denied, no_provider, exit 0 |
| order 12 | Final `make test-golden-local`, loopback_fixture, exit 0; 34.56s |
| `external.jsonl`, `public-shape-20260908` order 1 | Actual public array-text request: HTTP 400 `invalid_request` / `invalid JSON string` |
| same run, order 2 | Actual distinct string messages: HTTP 200, exact exposed model, `Hello`, 13 prompt tokens |
| `public-product-20260908` order 2 | Real PTY text connection passed; canonical cognitive SEND failed HTTP 413; driver exit 1, 14.98s overall |
| `public-workbench-20260908` order 1 | Actual synthetic full-profile qualification through guided PTY, exit 0, 135.83s; not full Case execution |
| `public-shape-20260908` order 3 | Independent synthetic 6000-character string, max_tokens 1: HTTP 413 `request_too_large`, `token output capacity exceeded`, 2.15s |

`curl` exit 0 in these diagnostics proves completed HTTP transport, not HTTP
success: the retained status/body are the qualification posture.

The final strict-text product run `yai-connect-product-arjq4__e` proves full
profile refusal without binding, explicit text connection, Turn commit before
provider result, exact cognitive lineage and replay-equivalent CaseState v15
at generation 17. Its model `vision-whisper-only-a-name` grants no media or tool
meaning. Existing I04 order assertion still proves four distinct text inputs;
fixture logging now additionally identifies `string_messages` wire layout.
Final Golden free home: `yai-golden-free-0xqp0ydj`; Workflow home:
`yai-golden-free-2nfm7dit`. These separate reference runs use real filesystem,
process, SQLite, HTTP and MCP paths and retain existing review, restart, replay,
derived rebuild and isolation assertions. Workflow run records 18 model fixture
dispatches, not 18 real YVEX requests. The full publication union includes
contract, product, recovery and existing publication-endurance assertions;
classification audit reports 471 entries, 370 Rust identities, 85 Make leaves,
and preserved reachability from the 71 historical smoke leaves. Counts are
supplemental, not external compatibility claims.

Retained development failures are not hidden: `hardening-local` orders 1–2 and
`public-product-20260908` order 1 were interrupted test-driver attempts. Local
order 1's pre-state assumption of a fresh binary was wrong after the sandbox
TMPDIR build failure; it is not fixed-binary evidence. The external first driver
failed to stop promptly after the same 413, so its 417.82s is not provider latency.
The final driver fails immediately on terminal refusal/failure. Gate orders
2/6 exposed array-specific fixture observation/response assumptions, now updated
without dropping the order or ORCHID output oracles. Order 4 exposed a stale
arbitration guard preceding the duplicate-step guard; the test accepts either
typed rejection and still requires unchanged canonical state. Order 5 exposed
an obsolete five-request expectation after removing an unrelated JSON probe;
the fixture now expects four requests and fails boundedly instead of hanging.

## YVEX external findings

Operator-supplied public endpoint: `http://127.0.0.1:18001` on Exon, forwarded by
the operator to Spark's loopback OpenAI service. Exact catalog identity:
`deepseek-v4-flash-mixed-mxfp4-release-v1`. Runtime commit/ref was not independently
inspected: this is black-box consumption, not source qualification. The operator
reported YVEX 0.1.0/protocol 20; no inference is made about a private contract.

`YAI_DEFECT`: old 30s response allowance, opaque failure/wait UI, mandatory
full-profile setup for ordinary text, and array-only text lowering were exposed
by this deployment and addressed generically. Live native function call and
result consumption now passed at **57890ms** and **46706ms** respectively. This
refutes a blanket claim that this public target cannot use functions. These
times are informational observations, not a throughput benchmark or instant
setup promise.

`YVEX_CANDIDATE` / outstanding deployment boundary: a real minimal fresh Case
SEND (request bytes written 6153) returned HTTP 413 `request_too_large`.
Separate synthetic input reproduced the explicit `token output capacity exceeded`
message with max_tokens 1. Exact internal capacity, threshold, cause and remedy
remain unresolved. We do not infer a 4096 limit, reduce authority/context
material, or administer the provider to hide it. The YVEX operator/session must
establish a public input-capacity contract that admits the real Case request.

Failed external test Turn:
`conversation-turn:sha256:c4e676b040724de3a6fa3015c83bdf2bc8cac74eeee63cecba6919595af9ca46`
in `yai-connect-product-hcrmiee2`. It remained delivery-indeterminate without
ProviderResult or retry. It is not the operator's original un-dispatched `ciao`
Turn. No possibly completed work was blindly repeated.

## Closure boundary

YAI onboarding/transport hardening is internally qualifiable; **real full Case
dialogue and Golden YVEX lifecycle are not PASS**. Small text and synthetic
native functions passed independently. No full Golden external effect was
attempted after the demonstrated context rejection. The cumulative runbook is
updated in `docs/zero-to-current.md`; the operator must resolve the current
deployment limitation before following its real-model lifecycle.

`HUMAN_GOLDEN_CASE = PENDING_OPERATOR`.
`CONTINUITY_CANARY = NOT_RUN`; private operator state was preserved.
Remaining pressure: public provider input-capacity evidence and admission;
subsequent real full-context/Golden qualification. External media, target quality,
performance optimization and streaming remain unclaimed. No I07 is started.

Pre-publication qualification: local publication/characterization and Golden
gates PASS; docs/layout/topology, formatting and Clippy PASS. Explicit staged
whitelist: current Rust application/transport changes, corresponding fixture and
test metadata, bounded documentation/runbook and this evidence package. No
engine schema/reducer, root README, REPLAI vendor/pin, operator Case, or YVEX
implementation is included. Final staged-diff inspection and remote equality
are publication actions, reported in the post-commit handoff.
