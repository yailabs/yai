# HISTORICAL.SEMANTIC.RECONSTRUCTION.0

Baseline: `a08ede230705e3d68e69fb44dfb7eb73c2860f2b` on the existing physical
`master`; HEAD/origin/master/remote master reconciled before mutation.
Intended isolated commit: `feat: reconstruct scoped historical Case semantics`.
Pre-publication state: implementation, targeted contracts, deterministic
publication/characterization union and Golden local free/Workflow **PASS**.
Ready for isolated commit/publication; actual commit SHA and remote equality
belong to the post-commit handoff, not this self-containing report.

## Contract and ownership

`semantic_state::historical` is a derived reader within the existing semantic
composition owner, not a canonical owner or persisted snapshot. One LMDB read
transaction qualifies current Tenant membership, exact Principal/Participant
association, current view admission, Case/history and exact policy backing.
The public product is `yai case as-of CASE GENERATION_OR_TRANSITION`. A sole
current linked Participant is resolved automatically; `--participant` cannot
impersonate another Participant, even for a Tenant owner. Normal operator
inspection uses that existing current association, not a new audit authority.
Explicit model-context inspection additionally requires current admitted view.

`HistoricalSemanticView v1` separates prefix materialization, recorded evidence,
historical bound-artifact semantics, current normative meaning, original-source
closure and historical/current comparison. It cannot be consumed as W, a Grant
or an admission command. No provider inference, external effect or Transition
append is part of reconstruction. Current S/W/compiler v2, semantic delta v1,
Projection/ContextFrame v10, rendered input v7, Transition v18 and CaseState v15
are unchanged. Semantic/operational owner delta **0**; LMDB delta **0**, **37/40**.

## Coordinates, time and epistemic limits

Generation is the exact contiguous per-Case Transition sequence; a Transition ID
resolves to the same generation and identity. Zero, future, foreign/unknown IDs
and unsupported wall-clock input refuse. Recording time is exposed only from
the recorded Transition; observation time only from ResourceObservation. No
generic event-time producer exists, so occurrence time is explicitly absent.
An arbitrary timestamp inside observation JSON remains reported data, not a
new event-time authority. Late evidence is known at its recording generation,
never retroactively inserted at the earlier occurrence it describes.

Prefix replay preserves own Participant roles, active PolicyBindings, currently
disclosed resource envelopes and own review/Grant/effect materializations.
Typed evidence covers own operations, exact Decisions/DecisionBasis, reviews,
Grants, filesystem effect history, resource observations, admitted content,
ConversationTurns and provider claims. Claims remain claims, PREPARE remains
unresolved and a receipt remains consequence evidence. Workflow/Handoff detail,
process-effect detail and derived W20 Episode/Assertion reconstruction are
explicitly outside this first profile; no blanket historical-family claim.

Current disclosure is checked before exporting historical values. Resources must
still match the exact current envelope and Participant visibility. Shared paths
or endpoint/model identities are never access evidence. Existing writers do not
support resource detach/rebind or Participant view revocation; this wave does
not invent them. Unknown/current scope refuses, changing current disclosure
changes the view identity, and a same-Tenant second Participant sees neither
another's historical observation nor its private resource identity.

The historical Case prefix and independent governance catalog do **not** provide
a universal common temporal cut. Exact bound immutable artifacts and publication
anchors reconstruct the applicable normative composition/EffectivePolicy ID.
They do not prove every global lifecycle fact at an arbitrary Case generation.
The view says `global_catalog_cut_unavailable`; exact validity at an actual
Decision remains in its committed DecisionBasis, including authority time and
policy validity contracts. Current revoke/catalog posture is shown separately.
Current sampled wall-clock values are not represented as historical timestamps
or incorporated into reproducible semantic identity; actual validity posture,
revocation identity and committed authority floor are retained.

Original policy sources and admitted content are resolved by exact identities.
Unavailable original bytes do not erase an intact artifact/recorded DecisionBasis
or get replaced by current mutable files. Missing artifact/publication closure
is explicit and no partial artifact set masquerades as complete policy meaning.
This includes a bound-then-unbound policy with **no Decision**: the historical
reference still requires source closure. The dedicated negative retains that
reference after original/artifact loss and blocks incomplete old composition.

Comparison is derived: unchanged current materialization, still-recorded evidence
(explicitly **not current authority**), superseded policy lineage, removed state,
changed state and items not yet present at the coordinate. It is not SemanticDelta
or a mutation API. Identity binds representation, normalized coordinate, complete
source history, current scope and current normative meaning plus source closure.

## Archaeology and recovered properties

- `yai-dev` at `45c36bd0c`, `lib/knowledge/episodic/episodic.c` and its public
  header: fixed 256-entry process-global array, append/latest/reset; this is not
  historical semantic reconstruction. No global array, graph-as-memory owner or
  summary-as-truth is recovered. The later directory cutover is visible at
  `1c934d3a5`; it is not evidence of a removed general as-of implementation.
