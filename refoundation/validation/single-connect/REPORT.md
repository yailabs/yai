# One Case provider connection action

Baseline: `35705ff1ee979e12e3c56d15275b43a0212f248a`, clean master with HEAD,
origin/master and remote master equal before mutation. Intended commit:
`fix: unify Case provider connection without profile suffix`.
The containing SHA is recorded in the final handoff, not this report.
Implementation and focused qualification complete. The interrupted publication
checkpoint below is retained; the operator subsequently confirmed and published
the REPLAI dependency change as `34f263c1b4423b413b34c3dd40c0cf421a9d34a2`.
Combined-worktree validation passed against that preserved parent. Ready for
isolated commit/push; the final handoff records publication identity/equality.

## Decision and ownership

Remove the invented `/connect workbench` command, help/completion entry and
backend `case_work` connection selector. `/connect` is the sole guided product
action. Its retained complete endpoint/model form uses the same controller.
No alias or profile chooser replaces the removed suffix.

The existing qualification owner tests text, native function/result round trips
and JSON. Text must qualify to bind PrimaryConversation. Independently proven
optional shapes remain usable; failed shapes remain unqualified with explicit
failure evidence. Connection output reports exact `capabilities`,
`realization_shapes` and `qualification_failures`, not a blanket profile label.
Execution still requires the exact mechanical shape, current semantic evidence,
governance and Case authority. This is not silent fallback from a requested
capability: connection no longer requests a named all-or-nothing capability bundle.
Golden explicitly requires tools and JSON before starting its complete lifecycle.

Singleton discovery, multiple-model selection, conditional scope/authentication,
explicit consent, operator-attested suitability, exact pinning/replacement and
generation revalidation remain. No model-name inference, target substitution,
automatic retry, terminal mechanics or provider administration is introduced.

Transition v18, CaseState v15, ProviderQualification v5, LMDB 37/40 unchanged.
Semantic/operational owners +0, databases +0, canonical schema delta 0. The
observable controller JSON replaces `connection_profile` with exact capability
status; it is not canonical state. This delivery does not edit the REPLAI pin or
root README; concurrent worktree changes are distinguished below.

## Archaeology and protected properties

Directly re-read current qualification, realization, controller, terminal and
tests plus history `f6c7c8b`, `953f3d8`, `51dd772`, `35705ff`. The profile split
in `51dd772` protected text-only use while withholding unproven tools/JSON.
Recover that property at the existing exact-shape boundary, not as product modes.

Re-read `yai-dev` at `5c1c7b9d099eea9f2947146cd821d6501c4a6ddf`,
`src/models/registry/model_alias.c` (history `cffb318b9`) and
`src/runtime/provider/runtime_provider_lifecycle.c`. Public catalog/readiness
checks are historical pressure, not evidence for a user-facing connection
profile or capability authority. Alias registries, shell probes and lifecycle
ownership remain rejected; current exact metadata and qualification owners stay.

## Evidence contract

Use the existing `../provider-connect-hardening/capture.py`. Retain exact
commands/cwd/environment, run/order/pre-state, exit and bounded unedited output
in this directory. Product transcripts preserve run-local IDs and raw PTY bytes.
Fresh temporary test homes only; no operator Golden home or canary reset.
Use TEST.TOPOLOGY.0: no-provider unit/component/static; real loopback product,
contract/recovery/Golden; separately labeled real YVEX with no fixture fallback.

The same cumulative `docs/zero-to-current.md` is updated. Human acceptance is
`PENDING_OPERATOR`; canary `NOT_RUN`; I07 remains unselected.

## Recorded local qualification

All commands run from `/home/mothx/computer-science/projects/YAI/yai` with
`TMPDIR=/tmp`; terminal/publication lanes use `build/r4-venv/bin` on PATH.
`gates.jsonl` retains exact commands, exits and bounded stdout/stderr, including
the failed layout attempt rather than replacing its record. Independent gate
orders identify invocation starts, not necessarily completion order.

