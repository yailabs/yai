# Qualified knowledge and experience Recall

Recall reconstructs a bounded, query-conditioned view across admitted source
knowledge D, Case history H and the current-at-cut semantic foundations S. It
does not assemble a prompt, invoke a model, change W or append a Transition.
Maturity and selection remain exclusively in [ROADMAP](../ROADMAP.md).

## Typed and product contract

```sh
./yai case recall CASE 'billing retention'
./yai case recall CASE 'billing retention' --ref EXACT_ID --json
./yai case recall CASE 'billing retention' --at GENERATION_OR_TRANSITION --json
```

The native CLI now constructs `RecallRequest::integrated` (`yai.recall_request.v2`)
and consumes `yai.recall_trace.v2` from the existing authorized Recall entrypoint.
`RecallRequest::new` and explicit v1 requests retain the independently qualified
H/S-only contract and v1 serialization, with no documentary field. V2 adds one
typed documentary family, not a second Recall API or knowledge-search workflow.
Older serialized diagnostic counters remain readable; absent additive v2
measurements default to zero and are never used as semantic evidence.
No canonical schema, storage owner or persistent Recall/knowledge cache is added.
Future clients consume the same typed result below CLI formatting.

The request binds current Case generation, linked Participant, normalized query,
exact historical cut, mandatory refs, bounds and version. Equivalent Transition
and generation cuts normalize identically. Unknown and hidden refs share one
refusal. Exact source/revision/unit/entity anchors cannot be replaced by similar
text; a known revision with missing backing remains an incomplete source result.

## One qualified read basis

One LMDB read transaction supplies historical reconstruction and authorized D.
Canonical history and its cut materialization are reused; D does not independently
replay the Case. Current source permissions reuse one materialized normative
status and the loaded history, including exact existing provenance obligations.
Discovery, relations, closure and final assembly consume this immutable qualified
input, never another live source read or cached index assertion.

Current Tenant Owner, declaring Principal, linked requesting Participant,
knowledge role, non-revoked acquired source, resource envelope and current
EffectivePolicy/content-read permission are required by the bounded D profile.
This is not a new general audit authority or a broader M07 sharing contract.
An authorized non-Owner retains the existing H/S inspection surface with an
empty D family; adding D does not impose M07's Owner gate on all Recall.
Source-owned acquisition contents/observations are also filtered before H/W20
discovery: source revocation cannot be bypassed by asking historical Recall.
Independent operational evidence retains its own historical disclosure contract.

Default queries discover the applicable revision at the requested cut. Old
documentary revisions enter only through explicit historical/exact requests;
opaque old unit IDs require bounded retained-source derivation to resolve.
A revision first admitted after the cut cannot enter D at that cut. CURRENT
disclosure still applies to old revisions. Original loss never triggers live-file
substitution. Generic occurrence-time source queries and snapshotless historical
backing are not qualified by this profile.

## Discovery is not semantic resolution

H/S BM25 and optional qualified pre-encoded vectors retain their existing role.
D uses M07's exact units and the existing BM25 implementation. Corpus statistics
remain family-local and already disclosed. Stable round-robin family admission
starts with H/S, so document volume cannot occupy every candidate slot; a single
candidate slot can cover only one family. Scores across families are not compared
as if they established truth.

Required anchors resolve before ranked candidates. A candidate plus source,
parent structure, required contradiction/supersession context and qualified
expansion is an atomic group. Current policy lineage is reserved inside a group
before optional documentary material is admitted. Output pressure removes whole
optional groups in reverse admission order; it never silently trims a required
anchor or a member of mandatory disagreement context. If mandatory material
alone exceeds the profile, the request refuses.

