# Separate model replies, YAI notices and explicit diagnostics

Baseline: `0de04225f7bbb381083c1ab9a7155fec189ce186`, reconciled clean
HEAD/origin/master/remote master. Intended commit:
`fix: separate conversation replies from system notices`.
The containing commit identity belongs to the post-publication handoff.

## Scope and ownership

Normal SEND/retry/Case work emits separately framed `[Model]` and `[YAI]` blocks,
never a fused response containing a model reply followed by raw diagnostic JSON.
Connection progress uses `[YAI connection]`. Application wait workers finish
before buffered model output is rendered; no new streaming or terminal engine.
REPLAI remains the exact operator-selected `6365f84` native editor.

`/details` is an explicit process-local inspection of the last execution,
connection or failure. Current authenticated Case access is checked before it is
shown; subsequent actions clear the display snapshot. This is not history or an
authority owner. Canonical lineage remains inspectable through existing
`/history`, `/provider`, operation inspection and the one-shot structured CLI.
Restart discards the display snapshot, not Case state. Existing explicit
administrative/resource inspection actions retain their exact structured output;
this change is not a redesign of every slash command.

Model text is control-escaped and line-indented inside a distinct block. A model
cannot print a column-zero closing marker and impersonate a host notice. No
model prose sets execution status. Failures with no provider output have no
model-reply block. HTTP 413 gets a human explanation. This neither changes its
existing delivery-indeterminate posture nor permits retry or discards context.

Transition v18, CaseState v15, ProviderQualification v5, LMDB 37/40 unchanged.
Semantic/operational owners +0; database/schema/execution decision delta 0.
The controller's old stdout-oriented work summary is removed; exact steps and
IDs remain in its unchanged serializable result. No provider transport,
qualification algorithm, authority or retry semantics change.

## Archaeology

Re-read current controller results, REPLAI consumer, native interaction API,
probe reporting, work summary and the real PTY/Golden assertions. Historical
`yai-dev@5c1c7b9` `src/case/surface/case_reply.c` preserves exact machine replies;
`src/runtime/operator/terminal_utils.c` separates human and machine rendering
(history `cffb318b9`, `e9ad7f498`). Recover the presentation distinction, not its
old protocol/session owner, C renderer or alternate terminal implementation.
Current source history `f6c7c8b`, `953f3d8`, `51dd772`, `35705ff`, `0de0422`
shows raw debug lineage used as a test observation: move that observation to
explicit details/canonical reads rather than deleting the protected proof.

## Evidence contract and current checkpoint

Retain actual runs via `../provider-connect-hardening/capture.py`: exact command,
cwd, environment, material pre-state, run/order, exit and bounded unedited output.
All Cargo uses `TMPDIR=/tmp CARGO_TARGET_DIR=target`; PTY runs use
`build/r4-venv/bin` on PATH. Tests use fresh temporary homes, never the operator's
Golden home or canary. `CARGO_NET_OFFLINE=true`; PTY/socket/process-confinement
qualification runs outside the restricted sandbox with the existing test
contract. Orders identify command starts, not completion order of concurrent
gates. [gates.jsonl](gates.jsonl) retains successful and failed attempts separately.

| Order | Command | Proof / provider | Exit | Seconds |
| --- | --- | --- | --- | --- |
| 1 | `make build-rust smoke-provider-connection smoke-case-workbench smoke-guided-case-setup smoke-replai-terminal` | product/contract, loopback | 2 | 30.04 |
| 2 | same focused group | product/contract, loopback | 2 | 26.30 |
| 3 | `make test-fast smoke-replai-terminal` | unit/component no_provider; product/recovery loopback | 0 | 22.87 |
| 4 | `make test-golden-local` | product/recovery, loopback | 2 | 7.76 |
| 5 | `make test-golden-local` | product/recovery, loopback | 0 | 35.06 |
| 6 | `make test-release characterization` | publication union: unit/component no_provider; contract/product/recovery loopback | 0 | 286.91 |
| 7 | `cargo clippy --locked --manifest-path cmd/yai/Cargo.toml --all-targets -- -D warnings` | static, no_provider | 0 | 2.97 |
| 8 | `make test-golden-local` with retained product records | product/recovery, loopback | 0 | 35.13 |
| 9 | `make build-rust smoke-provider-connection check-docs check-layout` | product/contract loopback; documentation/static no_provider | 0 | 13.93 |

