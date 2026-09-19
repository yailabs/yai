# Bounded Semantic Working State Sufficiency Qualification

This report records the first model-independent YAI evaluation of whether
qualified Recall v2 plus Semantic Working State supplies the declared semantic
prerequisites for a bounded set of tasks. It is an evaluation artifact, not Case
authority, a production benchmark, or a claim of general Recall/W sufficiency.

## Reproduction and evidence

Run from the repository root:

```sh
RUSTUP_TOOLCHAIN=1.98.1 make smoke-semantic-working-state-sufficiency
```

The retained qualification implementation is
[`test_sufficiency.py`](test_sufficiency.py); its independently authored task
requirements are [`oracle.v1.json`](oracle.v1.json). The oracle is loaded and
hashed before Recall or W is inspected. Fixture labels are then bound to exact
source revisions, knowledge units, Transitions, Decisions, Observations and
relations created by the disposable product fixtures. Benchmark labels never
enter production requests or persisted Case truth.

Qualification run:

- run ID: `temporal-causal-relation-closure-final`
- inner run: `semantic-working-sufficiency-38`
- order: `11`
- material pre-state: `03916815ff8b371b4646028b6b608236d380a6d0`
  plus the six wave-owned oracle/test/documentation edits
- working directory: `/home/mothx/computer-science/projects/YAI/yai`
- oracle digest: `sha256:a104fb2971abb4169852e8796427aaa086ccb9a9386dac031984b628ee3c0e6f`
- provider/network posture: none / none
- exit status: `0`
- elapsed: `169.709 s` wrapper, `169.530 s` evaluator

Bounded unedited closure stdout line:

```text
semantic_working_state_sufficiency=PASS tasks=15 sufficient=14 refused_correctly=1 insufficient=0 forbidden_disclosure=0 provider_calls=0 model_calls=0 evaluation_transitions=0
```

`PASS` means the evaluation contract executed and reported all selected results;
it is not a model-answer score or a general sufficiency claim.

The same retained run ID carries two separately executed relation controls:

- order `2`, pre-state `exact-case-fixture-short-and-20081-transition-profiles`,
  command `cargo test --manifest-path engine/Cargo.toml
  recall_discontinuous_short_long_characterization -- --nocapture`, exit `0`;
- order `3`, pre-state `resolver-counterfactual-over-qualified-historical-view`,
  command `cargo test --manifest-path engine/Cargo.toml
  recall_relation_counterfactual_and_hidden_intermediate_do_not_become_memory
  -- --nocapture`, exit `0`.

Bounded unedited stdout excerpts:

```text
recall_characterization history=81 gap_each=0 source_visible_events=72 candidate_count=3 candidate_discovery_us=10895 relation_build_us=11537 qualification_us=516 closure_us=21 end_to_end_us=145975 selected_events=8 selected_relations=14 segments=5 units=5294 bytes=21176 exact_decision_observation=true direction=decision_to_observation provenance=typed_object_identity+observation.decision_id false_causal_links=0 pre_observation_cut_relation=0 relation_budget_refused=true zero_transitions=true
recall_characterization history=20081 gap_each=10000 source_visible_events=72 candidate_count=3 candidate_discovery_us=10739 relation_build_us=11291 qualification_us=13418 closure_us=20 end_to_end_us=1037041 selected_events=8 selected_relations=14 segments=5 units=5308 bytes=21230 exact_decision_observation=true direction=decision_to_observation provenance=typed_object_identity+observation.decision_id false_causal_links=0 pre_observation_cut_relation=0 relation_budget_refused=true zero_transitions=true
recall_counterfactual scope=resolver_contract normative_link=present_vs_absent observation_link=typed_decision_id chronology=unchanged same_resource_without_link=false_causal_relations_0 missing_backing_relation_0 hidden_intermediate=semantic_identity_equal canonical_mutation=0
```

## Oracle and metrics

Each task declares mandatory evidence, optional material, known distractors,
mandatory current controls, forbidden material and permitted refusal. The
evaluator reports a vector rather than one product score:

- mandatory evidence coverage: resident required identities found in W divided
  by independently declared required identities;