- Same epoch, OP-4 `docs/architecture/peer-conflict-ordering-replay-model.md`
  and `tests/integration/qualification/lan/ql_lan_peer_offline_replay_v1.sh`:
  late observation/receive ordering and explicit replay posture are useful
  distinctions, not a causal clock or a Case historical query. Preserve their
  missingness/ordering discipline, reject historical Workspace/peer registries.
- Current Transition reducer and Wave 9–11 immutable binding, historical
  DecisionBasis and replay oracles are the stronger executable mechanisms.
  Reuse the reducer, exact artifact matcher/publication anchors and pure policy
  materializer; never call current-catalog composition historical authority.
- W20 Episodes/Assertions remain derived. Their caches are not historical truth;
  broad temporal reconstruction of those families is not promoted here.
- No YVEX internals were inspected or administered. Public-roadmap URL reads
  returned 404; no local YVEX roadmap was found. The supplied producer-performance
  posture and existing published YAI qualification evidence remain authoritative
  for this separation. No live endpoint or generation retry was attempted.

## Qualification

Exact commands, run IDs, material pre-state, unedited output and exit status are
retained in [contracts-r2.log](contracts-r2.log) and
[publication-r2.jsonl](publication-r2.jsonl). [Source identities](source-sha256.txt)
bind the implementation/test whitelist used by both final runs. Baseline SHA is
the parent; these runs qualify the then-uncommitted source identities. The earlier
r1 development gates also passed, but are not used as final evidence after the
unbound-policy source-closure hardening. No outputs from different runs are
combined into one causal trace.

Publication output is losslessly line-encoded as `combined_output_line`, including
the original trailing spaces. `jq -r '.combined_output_line' publication-r2.jsonl`
reconstructs the original combined stdout/stderr byte-for-byte; `cmp` verified it
against the raw capture before archival. Evidence was not whitespace-normalized
to satisfy `git diff --check`.

Working directory for all commands:
`/home/mothx/computer-science/projects/YAI/yai`.

| Run / order | Exact invocation and material pre-state | Proof / provider | Actual result |
|---|---|---|---|
| historical-contracts-r2 / 3 | `TMPDIR=/tmp CARGO_TARGET_DIR=target cargo test --manifest-path engine/Cargo.toml historical_ -- --nocapture --test-threads=1`; fresh disposable Case/store per test | Contract/recovery, no_provider | Exit 0; 15 matching tests (7 new, 8 existing historical compatibility/recovery controls), no failures |
| historical-publication-r2 / 4 | Activate `/tmp/yai-governance-qa-venv.Jf2vcj/bin/activate`, then `env -u YAI_EXTERNAL_PROVIDER_BASE_URL -u YAI_EXTERNAL_PROVIDER_MODEL -u YAI_EXTERNAL_PROVIDER_LOCALITY -u YAI_HOME TMPDIR=/tmp make check characterization test-golden-local`; existing build cache, suite-owned fresh homes/resources | TEST.TOPOLOGY unit/component/contract/product/recovery/bounded-endurance union; no_provider and loopback_fixture only | Exit 0; documentation/layout/topology, deterministic regression and both Golden local modes PASS |
| historical-docs-r1 / 5 | Same directory/QA environment, `TMPDIR=/tmp make check-docs check-layout test-roadmap test-topology`, `python3 tools/checks/check-roadmap.py --summary`, `python3 tools/validation/topology.py audit --static`, `git diff --check`, `sha256sum -c refoundation/validation/historical-semantic-reconstruction/source-sha256.txt`; final qualified sources and closure wording | Documentation/layout/topology and identity verification, no_provider | Exit 0; [unedited output](docs-r1.log), generated counts agree and every qualified source hash matches |

The product subrun `yai-historical-product-95hqu5cx` in publication-r2 invokes
actual `./yai`: init/create/link, immutable intake/publication, binding and
replacement, revoke, as-of inspection, invalid coordinates/identity/budgets,
fresh-process reinspection, policy rebuild and replay. Its exact commands,
individual exit codes and stdout/stderr precede this unedited summary:

```text
historical_product=PASS original_policy_preserved=true current_revocation=true no_inference=true queries_append=0 rebuild_equal=true
```

Engine contracts and product commands are separate proof classes; neither
historical qualification uses a provider.
The historical policy oracle adds an intermediate review version so one Case
contains ALLOW, late observation, REQUIRE_REVIEW/eligible approval, replacement
DENY, global revoke and Case unbind. No authority behavior was weakened: missing
explicit reviewer eligibility initially produced the correct DENY in development.

Bounded unedited output from the **single** policy test in contracts-r2:

```text
test store::lmdb::tests::resource_access_tests::historical_tests::historical_policy_chronology_late_observation_and_current_permission ... historical_decisions p1_artifact=policy-artifact:de79f810069cb33ed2b18efa24a317dcd1f8098385e071d31c16623180625301 d1=decision:435ec9e02d9d24b3fc45e66fb45f95d8 basis1=decision-basis:ef92bf3bfe037d49f492158b3566e005 effective1=effective-policy:fe3cdf04d62e6cdf8af30d313834eeb2 d2=decision:b654413a930d6797976a79d7b2ca25ea basis2=decision-basis:679eaa6913c3721188f06f9117dee8d1 effective2=effective-policy:858eaf2db0403da0c07543721f28c59e review_action=review-action:sha256:48d0ff227d3146b9972e14e66 historical_view=historical-view:sha256:50e0d050abb05812fe2beba304990f5185ec54be273c309d41295a5fdc167009
historical_policy case=case:resource-contract before=7 p1=8 d1=11 late=12 review_policy=13 review=review:d6ce9cec9440100d5ebc709a51c9598f p2=19 d2=21 unbound=22 exact_basis=true old_allow_not_permission=true model_calls=0
```

