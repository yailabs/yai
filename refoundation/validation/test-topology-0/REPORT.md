# TEST.TOPOLOGY.0 — validation engineering closure

Baseline: master `6e332851b5066cbb1da25f816b8db31b74580acb` (I04).
Intended commit: `test: separate validation proof and provider execution lanes`.
Pre-publication state: implemented and locally qualified; awaiting isolated
commit/push/remote verification. Final publication identity belongs in the
handoff, not in the commit containing this report.

## Decision and authority

This horizontal engineering closure changes test selection, execution metadata,
launcher wiring and documentation, not YAI runtime semantics. The existing
`tests/classification.tsv` becomes the only classification authority. Its
414 entries cover 330 exact compiled Rust identities, named Make leaves,
assertion/helper files, operator procedures and historical non-proofs.

Proof and provider environment are independent. Provider-shaped values in a
pure/store test are `no_provider`; actual HTTP/TLS peers are `loopback_fixture`;
operator-selected YVEX is `external_yvex`; other real-model procedures are
`external_other`. Regression/characterization/qualification is a third axis,
not a test-pyramid level. Mutation/network/reachability/cadence remain explicit.
The catalog is a reviewed classification, not a sandbox or automatic inference
of what an arbitrary future test can do.

## Recovered engineering property

Direct archaeology inspected current Make recipes, all script consumers,
compiled engine/CLI lists and source tests, `tests/README.md`, old classification,
`docs/test-cases.md`, the YVEX qualification script, and historical testing work.
In `yai-dev`, inspected `tests/README.md`, Make/test history including
`e9ad7f49848b15b2341f99b406c8930d8ca431f8`, the pilot validation bundle and QA.4
test/tool closure. Recovered: assertions stay with tests; tooling selects and
reports execution; nonzero commands cannot become PASS. Rejected: copying old
directories, global `/tmp/yai-validation.out` evidence overwrites, readiness
labels substituting real invocation evidence, and treating historical docs as
current command authority. No YVEX source/admin access was needed or performed.

The old classification's `refoundation/source-refoundation-1/legacy-property-recovery.tsv`
reference is absent at this baseline. It remains explicitly recorded as a
historical unresolved reference, not claimed as executable coverage.

## Gate mapping and parity

| Old entrypoint | Current contract |
|---|---|
| `build-rust` | compile only; no hidden full engine/CLI suites |
| `check` | alias of deterministic `test-release` union |
| `smoke` | preserved named smoke leaves; no implicit full Rust suite via build |
| `characterization` | posture selection, shared dependencies with release |
| `qualification-yvex-provider` | explicit external text lane; also `test-external-yvex` |
| `endurance-agentless-case-runtime` | retained mixed runtime characterization alias; not a new endurance claim |

`test-fast` is no-provider pure/bounded component proof plus guards.
`test-local` includes loopback/product/recovery; `test-release` also retains
formerly default bounded endurance. `test-endurance` adds the three previously
ignored W19/H19/W20 large characterizations. Independent proof-class targets
and metadata inspection localize failures. Manual listing never means PASS.

An executable audit compares actual `make -n check characterization` selection
with the catalog and the immutable old Make graph. All 71 old smoke leaves
remain reachable. The two characterization-only product scripts gain named
leaves; topology assertions and preserved Rust doctests complete the 75 current
Make leaves. All 322 formerly default Rust tests remain publication-reachable;
four ignored CLI proofs remain through their original explicit wrappers.
Three ignored scale tests stay opt-in; the ignored concurrent-build child
remains harness support exercised by its parent, not a standalone pass.

No assertion was deleted. The shell-procedure changes are launcher-only,
except the external YVEX admission/reporting changes described below.
No runtime files or historical I01–I04 dossiers change.