V2's `events` bound is the combined selected-item bound: historical events,
documentary units and source descriptors. Relations and segments also share
aggregate bounds across families. Existing defaults remain 16 candidates,
64 items, 128 relations, 16 segments, 4 expansion steps per documentary/experience
phase, 16,384 serialized-character units and 1 MiB. Units are not model tokens.
Mandatory backing/conflict closure is not rank-truncated. Input preparation retains
the bounded historical and M07 profiles; oversized source universes refuse rather
than silently claiming complete Recall. Host work is not Case-age independent.

## Trace meaning and exact backing

Historical events, W20 assertions, their epistemic classes, lifecycle posture,
recording/observation coordinates and typed experience relations remain intact.
Documentary segments retain source revision, captured backing, parser/profile,
location, admission Transition/generation, current-revision and cut applicability,
typed units, selection reasons and unresolved disagreements. Document structure
order is not Case chronology. Documentary admission time is not event time.

`source_stated`, `deterministic_structure` and `recorded_observation` are not
flattened into facts. A policy document remains documentary evidence; only the
independent PolicyBinding/EffectivePolicy/DecisionBasis path is normative.
Different documentary values do not choose a winner by recency, repetition or
rank. Historical/current revision posture is separate from disagreement.

Existing knowledge and experience graph relations keep their exact kind/backing.
An explicit documentary reference may resolve an already disclosed historical
object. Its cross-reference is **source-stated reference, not causality or
normative support**. Shared Resource identity is a subject association, not proof
that all operations over that Resource explain a claim. Disconnected segments
are legitimate; similarity and adjacency never create edges.

Two distinct content admissions may reuse one immutable payload. The experience
graph preserves both event identities; a shared payload ID alone cannot select
one causal/support event. Exact Transition anchors disambiguate. This corrects
whole-graph refusal on physically deduplicated sources without choosing the first
or latest admission as causal truth.

Trace identity binds version, normalized request, disclosed qualified inputs,
selected semantics, availability, budget and vector evidence. Private history
hashes and hidden candidates are excluded. Timing is outside identity. Rebuilds
reproduce the same qualified trace; current generation, disclosure, exact backing
or revision change reidentifies or refuses stale material.

## Executable qualification

`make smoke-knowledge-recall` exercises the actual product, real temporary LMDB,
shared source bootstrap, Markdown/JSON/PDF/tree/SQLite D, current/as-of revisions,
policy dual role, content loss despite an extant live file, revocation, exact
Decision anchors, family/output pressure, restart and disposable-access rebuild.
It compares short/long histories with small/larger admitted source sets and
prints separate source-resolution, derivation, discovery and assembly evidence.
The qualified-read total includes historical resolution and D preparation;
source resolution, extraction and graph derivation are separate D measurements.
Qualification measures W20/group resolution; assembly includes all output-fitting
attempts, while the closure counter measures the accepted final closure pass.
These nested measurements must not be added as if they were disjoint phases.
The resolver's independent exact-reference counterfactual and mandatory conflict
closure tests live beside its implementation. Existing Recall v1, M07 and Golden
local gates remain independent regression requirements.

The product counterfactual removes an exact documentary Decision reference and
checks that its edge disappears while the mandatory Decision remains. Its SQLite
schema change is an explicitly external fixture intervention, followed by a real
governed YAI observation; it is not reported as a YAI corrective effect or as
proof that the document caused the change.

Legacy archaeology: `yai-dev` commit
`8a2b09e268fe6e20b1681dab7b22eac6b8239a8c`,
`src/knowledge/retrieval/hybrid.c` and `src/knowledge/recall/rank.c`, supplied
bounded per-plane candidates, exact anchors and inclusion-reason pressure.
Those properties belong in the existing qualified Recall resolver. Historical
activation/pin/recency/hotness ranking does not establish validity, disclosure or
causality; neither that ownership tree nor its repeat-count preferences are
restored. The later substrate-drain epoch
`2a4018147219044dfe1fad2268759b1f2a585945` and current W19/H19/W20 contracts remain
the ownership boundary, not a reason to recreate a legacy knowledge plane.