Additional independent controls in that run prove same-Tenant/different-current-
Participant isolation, unavailable disclosure refusal, view identity changes,
invalid/future coordinates, Transition/generation alias equality, provider-claim
epistemics, immutable admitted content despite mutable resource drift, explicit
original/artifact loss, full store close/reopen, graph/memory/context/policy-cache
drop/rebuild and `CaseState == replay(history)`. Historical queries append **0**
Transitions. Scope/missing-source tests compare the canonical pre/post history,
not merely successful command exit codes.

Reconstruction loads N canonical Transitions, replays the full N for current
materialization verification and two prefixes of length G (artifact reconstruction
and typed view composition). The reported reducer count is `N + 2G`, not a CPU
complexity guarantee. Several evidence/provenance passes also run. Output is
bounded by explicit item/serialized-byte budgets with refusal, not silent
truncation. A short and a 51-times-longer Case characterize cost; no
Case-age-independent CPU or universal small-history requirement is claimed.

Measurements from the single short/long cost test in **contracts-r2**, debug
build on this host; informational observations, not thresholds or benchmarks:

| History N | Coordinate G | Unique Transitions loaded | Reducer visits N + 2G | Output items | Reconstruction µs |
|---:|---:|---:|---:|---:|---:|
| 8 | 1 | 8 | 10 | 7 | 9,569 |
| 8 | 8 | 8 | 24 | 12 | 14,776 |
| 8 | 8 (repeat) | 8 | 24 | 12 | 14,718 |
| 408 | 1 | 408 | 410 | 7 | 29,982 |
| 408 | 8 | 408 | 424 | 12 | 35,585 |
| 408 | 408 | 408 | 1,224 | 12 | 45,119 |

Golden local uses distinct product runs in publication-r2, not the historical
oracle's Case or a live provider:

| Product run | Final checkpoint | Provider | Result |
|---|---|---|---|
| yai-golden-free-n0o58e_2 | order 164, golden_free, 15 model dispatches | loopback_fixture | PASS; review/restart/replay, no retry duplicates, W equal after derived rebuild |
| yai-golden-free-dbnrlb_w | order 148, golden_workflow, 18 model dispatches | loopback_fixture | PASS; review/restart/replay, no retry duplicates, W equal after derived rebuild |

Both record `external_request_attempted=false`. Their local success is not live
model-awareness evidence. Prior S/W locality, disclosure, delta/full equivalence,
typed admission and recovery contracts are preserved in the publication union.

## Project control and INTERLOCK CHECK

GOVERNANCE.COGNITIVE.CONTEXT.0's YAI implementation/local qualification is earned.
Its real model awareness remains **NOT QUALIFIED / externally blocked**: no
ProviderResult within 300 s despite capacity-compatible 4,448/16,384 input/sequence.
It is not retried here. Full External Golden remains **FAIL/incomplete**;
HUMAN_GOLDEN_CASE **PENDING_OPERATOR**; continuity canary **NOT_RUN**.

S12 advances only to PARTIAL for this exact bounded prefix profile. No new
maturity row/program, no M06 Recall promotion, no model-state promotion.
Counts remain row counts: ESTABLISHED 32, PARTIAL 24, OPEN 11, LATER 4, total 71.
K/S/M/Q program foundations now cite bounded historical backing; K04/M03/Q06
are not promoted to broader historical, causal or evaluation capabilities.
INTERLOCK CHECK: the historical/current reader composes established owners;
independent catalog missingness is exposed, not bypassed. No new cross-owner
mutation/authority seam requires selection. **I07 UNSELECTED**. No next
implementation boundary is selected by this closure.

README, REPLAI, YVEX and unrelated concurrent work are preserved. The cumulative
ZERO-TO-CURRENT adds an optional as-of inspection checkpoint; its normal model,
resource, review and Workflow lifecycle does not change.

**YVEX EXTERNAL FINDINGS:** no new live finding; no endpoint/generation retry,
fallback or producer administration. This no-provider semantic boundary does
not depend on producer prefill performance. The previous external failure is
retained, not reclassified as PASS or a completed Human Golden.

The baseline already contained unrelated changes in `docs/provider-governance.md`,
`tests/characterization/case-resource-access/test_reference_free.py` and untracked
`refoundation/validation/external-golden-closure/`. They are deliberately excluded
from this commit. The test-file change is opt-in run retention, inactive in these
ordinary local runs; it changes neither their request nor semantic assertions.
Their preservation means a globally clean worktree cannot be claimed; publication
cleans this milestone's own changes only.