| Proof class / provider mode | Command / single-connect run order | Result |
| --- | --- | --- |
| Product / loopback_fixture | `make smoke-provider-connection smoke-case-workbench`, 1 | exit 0, 11.45s |
| Product + mechanical refusal / loopback_fixture | `make smoke-case-workbench`, 2 | exit 0, 0.61s |
| Unit/component/static / no_provider | `make test-fast`, 3 | exit 0, 5.90s |
| Golden free/Workflow/recovery / loopback_fixture | `make test-golden-local`, 4 | exit 0, 34.94s |
| Formatting / no_provider | `cargo fmt --manifest-path cmd/yai/Cargo.toml -- --check`, 8 | exit 0, 0.36s |

`product-local.jsonl` retains the real PTY matrix: sole command/help, removed
suffix refused, automatic singleton, exact multiple-model choice, authentication,
text-only/full-capability/malformed responses, stale catalog/Case, cancellation,
replacement, canonical SEND and replay. Same misleading model identity cannot
manufacture tool or JSON support. No historical catalog property was dropped.

Order 2 additionally sets `YAI_WORKBENCH_TEST_EVIDENCE` to this directory's
`product-shape-refusal.json`. Its fresh Case has READY policy and an admitted
filesystem read. A connected text-only target refuses `/work` with
`cognitive_realization_shape_not_qualified`, zero ProviderInvocations and zero
resource observations. Explicit target replacement then permits the same
existing native-work path, real observation, result, replay and no-duplicate
retry. Thus optional qualification is not an execution bypass.

Golden free `yai-golden-free-01pkgvmn` and Workflow
`yai-golden-free-38_j9my0` use the single guided action and independently assert
functions/JSON before work. Actual resource, policy/review, restart, replay and
derived rebuild properties remain exercised; no operator canary is reset.

Clippy order 6 passed but omitted `CARGO_TARGET_DIR=target`, generating a C
compiler probe and headers inside the pre-existing ignored `cmd/yai/target`
cache. Layout orders 7/11 correctly refused that placement. Only the newly
generated `flag_check.c` and `include` directory were moved recoverably to
`/tmp/yai-single-connect-build.i0XwvG/` at orders 9/12; the pre-existing cache
remains intact. Clippy with `CARGO_TARGET_DIR=target` passed at order 10 (2.18s).
Order 13 confirms docs/layout pass after recovery, then fails during the build
for the independent dependency change below. No guards/assertions were weakened.

## YVEX external findings

Endpoint `http://127.0.0.1:18001`, exact operator-exposed model
`deepseek-v4-flash-mixed-mxfp4-release-v1`, public profile
`yvex.openai.compat.v2`. No source, engine, loading or private protocol inspection.
`external.jsonl` records catalog GET exit 0 (0.016s) and the actual REPLAI PTY
`--external --qualification-only` run, exit 0 (155.44s), without fixture fallback.
Run `yai-connect-product-ph77baaz` in `product-external.jsonl` confirms all three
independent capabilities, exact cognitive binding
`case-cognitive-binding:2fdfe87247e7287ed436abf3b5b65065`, and the single connection
action. Classification: **NO_ISSUE** for this setup/qualification scope.

This is real synthetic provider work, not full Golden/human acceptance or a
latency promise. No SEND/resource effects occurred. The previous Case-input HTTP
413 remains unresolved and was not retested or fixed by removing a command
suffix. `GOLDEN_CASE_EXTERNAL_YVEX = NOT_RUN` for the full lifecycle;
`GOLDEN_CASE_LOCAL = PASS`; `CONTINUITY_CANARY = NOT_RUN`;
`HUMAN_GOLDEN_CASE = PENDING_OPERATOR`; ZERO-TO-CURRENT updated in place.

## Publication checkpoint: concurrent work preserved

While `make test-release characterization` (single-connect/5) was running,
`cmd/yai/Cargo.toml`, `cmd/yai/Cargo.lock` and
`tests/characterization/replai-terminal/test_replai_terminal.py` changed outside
this delivery. The pin moved from `df5538c718b8d068432032e7fb116fb8bfab158e` to
`6365f84e12865871bf26ecf0d984b48213d81ebc`. HEAD/origin/remote remained the baseline.
The new dependency was unavailable offline; the gate exited 2 after 156.63s at
`smoke-typed-provider-realization`, with Cargo exit 101. Order 13 encountered the
same dependency limitation. Earlier passing focused/Golden/external runs do not
qualify the concurrently changed pin or a complete combined publication union.