No universal task sufficiency, arbitrary documentary interpretation, learned
navigation, all-owner S/Workflow Recall, general event-time algebra or model-native
state is claimed. Independent Recall inspection does not change W. The execution
compiler below is a separate consumer of this qualified result.

## Recall-aware working-state compilation

`case context compile CASE INTENT` is a read-only native application operation,
not provider execution. It automatically obtains Recall v2 and compiles W v3.
Its typed owner is `LmdbRecordStore::compile_working_state_authorized`, returning
`QualifiedWorkingState`; CLI formatting owns neither discovery nor selection.

```text
WorkingStateRequest (current generation, Participant, intent, exact refs, budget)
  → one qualified read transaction: current S/EffectivePolicy + historical cut + D
  → existing Recall v2 discovery/resolution
  → private qualified atomic evidence groups
  → existing SemanticState compiler and shared residency budget kernel
  → SemanticWorkingState v3 (yai.state_compiler.v3)
```

The request embeds the existing `CompilationRequest`; intent mechanically becomes
the Recall query. `--ref` requires exact remembered backing; `--require` first
resolves current S and otherwise becomes a required Recall reference. `--at`
qualifies remembered evidence by exact generation/Transition, never execution
authority. `--resource` is exact execution focus, not fuzzy entity inference.
The Participant must be currently linked and have the existing admitted model
context view even though this operation performs no model call.

Current authority, bindings, lifecycle, unresolved control, visible resources,
Participant and task remain pinned independently of Recall ranking. The v3 path
does not run S's older derived-memory retriever: remembered content comes from
Recall. Each `RecalledEvidence` entry carries typed event/segment/assertion,
documentary, relation and exact closure values from a resolver-qualified group.
It does not carry a serialized RecallTrace, candidate list or narrative. Repeated
hits resolving to the same semantic membership reuse a group and its first
qualified selection explanation. Documentary `source_stated`, recorded observation,
historical DecisionBasis and unresolved disagreements keep their nested posture;
the outer entry remains derived memory, including when an exact requirement pins
it. Selection does not admit a fact or reactivate historical policy.

The inspection default uses the existing 48-item view and Recall 16-candidate /
16,384-unit envelope, with a 32,768-unit final W and 1 MiB byte ceiling. This
inspection envelope accommodates the measured bounded D/H groups plus current
control and provenance; the 16,384-unit pressure oracle omits optional groups
atomically. It is not a provider-capacity or universal-sufficiency promise. `--limit`,
`--units`, `--bytes` and `--candidates` make pressure explicit. Semantic units remain
serialized Unicode-character quarters, not model tokens. Final W charging includes
identity, request, provenance and omission metadata, not just selected sentences.
Optional groups may be omitted whole for exact task locality or final budget;
mandatory current state and required recalled groups refuse on overflow. Required
incomplete backing refuses. Optional incomplete Recall retains global limitations,
closure posture and candidate omissions plus each included group's typed missing
backing. None of these fields claims universal execution sufficiency.

Identity binds compiler/schema, qualified current visible entries, current Case
generation, scope, intent/output contract, exact requirements, budgets, Recall ID
and disclosed source identity, selected evidence and omissions. Timing is outside
identity. No private candidate universe is exported. One request reuses the same
source/authorization basis: W never repeats BM25, graph discovery or source reads.
Current-state projection and pure group assembly still scan/copy source-derived
values; full-history CPU and overlapping closure cost are not constant.

`validate_working_state_authorized` requalifies the current request and compares
the entire W, including when Case generation is unchanged. Losing exact backing,
catalog revocation or changed Recall invalidates old W independently of an S-only
delta. This operation explicitly returns `FullRecompilation`; forward SemanticDelta
v1 remains unchanged and cannot certify equal-generation Recall freshness.