- Recall required coverage: which required recalled identities reached Recall;
- control preservation: mandatory current Case, Tenant, authority and task
  entries survive selection;
- distractor admission and forbidden disclosure: independent counts, never a
  relevance bonus;
- exact closure and epistemic postures;
- Recall-stage versus W-stage missing identities;
- semantic units, output bytes, omission counts and phase timings.

The aggregate mandatory coverage is `0.96`. This intentionally counts the
missing-backing task's unavailable required item as not resident even though W
correctly refused; it must be read with `REFUSED_CORRECTLY=1`, not as an opaque
score. All 14 compilable task outcomes preserved current control. One unrelated
repository documentary unit was admitted in the documentary lookup; the two
temporal tasks admitted zero distractors. Forbidden disclosure was `0`.

## Task results

| Task class | Posture | Required W coverage | Recall coverage | Control | Distractors | Forbidden |
|---|---:|---:|---:|---:|---:|---:|
| current-state lookup | SUFFICIENT | 1.00 | 1.00 | yes | 0 | 0 |
| historical reconstruction | SUFFICIENT | 1.00 | 1.00 | yes | 0 | 0 |
| documentary knowledge | SUFFICIENT | 1.00 | 1.00 | yes | 1 | 0 |
| documentary/operational contradiction | SUFFICIENT | 1.00 | 1.00 | yes | 0 | 0 |
| temporal-causal explanation | SUFFICIENT | 1.00 | 1.00 | yes | 0 | 0 |
| mixed-source governance/knowledge | SUFFICIENT | 1.00 | 1.00 | yes | 0 | 0 |
| cross-Case source reuse | SUFFICIENT | 1.00 | 1.00 | yes | 0 | 0 |
| revoked evidence | SUFFICIENT | 1.00 | 1.00 | yes | 0 | 0 |
| task switch | SUFFICIENT | 1.00 | 1.00 | yes | 0 | 0 |
| exact mandatory reference | SUFFICIENT | 1.00 | 1.00 | yes | 0 | 0 |
| long-history distractor pressure | SUFFICIENT | 1.00 | 1.00 | yes | 0 | 0 |
| large-source distractor pressure | SUFFICIENT | 1.00 | 1.00 | yes | 0 | 0 |
| wrong-memory lure | SUFFICIENT | 1.00 | 1.00 | yes | 0 | 0 |
| missing backing | REFUSED_CORRECTLY | 0.00 resident | 1.00 candidate | n/a | 0 | 0 |
| hidden evidence | SUFFICIENT | 1.00 | 1.00 | yes | 0 | 0 |

The contradiction task retained `source_stated`, `recorded_observation` and
`control_state` together. Historical and wrong-memory tasks retained the old
source revision at the exact cut while present control remained current. The
mixed-source task used the real `yai-mixed-v1` Markdown fixture: its exact
documentary unit reached Recall/W, while EffectivePolicy came only from the
independent publication/binding path. Cross-Case A/B visibility and C absence
used one exact same-Tenant backing without sharing authority.

## Temporal-causal finding and correction

The original qualification at `f5bc25e928c2d89ce2ba292d6f0dc62b04b7ca78`
reported both temporal tasks at `0.67`: exact Decision and Observation endpoints
were found, the relation selector reported `decision_observation` missing, and
`working_stage_failures=0`. That retained finding triggered a full producer →
graph → Recall → W trace rather than a ranking change.

The exact relation already existed before Recall and already survived Recall
closure. The evaluator was wrong: its generic selector required fields named
`from`/`source` and `to`/`target`, while the canonical `ExperienceRelation`
contract uses `relation_id`, `from_event`, `to_event`, `posture`,
`known_at_generation` and exact `sources`. The corrected oracle now binds the
database source's exact Observation backing to its canonical Transition, finds
the exact Decision endpoint and relation in the qualified experience view
*before* Recall, and requires that identical identity, direction and provenance
in Recall and W. It does not derive ground truth from Recall output.

