# SEMANTIC.STATE.REFOUNDATION.0

Authority: bounded implementation/qualification evidence, not live project control.
Baseline: `c0d7e8d276e61d699daa76bb57e3f7b6c3cd90b8`, reconciled equal to
origin/master and remote master in the clean existing physical master worktree.
Intended commit: `feat: compile bounded semantic working state from Case owners`.
Implementation and deterministic/Golden-local qualification are complete.
Pre-publication state: isolated staged review, commit, push and remote equality
verification remain publication steps. The containing SHA belongs in the
post-publication handoff, not in this report.

## Archaeology and design constraints

At the baseline, `context.rs::compile_projection` combined owner extraction, disclosure,
historical limits and selection. `residency.rs` then independently selects that
Projection under invocation budgets; `provider.rs::compile_semantic_invocation`
coordinates both plus derived retrieval and rendering. Preserve existing typed
content, unresolved reviews/effects, exact invocation lineage and consolidation
isolation. The replacement must have one selection path upstream of lowering.

Direct read-only historical inspection, not recovery-ledger authority:

| yai-dev revision / source | Observed property | Current reuse / rejection |
|---|---|---|
| `6fa23af2592de5518dda7a2e59aae4cb2b82d89c`, `knowledge/context/request_contract.c` | explicit intent, scope and budget request; implicit session acquisition and default global/user Case | retain typed scoped request; reject session ownership and ambient fallback |
| same revision, `knowledge/workset/workset_engine.c`, retrieval/constraints/missingness functions | current anchors, bounded references, inclusion reasons and missingness | retain mandatory-current selection and inspectable omissions; reject summary JSON parsing, invented semantic/episodic refs, fixed salience and session-owned workset |
| `8a2b09e268fe6e20b1681dab7b22eac6b8239a8c`, `src/knowledge/retrieval/hybrid.c` | per-plane/total bounds and exact-reference deduplication | preserve bounded deterministic candidate selection; no plane/registry resurrection |
| same revision, `src/knowledge/memory/semantic.c` | mutable global concepts, time/salience ranking, byte-hash pseudo-vectors | reject as semantic authority; use current typed derived hierarchy and provenance |
| `085cf7ad242e11bde97c78f2cb7299c54b7a43c5` deletion/move history; current `5c1c7b9d0` substrate working delta/privacy contracts and knowledge-memory boundary test | working moved out of memory; later drained contract anchors export no execution API; boundary test proves placement/compilation, not a semantic compiler | no historical delta implementation claimed; recover distinctions only, not owners |

S is a read-only qualified composition of existing Case sources, not durable new
truth. W is a disposable scoped compilation with explicit intent/purpose,
source identity, relevance and bounds. Delta is derived change information,
never a mutation API; unsafe incremental classes explicitly recompile.
ContextFrame/Projection remain compatibility artifacts. No YVEX E, REPLAI/pin,
Agent, StateFabricStore or cross-repository executable contract is in scope.

## Implemented ownership and representation

Before: owner extraction and historical limits in Projection, then another
Residency selection, with runtime implicit index/encoder refresh in CLI plumbing.
After:

```text
existing canonical owners / immutable owned objects
  → replay-qualified SemanticState S
  → one bounded State Compiler
  → SemanticWorkingState W
  → context-compatible Projection / ContextFrame
  → unchanged exact cognitive / provider execution
```

`engine/yai-engine/src/semantic_state.rs` owns the read-only composition/compiler
seam, not a durable semantic owner. S wraps exact CaseState/history only after
`CaseState == replay(history)`; it cannot be deserialized as authority or mutated
through this API. Immutable bytes remain with Content/Artifact owners. W/delta
are disposable derived values. No Agent, StateFabricStore, generic mutation API,
shared provider state or new LMDB database exists.