`QualifiedWorkingState::lower_context` (CLI `--projection`) lowers only its selected
W entries as Projection v11 after integrity recompilation from the same in-process qualified basis;
it performs no second read/discovery. This is snapshot-compatible lowering, not a
claim that a retained snapshot remains authorized forever. A later consumer must
requalify current authority/backing; no execution is performed here. Existing
conversation/Workflow provider entrypoints retain their independently qualified
S-only v2 path. This milestone neither migrates their invocation lifecycle to v3
nor permits a v3 artifact to bypass existing transactional invocation admission.
ContextFrame v11 preserves the same entries, requires the compiled intent/output
contract and adds no memory. A pinned, non-authoritative `RecallQualification`
entry carries Recall closure/limitations and W omissions through both lowerings;
metadata cannot disappear while its selected evidence remains. V10 remains the
existing provider path; v11 is explicit derived compatibility, not a W→E schema.

W/Recall remain disposable. Transition v19, CaseState v16, canonical owners and
37 LMDB databases are unchanged. The v3 operation introduces no W→E transport,
provider calls, paging, ambient refresh or learned navigation.

Executable oracles: `make smoke-recall-working-state` reuses the real source
bootstrap/knowledge fixture (Markdown/JSON/PDF/filesystem/SQLite), with current
control, distant history, exact Decision/Observation references, task switches,
contradictions, revision/as-of, revoke, missing backing, atomic overflow, restart
and graph/index/memory rebuild. The typed policy oracle in
`engine/yai-engine/src/store/tests/recall_tests.rs` compares historical ALLOW
basis with current replacement/revocation, stale/tampered W and identical lowering.

Legacy archaeology additionally inspected `yai-dev` at
`8a2b09e268fe6e20b1681dab7b22eac6b8239a8c`,
`src/agents/grounding/context_pack_selection.c` and
`context_pack_completeness.c`: required source references, explicit partial/stale
postures and completeness distinct from sufficiency belong in this current
compiler. Its Agent pack owner and `src/knowledge/workset/working.c` global
hotness/upsert working-memory lifecycle remain rejected, not restored.

## Scoped semantic paging

`case context compile CASE INTENT --paged` opts into **SemanticWorkingState v4**
and compiler v4; omitting the flag preserves v3. It uses the same qualified Recall
and W compiler, not another retriever. `--resident-groups` bounds optional evidence
residency independently of mandatory current state and exact task dependencies.

The bounded `yai.exact_semantic_group.v1` reference profile locates an already
qualified, closed Recall group: exact event/unit/source-descriptor members,
source/revision/path coordinates, historical cut and normalized semantic digest.
The reference catalog is itself bounded and charged to W. Incomplete or oversized
groups are explicitly counted as nonpageable; they cannot masquerade as closed
pages. References are locators, not capability tokens or resident evidence.
Deferred means revalidation required, not false, deleted or permanently readable.

`LmdbRecordStore::page_working_state_authorized` consumes a serialized derived W,
authenticated Principal and `yai.semantic_page_request.v1` (base identity,
Participant, exact catalog references, page-in/page-out and existing RecallBounds).
The native consumer is `case context expand CASE --working-file FILE --ref REF`.
It returns typed **SemanticPage v1**, expanded W v4, evictions and measurements.
No canonical page store or Transition is created.

One read transaction requalifies current Participant/disclosure, current control
and source permissions. Exact source/revision/path scope is applied before source
authorization/byte derivation; acquisition Decisions also inherit their exact
source's visibility gate. Retained resident groups and requested incoming groups
are rebuilt through existing historical, documentary and experience owners.
The shared resolver skips candidate discovery, BM25 search, vectors and graph fanout;
it resolves only declared members and required closure. Closure escaping the
declared scope, changed semantic backing, missing material or overflow refuses.
No live file substitutes for missing captured backing. Historical evidence stays
at its original cut; current EffectivePolicy remains the present control basis.