No revert/reset, pin modification, staging of those files or mixed commit was
performed. Coordination with the concurrent delivery is required; then rerun
the complete publication union on the agreed final baseline, inspect ownership
of shared diffs, commit only this change and verify remote equality. The
publication gate was **BLOCKED**, not PASS at that checkpoint.

## Resumed qualification and operator pressure

The operator confirmed ownership of the new REPLAI pin. Its isolated published
commit `34f263c` changes only dependency resolution, pin documentation and terminal
artifact assertions, with its own evidence. It is preserved exactly; this
connection delivery contains none of those dependency or pin hunks. Remote master
was observed at that commit before resumed publication. `resumed.jsonl` records
the exact dependency fetch (order 1, exit 0) and combined build (order 2, exit 0).
All resumed Cargo commands use `CARGO_TARGET_DIR=target` and `TMPDIR=/tmp`; local
PTY/gates additionally use `build/r4-venv/bin` on PATH.

| Proof class / provider mode | Resumed order / command | Actual result |
| --- | --- | --- |
| Publication regression/characterization / no_provider + loopback_fixture | 3 / `make test-release characterization` | exit 0, 299.11s |
| Golden free/Workflow/recovery / loopback_fixture | 4 / `make test-golden-local` | exit 0, 34.76s |
| Static / no_provider | 5 / `cargo clippy --locked --manifest-path cmd/yai/Cargo.toml --all-targets -- -D warnings` | exit 0, 13.32s |

The final union includes the real REPLAI PTY, single connection matrix, Case
workbench mechanical-refusal proof and canonical runtime regressions. Its
topology audit reports preserved historical coverage and zero duplicated Make
leaves within the composite invocation. This qualifies the new pin together
with the single connection delta; it does not fix or qualify the external
capacity/performance pressures below.

Separately, the operator reported successful single `/connect` against the same
public endpoint/model: qualification
`provider-qualification:2a36f2dc4c609ad9a71e7b747f71009c`, binding
`case-cognitive-binding:6a318bac35542d5826a070fc38274c58`, all three shapes true,
no qualification failures. The next `ciao` Turn
`conversation-turn:sha256:de1a6b52c81f6af4503e12cc343f3e0acefb067f776b0090c3771e45f75ce788`
at generation 14 failed with HTTP 413 `request_too_large`, request bytes written
6891. These are **operator-reported** identifiers/outcomes, not a reconstructed
automated transcript or full human Golden PASS.

Source inspection distinguishes the pressures:

- `YAI_DEFECT` (setup latency): the single action currently invokes six serial
  synthetic inference requests: basic text/JSON, ordered text/JSON and native
  call/result. Removing the profile selector does not make qualification a
  metadata-only connection. Eliminating overlapping probes or deferring optional
  qualification requires preserving exact evidence; this delivery makes no
  instant-connection or latency-improvement claim.
- The reported `function_result` 47007ms is measured around the HTTP request,
  not local editor time. Provider/network/queue/model contributions cannot be
  separated by that measurement, and no equivalent llama performance comparison
  was run.
- `YVEX_CANDIDATE` / unresolved public capacity: the earlier independent
  `public-shape-20260908/3` in
  [raw evidence](../provider-connect-hardening/external.jsonl) sent synthetic
  string content with `max_tokens=1` and received HTTP 413, explicitly
  `token output capacity exceeded`. That earlier observation is not a retest of
  the operator's current Turn. No internal limit or remedy is inferred.
- YAI's renderer sends a typed ContextFrame plus exact input, not merely `ciao`.
  The `bytes` diagnostic counts written HTTP headers and body, not tokens or the
  provider limit. Current non-success HTTP classification is conservatively
  delivery-indeterminate; it does not prove model execution occurred. No blind
  retry, history rewrite or context/authority removal was performed.

No additional external inference is queued while the operator uses this target.
Full real Case dialogue and the external Golden lifecycle remain unqualified.
