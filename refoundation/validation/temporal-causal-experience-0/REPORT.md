# TEMPORAL.CAUSAL.EXPERIENCE.0

Baseline: `6ac5f1b2706809e70b9d54f71ce8a49c6e544193`, existing physical master.
HEAD/origin/master/remote master reconciled equal before mutation.
Intended isolated commit: `feat: derive qualified temporal experience relations`.
Pre-publication state: implementation, deterministic publication/characterization,
Golden local free/Workflow, targeted and documentation qualification PASS;
ready for isolated staged-diff inspection and publication. Actual containing SHA
belongs in the final handoff, not this containing report.

## Ownership and bounded contract

`graph::experience` is a derived inspection contract in the existing graph/access
owner. One shared historical read snapshot supplies replay-qualified, currently
disclosed source evidence. No second historical engine, canonical event owner,
Episode store, causal database or persisted graph-as-truth. The product is
`yai case experience CASE current|GENERATION|TRANSITION`, with optional exact
directed path, hop/output bounds, human-readable backing and structured JSON.

Temporal order means recording sequence, not real-world occurrence order.
Observation time comes only from typed producer fields; occurrence time remains
absent. Late result JSON describing an earlier occurrence does not become a
timestamp authority or knowledge at an earlier cut. Exact policy replacement
remains lifecycle, not P1 causing P2 or changing D1 retroactively.

Relation classes: recording precedence; structural proposal/operation/decision/
review/Grant/PREPARE/result/content lineage; DecisionBasis normative/evidence
support; recorded receipt/observation consequence; explicit replacement/unbinding
and invalidation. Provider claim posture is independent of the structural origin
relation. No generic physical-causal, inferred or model-proposed causal edge is
implemented. Missing required backing suppresses qualified edges without erasing
recorded existence. Historical source inspection retains detailed missingness.
Path anchors require backing even for a zero-edge path. Ambiguous object aliases
refuse the whole view rather than guessing a historical occurrence; general
multi-occurrence object navigation is not claimed.

W20 MemoryEpisode derivation is unchanged. Scoped inspection slices use exact
typed operation/result/Turn keys, never arbitrary nested JSON fields. They are
distinct disposable view identities, not replacement Episode records. A typed
origin can connect two actual W20 Episodes without merging them. General W20
assertion contradiction, Workflow/Handoff traversal and query-conditioned Recall
remain outside this initial profile. Existing contradiction/supersession tests
remain part of the publication union; recency is not relabelled contradiction.

Source qualification reuses the bounded historical profile. The result identity
binds schema, cut, disclosed events/relations, query and current own disclosure;
it excludes private whole-history/scope hashes. Hidden versus absent intermediates
produce the same path result, labels/counts/IDs and explanation. Unknown/hidden
anchors share a refusal. The public reader always requalifies current access;
this view cannot authorize an Operation or feed W without a future qualified seam.

## Archaeology

- Current Transition validation requires exact typed lineage IDs in `causal_refs`
  (Invocation/Result, Operation origin, DecisionBasis, Grant/PREPARE, review,
  binding replacement, Workflow and Handoff). The field also accepts extra
  nonempty IDs. Neither the field name nor an extra ID proves physical causality.
  The relation reader resolves the actual typed payload fields instead.
- Existing LMDB `derive_graph_relations_from_transition` and ephemeral RuntimeGraph
  provide object/ref materialization, not this scoped temporal/source-closure
  contract. They remain compatibility/access consumers; cached edges do not
  establish new experience truth. HistoricalSemanticView supplies the stronger
  current-disclosure and exact-cut foundation; its code is shared, not copied.
- W20 structural grouping excludes common Case/resource identities but recursively
  collects payload field names. It remains a derived grouping, never causal proof.
  The new disclosed slices therefore use only typed top-level lifecycle keys;
  no payload prose/JSON is interpreted as a relation or a hidden join key.
