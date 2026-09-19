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

- run ID: `semantic-working-state-sufficiency-final`
- inner run: `semantic-working-sufficiency-38`
- order: `1`
- material pre-state: `f5bc25e928c2d89ce2ba292d6f0dc62b04b7ca78`
- working directory: `/home/mothx/computer-science/projects/YAI/yai`
- oracle digest: `sha256:a104fb2971abb4169852e8796427aaa086ccb9a9386dac031984b628ee3c0e6f`
- provider/network posture: none / none
- exit status: `0`
- elapsed: `176.990 s` wrapper, `176.802 s` evaluator

Bounded unedited final stdout line:

```text
semantic_working_state_sufficiency=PASS tasks=15 sufficient=12 refused_correctly=1 insufficient=2 forbidden_disclosure=0 provider_calls=0 model_calls=0 evaluation_transitions=0
```

`PASS` means the evaluation contract executed and reported all selected results;
it does not rewrite the two `INSUFFICIENT` task findings as successes.

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

The aggregate mandatory coverage was `0.88`. This intentionally counts the
missing-backing task's unavailable required item as not resident even though W
correctly refused; it must be read with `REFUSED_CORRECTLY=1`, not as an opaque
score. All 14 compilable task outcomes preserved current control. Forbidden
disclosure was `0`. One unrelated repository documentary unit was admitted in
the documentary lookup, recorded as distractor pressure rather than hidden.

## Task results

| Task class | Posture | Required W coverage | Recall coverage | Control | Distractors | Forbidden |
|---|---:|---:|---:|---:|---:|---:|
| current-state lookup | SUFFICIENT | 1.00 | 1.00 | yes | 0 | 0 |
| historical reconstruction | SUFFICIENT | 1.00 | 1.00 | yes | 0 | 0 |
| documentary knowledge | SUFFICIENT | 1.00 | 1.00 | yes | 1 | 0 |
| documentary/operational contradiction | SUFFICIENT | 1.00 | 1.00 | yes | 0 | 0 |
| temporal-causal explanation | INSUFFICIENT | 0.67 | 0.67 | yes | 0 | 0 |
| mixed-source governance/knowledge | SUFFICIENT | 1.00 | 1.00 | yes | 0 | 0 |
| cross-Case source reuse | SUFFICIENT | 1.00 | 1.00 | yes | 0 | 0 |
| revoked evidence | SUFFICIENT | 1.00 | 1.00 | yes | 0 | 0 |
| task switch | SUFFICIENT | 1.00 | 1.00 | yes | 0 | 0 |
| exact mandatory reference | SUFFICIENT | 1.00 | 1.00 | yes | 0 | 0 |
| long-history distractor pressure | INSUFFICIENT | 0.67 | 0.67 | yes | 0 | 0 |
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

## Findings

Both insufficient tasks fail at Recall, not at W. Their exact Decision and
Observation endpoints are present, but the independently required
`decision_observation` relation is absent from qualified Recall. W drops no
required item that Recall supplied: `working_stage_failures=0`. The same gap is
visible in the short temporal-causal task and after 256 unrelated historical
role changes, so this run does not attribute it to Case age alone. No fixture ID,
query term or ranking bonus was added to hide it.

Task switching changed W, turned over 31 task-specific knowledge identities,
preserved all mandatory control and admitted zero previous-task identities.
The exact-reference task retained its mandatory Decision. Missing immutable
backing refused instead of reading the live file. Revoked and cross-Case hidden
material produced zero forbidden disclosure.

## Budgets, paging and refresh

| Budget | Semantic units | Result | Selected units | Optional omissions |
|---|---:|---|---:|---:|
| generous | 131,072 | SUFFICIENT | 25,589 | 0 |
| normal | 32,768 | SUFFICIENT | 25,589 | 0 |
| constrained | 16,384 | SUFFICIENT | 15,666 | 2 |
| impossible | 1 | REFUSED_CORRECTLY | — | — |

The exact mandatory Decision survived the constrained profile while two optional
items were omitted atomically. The impossible profile refused rather than
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
| short history / small source set | 83 | 10 | 469,351 | 130,290 | 258,573 | 387,342 | 13,663 | 22,236 / 88,942 |
| short history / large source set | 290 | 73 | 7,197,587 | 584,494 | 6,222,242 | 1,965,501 | 17,102 | 25,914 / 103,655 |
| 256-change history pressure / small source set | 338 | 10 | 536,674 | 162,528 | 261,961 | 444,903 | 13,851 | 20,865 / 83,460 |

The 73-source profile makes source/knowledge resolution the dominant cost.
Irrelevant history/source growth did not force unbounded W growth, but no
constant Case-age, source-count, or latency property is claimed.

## Legacy archaeology

`yai-dev` history and current retained sources were reinspected, including
commit `dda93ee3a` and `src/agents/grounding/context_pack_completeness.c` plus
`context_pack_selection.c`. The strongest retained mechanism checks required
references/input-family presence, selected/omitted/stale posture and materialized
consumability; its own comment explicitly says a complete pack is not proof of
sufficiency for live consumption. Those useful distinctions are recovered here
as independent prerequisites, omissions and freshness controls. The historical
Agent Context/Context Pack owner, C global structures and presence-only
"complete" predicate are not restored. Current Recall/W and Case authority
remain the owners under evaluation.

## Nonclaims and remaining work

- This is not a model-answer benchmark or a universal sufficiency claim.
- The suite does not qualify the missing temporal-causal relation, learned
  navigation, arbitrary prose understanding, general paging, all-owner Recall,
  or constant-scale cost.
- It creates no Transition, CaseState, LMDB, canonical owner, Recall/W schema,
  provider dependency or persisted benchmark result.
- It makes no W → E, StateProfile, State Read, Program N or B1 claim.
- The semantic oracle is deliberately realization-independent so a later
  context-versus-E State Read comparison can reuse the same Case/task/required
  sets without changing semantic ground truth.