This is fixed-generation working-set continuation, not task refresh. Case/task
replacement requires compilation; same-generation current control changes also
refuse. The caller names its expected base W identity; there is no global mutable
"latest W" registry or server-side task session. Every retained/requested group is revalidated, including backing loss
outside Case generation. Unrequested deferred backing is not eagerly read and
has no promised availability. W/file hashes detect identity changes but grant no
authority: current owners independently qualify selected material. This does not
certify freshness of the original global relevance universe; a new question or
new relevance discovery belongs to Recall/full compilation.

Page bounds cover aggregate items, relations, segments, serialized bytes and
Unicode-character-quarter semantic units, not model tokens. The exact profile
has no recursive depth/crawler mode. Required closure is atomic. The unchanged
resident W envelope separately includes control, references, provenance and
resident groups. Incoming groups must fit or refuse; optional resident groups
may leave through deterministic existing compiler selection. `--page-out` only
removes optional residency, never source/history or mandatory task dependencies.
Page-in does not permanently turn an optional group into a task requirement.

W identities bind original Recall/compilation basis, current semantics, catalog,
parent W, exact page/request, budgets and omissions. Page identity binds its
request, base W, current control and exact resolved groups; timings are excluded.
Page-out and later page-in reproduce qualified content, not the old lineage ID.
Projection/ContextFrame **v12** lower only selected W4 entries, including deferred
reference posture; they never fetch a page. Provider/Conversation/Workflow still
use their qualified S-only W2/Projection10 path. W3/Projection11 remain compatible.

`make smoke-semantic-paging` exercises the actual CLI and real persistence;
the adjacent typed policy oracle covers old-cut evidence under current DENY,
reopen/cache rebuild, missing/hidden and zero discovery/mutation. Characterization
separates scoped source resolution, exact group qualification, closure, W compile
and page-out. History/access derivation still scans history and repeats bounded
experience derivation per retained group; exact paging is not O(1).
The existing M07 derivation also validates a lexical index over the scoped sources;
it performs no search and does not index the whole Case on a page request.
Arbitrary source continuations, neighborhood crawling, optimal eviction, automatic refresh,
provider demand loops and computational/KV paging are not qualified here.

Legacy archaeology at `yai-dev` commit
`8a2b09e268fe6e20b1681dab7b22eac6b8239a8c` inspected
`src/knowledge/retrieval/exact.c`: its caller-supplied high-score references were
not a source/disclosure-qualified pager. `context_pack_completeness.c` preserved
the useful distinction between reference presence and consumability/partial/stale
posture. Those failure distinctions belong in this compiler; historical Agent
packs, mutable worksets and their ownership tree remain rejected.

## Prompt-independent semantic refresh

`case context refresh CASE --working-file FILE` reuses the task already carried
by W3/W4. It accepts no prompt, intent, new exact reference or historical cut.
The typed owner is `LmdbRecordStore::refresh_working_state_authorized`, with
`yai.working_refresh_request.v1` and `yai.working_refresh_result.v1`. The request
binds the expected base W ID, Case and Participant, plus an optional explicit
item/unit/output-byte envelope. Scope, task, output contract, exact requirements,
Recall bounds and an explicit evidentiary cut are otherwise preserved.

```text
prior W request (not its remembered contents)
  → current generation/Participant/authority/source basis in one read transaction
  → existing Recall v2 candidate discovery and qualified reconstruction
  → existing full W compiler
  → W3, or W4 with freshly qualified paging preferences
```

Unlike exact paging, refresh intentionally reconstructs global bounded Recall.
There is no refresh-specific retriever or repeated source-discovery pass inside
W. Current generation is resolved in Recall's own read transaction, not through
a preliminary read followed by a different compilation snapshot. An absent
historical cut advances to that current basis; an explicit generation/Transition
cut stays pinned. Current disclosure and execution authority apply to both.
Current DENY/blocked policy can be represented for inspection; a W is not a
Grant and refresh never executes an operation.