Attempts 1–2 exposed old terminal-text assertions, not canonical execution
failures: align the human refusal mapping and inspect the exact code through
`/details`. Attempt 4 exposed a driver variable shadowing its evidence pathname
with response bytes after completing the prerequisite/review path. Fix the
driver, retain that failure, and repeat with fresh homes. No authority assertion
was dropped to obtain a pass.
Order 9 repeats the focused connection suite after a final wording-only
correction to its progress message and cumulative runbook/report updates.

Actual product evidence, distinct from aggregate gate excerpts:

- [product.jsonl](product.jsonl): separate PTY connect runs, their run IDs and
  command/action order; catalog/approval/capability failures and exact evidence
  remain tested. No automatic final JSON; explicit `/details` recovers it.
- [work.json](work.json): exact real Case work through native loopback functions,
  human presentation plus explicit detail assertions and canonical CLI reads.
- [terminal.json.gz](terminal.json.gz): unedited compressed evidence copied from
  `/tmp/yai-r4-xtnwmm5y/evidence.json`, order 3. Native REPLAI PTY SEND commits
  before provider completion; details correlate exact target/plan/lane/result.
  Real HTTP 413 and lost-response fixtures produce no model-reply block and no
  retry redispatch. A model emitting a fake closing delimiter or ANSI controls
  cannot become a host block. The pure renderer invariant is separately
  classified in `tests/classification.tsv` as unit/no_provider/fast.
- [golden-free.jsonl](golden-free.jsonl), run `yai-golden-free-wamk5kl3`, and
  [golden-workflow.jsonl](golden-workflow.jsonl), run
  `yai-golden-free-isv8my17`: order 8 uses
  `YAI_GOLDEN_EVIDENCE_PREFIX=$PWD/refoundation/validation/conversation-presentation/golden`
  to retain complete separate real-product records (not reconstructed outputs).
  This intentional repeat of order 5 is for evidence retention, not a new gate.
  Free/Workflow execute 15/18 fixture model dispatches respectively; human review,
  exact retry recovery, replay, derived rebuild, isolation and Handoff assertions
  remain intact. Each file records its own Turn/review IDs and material pre-state;
  no identifiers are combined across the two histories.

Topology audit reports old smoke coverage preserved (71 leaves), zero repeated
Make leaves in the publication/characterization union. Counts supplement, not
replace, the proof/provider classifications above. No performance claim is
derived from local fixture timing.

Pre-publication checkpoint: local release/characterization, Golden, documentation
and layout pass. Order 10 retains the final format/diff/documentation guard run;
the staged ownership audit admits only presentation, its tests, navigation and
actual evidence, not engine/schema/provider execution changes. Publication
status belongs to the handoff.

## External and product limits

YVEX EXTERNAL FINDINGS — EXPECTED_LIMITATION: no new live inference in this presentation-only change;
the operator is using the public target. Known HTTP 413 input-capacity refusal
and serial synthetic probe latency remain open, as recorded in
[single connection](../single-connect/REPORT.md). No YVEX source, engine,
configuration or private protocol is inspected or administered. No full external
Golden success is claimed. HUMAN_GOLDEN_CASE = PENDING_OPERATOR;
CONTINUITY_CANARY = NOT_RUN. The cumulative ZERO-TO-CURRENT runbook is updated
for normal output versus explicitly requested evidence. I07 remains unselected.
