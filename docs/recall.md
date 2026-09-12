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
navigation, all-owner S/Workflow Recall, general event-time algebra, automatic
R → W, or model-native state is claimed. The next W boundary is not selected by
this contract.