- `yai-dev` `45c36bd0c`: `include/yai/graph/lineage.h`,
  `lib/graph/materialization/from_runtime_records.c`, QG-2
  `docs/architecture/unified-graph-workspace-edge-model.md`, and
  `tests/integration/workspace/workspace_graph_read_surfaces_dp16_v1.sh`.
  Useful recovered discipline: exact anchors, distinct observed/accepted/
  canonicalized posture and graph presence not implying authority. That runtime
  used process-global Workspace counters/string IDs and optional Redis emission;
  neither ownership nor source-plane/Workspace registries are recovered. The
  later deletion/layout cutover is `1c934d3a5`, not evidence of a removed general
  qualified temporal-causal engine. Current typed history/admission is stronger.
- The old `yvexlabs/yvex` public roadmap URL returned HTTP 404. Discovery of the
  current [public YVEX roadmap](https://github.com/yailabs/yvex/blob/models2/ROADMAP.md)
  succeeded (HTTP 200 on its raw `yailabs/yvex/models2/ROADMAP.md` surface).
  Its current snapshot/target sections keep MAINTENANCE.ARCHITECTURE.REFOUNDATION.1
  active, A01 behind consumer cutover and native cognitive state OPEN. The producer
  separately reports a 12,055-token workload reaching 10,308 prefill tokens in
  603.98 s with zero generated and HTTP 504. That is producer-published evidence,
  **not a new YAI external run** and not a replacement for YAI's retained 300 s
  failure. No implementation, CLI, runtime or live endpoint was inspected or
  administered. The YAI/YVEX ownership boundary is preserved.

## Qualification

All commands below ran in `/home/mothx/computer-science/projects/YAI/yai` on the
baseline plus the source/test bytes in [source.sha256](source.sha256). The manifest
was checked before and after the final publication run and after targeted reruns.
Documentation closure follows that frozen source run. No operator-owned home,
canary or external provider was used. Every test creates its own disposable state;
product subruns record the actual home, command, execution order and raw outputs.

Retained output is losslessly JSON-line encoded: `combined_output_line` is one
original combined stdout/stderr line. `jq -r '.combined_output_line' FILE` restores
the raw stream, verified byte-for-byte against the captured files. These are actual
outputs, not reconstructed transcripts. Run IDs below label independent invocations;
their identifiers must not be combined into one causal history.

| Order / run ID | Exact command and material pre-state | Exit / evidence |
|---|---|---|
| 1 / development-union | `make check characterization test-golden-local`, QA venv, `TMPDIR=/tmp`, disposable tests; development tree before the final manifest | 2; [bounded original tail](development-union-tail.jsonl), not qualification |
| 2 / development-diagnostic | Compiler diagnostic of the failed development test build; the full selector invocation was not retained, so this is not an independently reproducible qualification record. No test ran. | 101; [compiler output](development-diagnostic.jsonl) |
| 3 / qualified-publication | Command block below; final manifest, QA venv, external provider and operator-home environment unset, fresh disposable fixture/product state | 0; [complete publication output](publication.jsonl) |
| 4 / final-contracts | `TMPDIR=/tmp CARGO_TARGET_DIR=target cargo test --manifest-path engine/Cargo.toml resource_access_tests::historical_tests -- --test-threads=1 --nocapture`; unchanged manifest, isolated LMDB fixtures | 0; 11 passed, [complete output](contracts.jsonl) |
| 5 / final-operation | `TMPDIR=/tmp CARGO_TARGET_DIR=target cargo test --manifest-path engine/Cargo.toml resource_process_canonical_prepare_fence_real_exit_terminal_and_replay -- --test-threads=1 --nocapture`; unchanged manifest, test-owned confined process and fresh LMDB | 0; 1 passed, [complete output](operation.jsonl) |
| 6 / final-documentation | `TMPDIR=/tmp make check-docs check-layout test-roadmap test-topology`; source unchanged, roadmap closed at bounded scope, no product state | 0; [complete output](documentation.jsonl), 8 roadmap and 10 topology tests |

Final publication invocation:

```sh
source /tmp/yai-governance-qa-venv.Jf2vcj/bin/activate
env -u YAI_EXTERNAL_PROVIDER_BASE_URL -u YAI_EXTERNAL_PROVIDER_MODEL \
  -u YAI_EXTERNAL_PROVIDER_LOCALITY -u YAI_HOME TMPDIR=/tmp \
  make check characterization test-golden-local
```

The publication union passes all selected component, contract, endurance,
endurance-characterization, recovery, unit and unit-characterization Rust
partitions, plus product/characterization leaves. TEST.TOPOLOGY discovers 502
entries, 93 assertion files and 397 Rust tests; those are catalog counts, not a
substitute for the executed proof/provider partitions in the log. New evidence
comprises five Rust properties and one no-provider product leaf; existing
historical disclosure/review and real process lifecycle oracles are extended.
Final documentation/link/layout checks pass. Roadmap's read-only generated
summary remains ESTABLISHED=32, PARTIAL=24, OPEN=11, LATER=4, TOTAL=71; no maturity
row was promoted. `git diff --check` passes. These documentation checks do not
certify runtime maturity or external acceptance.

Product run `yai-experience-product-xw2tgcxi` starts with a fresh home and executes
34 actual `./yai` commands: policy ingestion/validation/publication, Case binding,
replacement, inspection at exact cuts, human-readable path, revoke/unbind,
negative requests, rebuild, history equality and replay verification. Bounded
unedited stdout excerpt (same run, after those ordered commands):

```text
experience_product=PASS provider=no_provider policy_replacement=qualified unbinding=qualified current_permission_not_restored=true human_readable=true restart_equal=true reads_append=0
```

Separate Golden local product subruns inside `qualified-publication`:

| Run ID | Final record | Scope / provider | Observed result |
|---|---|---|---|
| `yai-golden-free-wykegks0` | order 164 | `golden_free` / `loopback_fixture` | PASS; 15 model-fixture dispatches; no external request |
| `yai-golden-free-pdw0g8xz` | order 148 | `golden_workflow` / `loopback_fixture` | PASS; 18 model-fixture dispatches; no external request |

Both retain `HUMAN_GOLDEN_CASE=PENDING_OPERATOR`. Their product command streams,
recovery/rebuild and canonical identifiers remain in the complete log. Neither
is external YVEX evidence or operator acceptance.

### Relation and negative oracles

- `final-operation` performs a real confined process whose exit is 7, not a
  synthetic successful effect. Its qualified four-edge path is Operation →
  Decision → Grant → PREPARE → receipt/observation. Restart does not redispatch.
  Bounded unedited output from this run only:

```text
resource_process: operation=operation:8da7d0d76575a86f0ff2313a1b92a8d3 grant=grant:94d018530d00b51fac10226092f15d42 effect=effect:94d018530d00b51fac10226092f15d42 receipt=effect-receipt:07bd3f176301157a320efddce0994836 exit=7 replay=equal restart=no_redispatch provider=no_provider
```

- `final-contracts` policy oracle: P1 binding
  `case-policy-binding:89154be7d2934bcade6ed0b71f562f7a` supports D1
  `decision:461c72b3e28e1753a817d7f48a4c9b92`; P2
  `case-policy-binding:12ea50de9e569df00813b5682f0d1d2c` replaces it at generation
  12 and supports D2 `decision:00efe97119e560ed915e4da85063d39b`.
  D1 stays historical and unchanged; same resource/nearby chronology creates no
  causal path. Late evidence is absent before recording; occurrence stays missing.
- The review oracle separately proves Decision → ReviewRequest → ReviewAction →
  re-evaluated Decision, retaining historical basis after replacement/unbinding.
- Paired qualified-input derivation fixtures differ only in the exact provider
  origin reference: edge with the reference, no edge without it. These fixtures
  qualify the constructor, not fabricated admitted provider execution. A separate
  committed typed filesystem-proposal history links `provider-result:experience`
  to `operation:7aed61a831822f2cca99a6ab5176b373` across two actual W20 Episodes:
  `memory-episode:3cfde6d824ebc699` and `memory-episode:fd81d09208cc72c0`.
  The candidate remains a ProviderClaim; its normalized operation is DENIED.
- Similar/adjacent material, extra `causal_refs` and arbitrary result JSON IDs do
  not create causal edges. Recording-only paths require explicit opt-in.
- Current two-Principal isolation, hidden-versus-missing intermediate equality,
  hidden anchor refusal, other-Participant scope noninterference and own-scope
  reidentification are executable negatives, not presentation filtering claims.
- Restart, graph/memory/context/effective-policy drop/rebuild and missing immutable
  policy backing are tested. Source loss removes qualified edges; budgets refuse.
  History and replayed CaseState remain equal and all inspection adds zero
  Transitions. No provider inference reconstructs these semantics.

### Cost characterization

`final-contracts`, same source and fixed scoped semantic task; 400 irrelevant
Participant-change Transitions distinguish the longer Case. Original measured
values, not production limits:

| History / inspected | Visible events / slices / relations | Historical reconstruction | Relation build | Build plus bounded path | Path output |
|---|---|---|---|---|---|
| 10 / 10 | 3 / 1 / 5 | 21,053 µs | 745 µs | 448 µs | 2 events / 1 edge / 2,723 bytes |
| 410 / 410 | 3 / 1 / 5 | 52,960 µs | 743 µs | 455 µs | 2 events / 1 edge / 2,724 bytes |

The path measurement includes relation construction; it is not a cached graph
traversal-only benchmark. Output stays bounded while source qualification scans
history and reconstruction cost grows. No Case-age-independent CPU claim or new
index/snapshot optimization is earned.

### Development findings retained, not counted as PASS

Development caught a path tie-break issue: hash sorting could
choose auxiliary support instead of the direct lifecycle edge. The constructor
now orders equal destinations by stable semantic kind; both edges remain in the
full view. A deliberately under-specified native-provider fixture was refused by
existing admission and corrected to the existing typed filesystem-proposal
history profile; admission was not weakened. A later development union stopped
at `smoke-second-carrier`: a test helper named `path` was shadowed by a PathBuf in
the newly added missing-backing negative (E0618). This was a compile failure,
not a carrier semantic failure. Qualifying the helper as `self::path` fixed it.
The final publication run was restarted from the beginning on frozen sources;
the failed development output is not spliced into its causal proof.

## Project control and INTERLOCK CHECK

M03 remains PARTIAL: this is a bounded qualified relation/path substrate, not
general causal understanding. S05 remains its existing bounded W20 scope, S12
PARTIAL, M06 OPEN. No new maturity row, program or model-state promotion.
INTERLOCK CHECK: no newly demonstrated cross-owner authority/execution seam or
registered producer contract pressure. The historical reader remains the source
qualifier, graph remains access and CLI remains presentation; no new Interlock.
I07 remains UNSELECTED; no next implementation wave is selected here.

External Golden remains FAIL/incomplete; governance live model-awareness remains
NOT QUALIFIED / externally blocked; HUMAN_GOLDEN_CASE PENDING_OPERATOR; continuity
canary NOT_RUN. No live request, fallback or retry occurs in this wave.

Transition v18, CaseState v15, S/W/compiler, Projection/ContextFrame, admission and
all canonical/operational owners are unchanged. Historical view v1 gains additive
scoped generic-resource PREPARE/indeterminate coverage; graph inspection v1 is new
and disposable. LMDB delta 0, current total 37/40. README and REPLAI unchanged.
ZERO-TO-CURRENT adds optional read-only experience inspection, not a delta manual.

Baseline unrelated changes in `docs/provider-governance.md`,
`tests/characterization/case-resource-access/test_reference_free.py` and
`refoundation/validation/external-golden-closure/` remain excluded. Only
milestone-owned cleanliness can be claimed; unrelated work must remain intact.