The single typed `SemanticEntry`/`SemanticValue` family is moved upstream from
context, not copied into a competing ontology. It distinguishes current lifecycle,
Tenant/Participant/control and policy bindings; admitted content/ordered Turns;
resource observations and effect consequences; derived operational/episodic/
semantic material with its precise epistemic posture; provider claims; pending
review/unresolved effects. Provenance kinds remain ContentObject, Transition,
Observation, EffectReceipt, CaseStateGeneration and DerivedMemory. A grounded
derived assertion is still derived, and a required provider claim is still a claim.

Policy binding references are mandatory current control, not materialized policy
rules or a new evaluator. EffectivePolicy, expiry/revoke, Decisions, review,
Grants, resource fences, Workflow adoption and provider dispatch remain with
their existing owners. Semantic requirements need exact typed sources; this
wave does not create generic objectives/obligations or infer policy from prose.

## W contract, locality and identity

`CompilationRequest` binds Participant/view/purpose, intent, output-contract ID,
current source snapshot, exact required refs, resource relevance, previous-item
preferences and explicit item/semantic-unit/derived-candidate limits. The existing
governed selection can prove a model view only for its exact current binding,
Case/Tenant and Participant. I06 delegation resolves the canonical Turn intent
and current Principal link; it does not disclose the author's unrelated Turns.

Qualification precedes relevance. Required IDs resolve only among qualified
sources; missing and hidden refs share `semantic_required_source_unavailable`.
Current control/unresolved sources and exact requirements survive selection or
the whole compilation refuses for insufficient budget. Optional recent content,
observations, Turns/results and intent/resource-matched derived material use
deterministic bounds. Relevance is lexical/exact-source, not learned, model-name
ranking or a fake capability score. Source IDs break ties deterministically.

S identity hashes representation version, complete history and materialization.
W identity hashes compiler/representation versions, S identity, exact request,
Case/generation/Participant, selected typed entries, bounds and explanations.
Changing source, scope, request or versions invalidates W. `lower_context`
fully recompiles to validate equality, so a forged serialized artifact cannot
reach normal provider rendering as current. Provider-labelled residency reports
have distinct IDs even when W and selected values are identical.

Selected and budget-omitted entries have per-entry reasons. Earlier locality
omissions have bounded aggregate reasons/counts, not hidden source IDs or an
unbounded omitted-history transcript. Units use the existing serialized-character
estimate, not model tokens. The bound covers selected semantic payload; source
reconstruction cost, request size and diagnostic metadata are not claimed to be
constant with arbitrary Case history. Full replay and reconstruction currently
scan history, and lowering's defensive recompile repeats this work.

Canonical operational memory and hierarchy are reconstructed before ranking;
current lifecycle/disclosure/resource visibility and assertion support closure
are rechecked. No graph/index bytes, stale vectors or encoder availability can
alter W. Unresolvable derived support is omitted, never promoted. Unsupported
source reconstruction errors refuse honestly.

Locality oracle, `semantic-state-compiler-final-20260909`, order 2, actual output:

```text
locality case=case:young transitions=8 selected=6 semantic_units=1056 omitted=3 required_retained=true
locality case=case:old transitions=1206 selected=6 semantic_units=1054 omitted=2399 required_retained=true
```

The old Case retains an exact older constraint absent from its recent conversation.
Task switching preserves that requirement without upgrading immutable application
content to policy. A separate real PolicyArtifact/binding test retains current
Case policy through 40 irrelevant Turns/task changes, then replaces and unbinds
it; the old reference refuses and removal/full-delta results agree. These are
bounded positive/negative locality oracles, not universal task sufficiency or
Case-age-independent compilation CPU. No fixture size becomes a production limit.

## Derived delta and restart

`SemanticDelta` binds exact same-Case forward history, source/destination IDs and
generations, request identity and typed Added/Replaced/Removed entry digests.
Digests cover values, posture and provenance. Supersession/invalidation appears
through current-owner replacement/removal; unresolved/control changes preserve
their typed source meaning. It is not a self-contained payload transport and
cannot apply a canonical mutation.