The returned private qualified basis supports downstream lowering without
another read/retrieval. It is a snapshot, not a freshness lease: future consumers
must perform current preflight before affected consumption. The operation is the
explicit preflight/reconstruction seam, not a watcher or automatic migration of
provider/Conversation/Workflow W2 consumers. Existing W3/Projection11 and
W4/Projection12 versions are unchanged. Projection does not discover staleness,
run Recall or recover old page contents.

Correctness-critical invalidation and enrichment use the same reference path but
mean different things: lost disclosure/mandatory backing must remove or refuse
old affected material before use; newly admitted evidence can change relevance.
The result reports current-control/Recall identity comparisons and whether the
predecessor must be replaced, not a speculative diagnosis of every cause.
It never returns removed hidden IDs/counts. Current outputs/identities derive
only from the freshly qualified universe. Missing exact mandatory material or
mandatory envelope overflow refuses. Optional material may disappear under
existing omission rules; `incomplete` means the selected Recall closure is
incomplete, not that every formerly relevant source was revisited successfully.
No result claims universal task sufficiency.

W4 refresh resets old page-in/out lineage to a newly compiled qualified basis.
Only groups in the new Recall catalog can retain a working-set preference,
matched by exact members, source/revision/path and normalized backing digest.
Prior resident groups are preferentially pinned where they fit; prior deferred
groups remain deferred where the same qualified group exists. New mandatory
task/current requirements override optional preferences. Changed, undisclosed,
unavailable or no-longer-discovered groups are not copied or silently sought by
another exact/global retrieval pass. Failed optional preference fitting falls
back to current bounded selection. A later page-in revalidates again.

The result distinguishes `unchanged`, `recompiled_equivalent`, `refreshed` and
`incomplete`; refusals use the existing typed-operation error path. Identity
equality is exact. `selected_material_equal` is the narrower comparison of
current entries and normalized selected evidentiary backing: it excludes Recall
metadata, reference catalogs, selection reasons and derived segment organization.
It does not promise equal task sufficiency, equal page lineage or universal
semantic equivalence. Timings never participate in W identity. The predecessor
ID is caller-supplied lineage, not a canonical LatestW record. Artifact hashes
detect changed identity, not authenticity/permission; a consumer protecting a
specific task artifact must retain and supply its expected base ID. A fully
changed artifact is a different base, not an authorized update of the old task.

Refresh always reports `FullRecompilation`. SemanticDelta v1 remains the forward
S-only contract: equal-generation backing loss cannot be represented by it.
The typed oracle proves identical qualified S before/after historical artifact
loss while Recall changes; it does not fabricate an equal-generation empty
delta. No global epoch, dependency database, W registry, Transition or canonical
owner is added.

`make smoke-semantic-refresh` exercises actual CLI/persistence, same-task source
revision and new governed Observation, current/as-of, optional/mandatory backing
loss, source and equal-generation policy revoke, W4 resident/deferred preferences,
restart/drop/rebuild and zero mutation/model calls. It compares refresh against
fresh compilation across short/long histories and small/larger source corpora.
Measurements expose current composition, qualified history/source resolution,
existing Recall discovery/assembly, W compile, W4 preference recompilation and
compatibility lowering. Nested measurements are not additive; full-history/source
CPU remains size-dependent and refresh is not claimed cheaper than compilation.

Fresh legacy archaeology at `yai-dev`
`8a2b09e268fe6e20b1681dab7b22eac6b8239a8c` inspected
`src/agents/grounding/live_context_refresh.c`, `live_context_consumption.c` and
`src/knowledge/context/case_state_refresh.c`, plus their consumers and history.
The first pair rejects consumption of stale/refresh-required snapshots; that
failure distinction is preserved here through current preflight. The latter
applies a delta or updates a timestamp and persists cognition/checkpoint state;
it is not qualified same-task Recall/W reconstruction. Its global cognition and
Agent live-context ownership are not recovered. The substrate-drain commit
`2a4018147219044dfe1fad2268759b1f2a585945` removed those historical owners.