No production relation producer, graph resolver, Recall ranking, Recall closure
or W selection code changed. The short and 256-change tasks both reach `1.00`;
Recall-stage and W-stage failures are `0`. The independent controls also prove
that same-Resource endpoints, immediate recording order, lexical similarity and
an unrelated Decision do not mint an edge. A pre-Observation cut, missing exact
Decision backing, hidden endpoint or crossed Case scope does not expose one;
relation-bound overflow refuses instead of returning endpoints as a falsely
complete group.

Task switching changed W, turned over 31 task-specific knowledge identities,
preserved all mandatory control and admitted zero previous-task identities.
The exact-reference task retained its mandatory Decision. Missing immutable
backing refused instead of reading the live file. Revoked and cross-Case hidden
material produced zero forbidden disclosure.

## Budgets, paging and refresh

| Budget | Semantic units | Result | Selected units | Optional omissions |
|---|---:|---|---:|---:|
| generous | 131,072 | SUFFICIENT | 18,100 | 0 |
| normal | 32,768 | SUFFICIENT | 18,100 | 0 |
| constrained | 16,384 | SUFFICIENT | 15,594 | 1 |
| impossible | 1 | REFUSED_CORRECTLY | — | — |

The exact mandatory Decision survived the constrained profile while one optional
item was omitted atomically. The impossible profile refused rather than
truncating mandatory semantics.

W4 with zero requested resident recalled groups kept the target documentary
unit deferred. An explicit exact page supplied it with complete group closure,
zero candidate-discovery passes, 11,441 page semantic units and 20,875 expanded-W
units. Deferred material was not counted as resident task evidence.

After a source revision, the existing ambient consumer operation returned
`refresh_required`, preserved the exact task, supplied a current replacement W,
made zero provider calls and appended zero refresh Transitions.

## Scale and performance characterization

Timings are single-run characterization in microseconds and are not additive:
the qualified-read and assembly measurements contain nested work.

| Profile | Transitions/generation | Sources | Qualified read | Historical resolve | Knowledge resolve | Recall assembly | W selection | W units / bytes |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| short history / small source set | 83 | 10 | 452,261 | 125,287 | 248,955 | 376,395 | 13,110 | 22,236 / 88,942 |
| short history / large source set | 290 | 73 | 6,868,591 | 563,513 | 5,929,289 | 1,895,803 | 16,493 | 25,914 / 103,655 |
| 256-change history pressure / small source set | 338 | 10 | 508,139 | 154,528 | 248,431 | 421,619 | 12,360 | 20,731 / 82,922 |

For the corrected temporal task specifically, relation production took
`10,898 µs`, qualified assembly `386,794 µs`, and W selection `14,082 µs`;
the 256-change control measured `10,821 µs`, `421,619 µs`, and `12,360 µs`.
Each used one candidate-discovery pass. These are single-run characterizations,
not latency bounds.

The 73-source profile makes source/knowledge resolution the dominant cost.
Irrelevant history/source growth did not force unbounded W growth, but no
constant Case-age, source-count, or latency property is claimed.

## Legacy archaeology

`yai-dev` at `5c1c7b9d099eea9f2947146cd821d6501c4a6ddf` and its relevant
history were reinspected, including `src/lineage/README.md`,
`substrate_graph_relationship_contract.c` and
`substrate_graph_reconstruction_contract.c`. The reusable doctrine is that an
owned relationship has exact provenance and direction, graph queries consume
rather than invent its semantics, and reconstruction uses structured records.
The historical Lineage plane, C ownership tree and registries are not restored.
The current Rust experience graph is the existing relation owner; Recall and W
remain downstream derived consumers.

## Nonclaims and remaining work

- This is not a model-answer benchmark or a universal sufficiency claim.
- The suite qualifies only the exact recorded Decision→Observation relation
  closure used by these bounded fixtures. It does not establish general causal
  inference, learned navigation, arbitrary prose understanding, general paging,
  all-owner Recall or constant-scale cost.
- It creates no Transition, CaseState, LMDB, canonical owner, Recall/W schema,
  provider dependency or persisted benchmark result.
- It makes no W → E, StateProfile, State Read, Program N or B1 claim.
- The semantic oracle is deliberately realization-independent so a later
  context-versus-E State Read comparison can reuse the same Case/task/required
  sets without changing semantic ground truth.