`compile_delta(old_W, old_S, new_S, request, delta)` validates old W and rederives
the exact delta before application. **All supported classes currently return
`FullRecompilation`**. This is an explicit safe fallback, not a claim that a
cache is incrementally updated faster. Both qualified source snapshots are
required. The exact equality oracle compares the whole W, including ID, entries,
request, provenance and explanations, not only rendered strings. Stale/tampered
delta, altered request, backwards/nonextension history and old-W/current-source
lowering fail. Serialization/replay restart reconstructs the same qualified result.

No provider continuation participates in delta identity. Computational E updates
remain outside YAI and cannot produce semantic authority through this interface.

## Compatibility, consolidation and versions

Normal conversation, Case runtime, free Golden and Workflow use the existing
`compile_semantic_invocation` seam consuming W. The shared semantic budget kernel
in residency is called once for selection; ResidencyPlan becomes a compatibility
inspection report. Projection/ContextFrame carry selected meaning/lineage and
render to the existing adapter. Exact targets, lanes, suitability, mechanical
shape evidence, composition/source closure and delivery safety are unchanged.

Invocation → Projection → `bounds.working_state_id` exposes W using existing
lineage. `context inspect --id <working-state-id>` recompiles that historical
generation; this is forensic reproducibility, not authorization to dispatch it
as current. W can be cached in the existing semantic-context artifact database.
No persisted compilation cache is needed for correctness.

The old `compile_projection` facade remains for engine inspection/compatibility
tests, sharing the same typed source extractor. No normal provider path calls it.
The legacy Journal diagnostic plane is not canonical and is not refounded here.
No source module was deleted wholesale: approximately 880 lines of types and
extraction leave `context.rs`, moving to the semantic composition seam; 121 lines
of implicit runtime hybrid retrieval/index refresh are deleted from `memory_cli`.
Explicit W19/H19/W20 index build/search/rebuild remains product-reachable.
This is ownership consolidation plus new compiler/delta/tests, not a claim of
net source-line reduction.

| Contract | Delta |
|---|---|
| Transition / CaseState | v18 / v15 unchanged; no payload/field migration |
| S identity / W / State Compiler / SemanticDelta | v1 derived contracts; S is not a persisted schema owner |
| Projection / ContextFrame | v8 → v9: W identity/selection lineage is explicit |
| RenderedInput | v7 unchanged; same generic adapter wire behavior |
| Semantic/operational durable owners | +0 / +0 |
| LMDB | +0, 37/40; reuse droppable semantic-context artifacts |
| Derived assertion schema/valid identities | unchanged; empty provider results no longer attempt to manufacture invalid empty assertions |
| REPLAI / YVEX / root README | unchanged |

## Qualification and coverage reconciliation

Exact commands, absolute cwd, environment, baseline SHA, run IDs/order, material
pre-state, actual exit and bounded unedited stdout/stderr are retained in
[qualification.jsonl](qualification.jsonl), [development.jsonl](development.jsonl)
and [external.jsonl](external.jsonl). Truncation is explicitly marked; nested
Golden logs preserve their own home/run/order and PTY bytes. Different runs are
not combined into one causal transcript. The baseline SHA labels the tested
worktree; the final commit containing these changes is reported after publication.

All commands ran from `/home/mothx/computer-science/projects/YAI/yai`, with
`TMPDIR=/tmp`, `CARGO_NET_OFFLINE=true`, `CARGO_TARGET_DIR=target`; terminal lanes
used `build/terminal-tests/bin` first on PATH with repository-pinned dependencies.
No live provider is needed by the deterministic union. An unrelated unavailable
mount broke the shell sandbox late in qualification; explicitly authorized
repository/test commands were used without changing that mount or host services.