Observed final union: 75 Make leaves, zero duplicate leaf executions, with
55 `no_provider` and 20 `loopback_fixture` leaves; all 11 suite/partition runs
returned zero. These are execution-group counts, not distinct assertion totals.
Focused contract and fast gates also pass independently. Eight topology
falsification tests, registry audit (180 operations, zero help/handler failures),
Bash/manual syntax, fmt, docs/layout and diff checks pass. Clippy passes the
existing repository contract with 12 engine and 13 CLI pre-existing warnings;
this closure does not claim `-D warnings` cleanliness.

## Duplication and failure localization

The generated ordinary Make graph shares leaf dependencies within one
invocation. `make check characterization` does not rerun a completed leaf.
Separate invocations deliberately rerun; no PASS cache hides source changes.
Full suites formerly hidden in build and recursive characterization are gone.
One absolute exported Cargo target directory prevents incidental nested build
trees; validation uses installed dependencies in Cargo offline mode.

Some existing verticals re-run Rust filters and verify their diagnostic output.
They remain intentionally: the output assertions are distinct from simply
passing the Rust test. Their safe future factoring is debt, not proof deletion.
Rust partitioning retains original default concurrency; large opt-in scale
fixtures serialize aggregate memory pressure. Ignored process-environment host
tests retain their original serialized wrappers.

Measured on this host/toolchain (informational, warm builds): old check
211.86 s plus old characterization 241.18 s = 453.04 s; final combined graph
252.91 s, about 44% less elapsed time for the union. This is not a claim that
new check alone is faster than old check: the new union also includes the old
characterization-only proofs and class partitioning changes scheduling.
Fast lane: 4.14 s. The retained bounded endurance Rust partition alone took
88.248 s, making that cost visible instead of hiding it in build. No cached
PASS was reused. The complete opt-in extended endurance lane was not run in
this closure; the incidental H19 scale execution during migration is recorded
separately and not substituted for that claim.

Each Make leaf reports ID/class/posture/provider/property. Rust partitions
report selected identities, exact command, provider modes, exit and duration.
Topology assertions inject a real failing child (exit 7), verify class-localized
nonzero propagation, reject duplicate/missing classifications and forbid
fixture-backed external claims. Compiled ignored-test auditing prevents silent
promotion of large ignored scale tests into publication.

## External YVEX findings

`DEPLOYMENT_LIMITATION`: no operator endpoint/model variables are supplied in
this session. The explicit external lane reports `blocked_external_dependency`,
zero request attempts, and script exit 3 (Make exit 2). No YVEX server/model
was started or inspected/administered. No live interoperability is claimed.

The script no longer guesses port 8001, selects the first exposed model or
probes an optional `/health` extension. It requires exact operator input,
uses `./yai`, supports an optional credential, and reports attempted discovery,
invocation and confirmed result separately. Any supplied ref is explicitly
operator-attributed, not falsely called a remotely observed commit. No fixture
fallback exists. Existing positive proof remains text Chat Completions through
the generic provider transport and legacy Advanced prompt client, not I03/I04
typed media, model quality or performance. Qualification failure is not masked.

## Invariants and remaining debt

Semantic schema delta: zero. Transition v15, CaseState v13 and I01–I04 contracts
unchanged. Semantic owner delta 0; operational owner delta 0; LMDB delta 0
(37/40). Root README untouched.

The fresh manual inspection exposed a pre-existing reporting inconsistency:
`store status` still prints literal Transition/CaseState v8 strings in
`cmd/yai/src/command_adapters.rs`, whereas current authoritative constants are
v15/v13. Raw output is retained; it is not used as schema evidence and is not
silently fixed in this no-runtime-change closure.

Remaining: factor duplicated diagnostic-checking Rust wrappers only with parity
proof; refine mixed verticals if there is real failure-localization benefit;
capture production YVEX text evidence with operator configuration, then qualify
public typed-media composition separately; keep endurance and manual evidence
explicit instead of treating availability/listing as execution. No CI platform,
runtime service, Interlock wave, H20/W21/W22 or provider administration begins.

See [execution evidence](EXECUTION-EVIDENCE.md) and
[manual acceptance](MANUAL-ACCEPTANCE.md) for actual runs and commands.