| Run / order | Exact inner command | Proof / provider mode | Exit / observed result |
|---|---|---|---|
| semantic-state-publication-20260909 / 3 | `make check characterization` | full existing unit/component/contract/product/recovery/publication-endurance union; no_provider + loopback_fixture as classified | 0, PASS, 292.949s |
| semantic-state-compiler-final-20260909 / 2 | `cargo test --manifest-path engine/Cargo.toml semantic_state::tests:: -- --nocapture` | 8 unit/recovery contracts, no_provider | 0, PASS, 1.111s outer command |
| semantic-state-golden-local-20260909 / 4 | `make test-golden-local` | complete free + Workflow product/recovery, loopback_fixture model, real resource transports | 0, PASS, 42.195s |
| semantic-state-golden-local-20260909 / 5 | `make smoke-case-reference-free` | standalone free product/recovery, loopback_fixture | 0, PASS, 18.721s; exact W retained in bounded final summary |
| semantic-state-external-golden-20260909 / 1 | `make test-golden-external-yvex` | external qualification, external_yvex | 2, FAIL, 136.035s; real 413, no fallback |
| semantic-state-documentation-final-20260909 / 1 | `make check-docs check-layout test-roadmap test-topology check-validation-topology` | documentation/validation infrastructure, no_provider | 0, PASS, 1.474s; final COMPLETE/UNSELECTED row and exact catalog |

The classified ninth new Rust assertion checks empty/whitespace ProviderResult
lineage without invented semantic assertions. Eight new compiler assertions
cover materialization drift, locality/task switch, disclosure, required budget,
tamper, exact delegation, model-independent lowering, delta and policy replacement.
Topology audit: 483 entries, 90 assertion files, 381 registered Rust tests;
71 historical smoke leaves remain reachable through 86 current Make leaves,
with **zero duplicate Make leaves** in the combined publication invocation.
Characterization reports `Nothing to be done` after the shared union. Individual
historical scripts intentionally retain their own diagnostic Rust repetitions.
No old-versus-new throughput benchmark or general speedup is claimed.

Migration failures are retained, not relabeled PASS:

- Initial locality selected irrelevant zero-match derived candidates; corrected
  by exact/intent-qualified optional selection, keeping required/control sources.
- Golden exposed I06 delegated author→model Turn visibility; corrected through
  the existing exact authorization helper, not ambient cross-Participant access.
- Three deterministic peers accepted only ContextFrame ≤v8; v9 is now explicitly
  accepted, without relaxing their payload/operation checks.
- W19/W20 assertions for implicit runtime retrieval IDs were intentionally retired
  with that algorithm. Explicit index/search/encoder qualifications remain; new
  checks require no implicit encoder and identical canonical W after index drop.
- Publication order 1 detected lost precision in observed-consequence memory
  posture. Specific operational postures are preserved, not collapsed into a
  generic derived class.
- Publication order 2 detected an empty I03 result poisoning hierarchy rebuild.
  The existing assertion producer now keeps raw execution lineage but emits no
  assertion for empty material. I03 malformed/empty/indeterminate/recovery checks
  pass unchanged in order 3.
- Terminal dependencies were absent initially; installed per the existing
  isolated test contract, with no REPLAI source/pin changes.

## Golden product lineage and preservation

Free and Workflow use the actual `./yai` REPLAI product, real LMDB and filesystem,
SQLite, constrained process, HTTP and MCP reference peers. The deterministic
model drives ALLOW/DENY/REQUIRE_REVIEW, human-role review, source repair, real
failing/passing tests, retry/restart, Workflow PlanPatch validation/adoption,
resource/Participant/Tenant isolation and explicit Handoff without authority
transfer. `/verify` proves current CaseState equals canonical replay. `/rebuild`
and explicit index drop/rebuild do not alter history or exact historical W.

These independently recorded final examples must not be merged as one run:

| Mode / nested run | Invocation | W | Selected / omitted / units |
|---|---|---|---|
| free / `yai-golden-free-31iirfek`, order 164 | `invocation:case:golden:free:model-prompt-44` | `working-state:sha256:40bdc1c35c9ba578271bfef82da19b416186d3f78c6781989cb68d67f4089cca` | 43 / 76 / 10230 |
| Workflow / `yai-golden-free-ivqw_tge`, order 148 | `invocation:case:golden:workflow:model-prompt-53` | `working-state:sha256:ca90b3a34be45b610ab68e146fcc0c8ea7712072e5dba6a6c3d198e2c99d25c2` | 46 / 93 / 11421 |

Their final summaries retain exact Turn/review/source identities, observed
15/18 model dispatches and `working_recompiled_equal_after_rebuild=true`.
Replay and W reconstruction execute after actual process restart. These bounds
are examples, not a claim that free and Workflow have equivalent task states.

## YVEX EXTERNAL FINDINGS

**GENERIC_PROVIDER_CONTRACT_GAP — external Golden FAIL, not deployment absence.**
Operator-supplied endpoint `http://127.0.0.1:18001`, exact exposed model
`deepseek-v4-flash-mixed-mxfp4-release-v1`, public profile
`yvex.openai.compat.v2`. No YVEX commit/ref was exposed or supplied for this run.
Real catalog and synthetic text/functions/JSON probes succeeded; real Golden work
then returned HTTP 413 `request_too_large` with 40,438 request bytes written.
The retained `/details` reports `DeliveryIndeterminate`, not a successful result
or safe cross-target retry. The test stops; external Workflow is not reached.

Run: `semantic-state-external-golden-20260909` order 1, nested
`yai-golden-free-tjifom6r`; canonical Turn
`conversation-turn:sha256:b8211ff568794f986dd32f69937a4f34057cffb23831ca91d275b465991920b2`.
Environment additionally sets `YAI_EXTERNAL_PROVIDER_BASE_URL` to that endpoint,
`YAI_EXTERNAL_PROVIDER_MODEL` to that model and locality `loopback` (operator
tunnel address, **not** fixture provider mode). Real requests executed. No private
protocol, producer source inspection, model administration or brand-specific
compensation was used. Exact server-side limit/cause is unknown; probe success
does not qualify Case capacity. The external run predates the final internal
posture/empty-assertion corrections and is not presented as a later rerun.

DeepSeek full Golden remains unqualified. Qwen/external cold substitution was
not run. Semantic context compatibility does not promise fitting every provider
limit, nor instant inference/probing. Future work needs truthful public capacity
and state-capability contracts rather than silently discarding required material.

## Product acceptance, roadmap and remaining pressure

`GOLDEN CASE LOCAL = PASS`; `GOLDEN CASE EXTERNAL YVEX = FAIL` as above.
`CONTINUITY CANARY = NOT_RUN`: no private operator home was reset or copied.
`HUMAN_GOLDEN_CASE = PENDING_OPERATOR`; automated PTYs cannot confer human PASS.
[`ZERO-TO-CURRENT`](../../../docs/zero-to-current.md) is **unchanged** because
setup, product vocabulary, free/Workflow/review/retry lifecycle and operator
authority are unchanged. Internal W inspection is optional plumbing, not a new
human workflow. No separate delta runbook is introduced.

Roadmap C01/C03/C04/C05 establish the explicitly bounded selection/composition/W/
delta-fallback properties; C06 and S09 become PARTIAL for measured locality and
task/Case control retention. Counts derive only from rows: 32 ESTABLISHED,
23 PARTIAL, 9 OPEN, 4 LATER (68 total). R/K/S/C/M/Q advance; A/E/O/W/X remain
load-bearing consumers. The one control row closes COMPLETE with the next
implementation boundary explicitly UNSELECTED; a small guard regression prevents
completion from silently selecting another wave. I07 stays UNSELECTED.

Remaining before a first experiential consumer: public W→E capability/identity/
invalidation contract, actual target and independent state-read oracle; precise
input-capacity qualification; qualified source expansion and task sufficiency;
reconstruction-cost work and proven incremental optimization if needed; broader
locality/cold-model evaluation; real external Golden, human acceptance and retained
canary evidence. State Read/Update, paging, model-native representations, training
and adapters are not implemented or authorized by this closure.
