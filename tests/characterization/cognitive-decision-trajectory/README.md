# Bounded Decision trajectory characterization

The executable oracle is the historical Case/experience fixture in
`engine/yai-engine/src/store/tests/historical_tests.rs` plus the typed
Application dispatcher tests in
`application/yai-application/tests/operation_flows.rs`. The fixture owns
its exact Decision, Operation, Observation, review and Policy identities.
The trajectory compiler does not supply expected relevance.

Run from the repository root:

```sh
cargo test --manifest-path engine/Cargo.toml -p yai-engine historical_policy_chronology_late_observation_and_current_permission --locked -- --nocapture
cargo test --manifest-path engine/Cargo.toml -p yai-engine decision_trajectory_short_long_characterization_and_backing_loss --locked -- --nocapture
cargo test --manifest-path application/Cargo.toml --locked --test operation_flows canonical_filesystem_effect_application_retry_observes_receipt_without_second_write
make smoke-case-resource-access
make smoke-case-source-bootstrap
```

The first fixture asserts an exact pre-cut, the selected Operation/Decision,
qualified Decision→Observation and review correction links, and absence of a
second same-Resource Observation or later Policy from the pre-state. The
second retains 10/266-Transition small/large observations, 256 misleading
`causal_refs`, rebuild/restart identity and policy-source loss. The Application
test asserts typed inspect/export/evaluate, hidden/wrong-Case refusal and zero
additional Transitions after inspection. The CLI product lane retains a later
content admission and proves it is absent from the original pre-Decision cut.
The Source lifecycle product lane refreshes an exact later Source revision and
proves its revision identity is absent from the original acquisition Decision's
pre-cut. It also reuses the exact physical Policy backing in a second Case and
refuses the first Case's Decision identity there.

Corpus queries take the earliest currently visible Decisions in canonical
order, at most 128, and report the count of later visible Decisions omitted
by the selected bound. Exact `inspect` can address any visible Decision within
the historical/graph budgets. The command's `--json` result is a structured
export; no dataset file is canonical.
`product-evidence.jsonl` retains the post-parity-fix CLI command transcript;
`evidence.jsonl` retains bounded test/characterization runs with their own
source fingerprints and run IDs. Earlier run IDs are not merged into the final
product claim.

Metrics are independent counts: trajectory and candidate-posture coverage,
historical W coverage, consequence/correction coverage, missing backing,
temporal/false-causality/cross-Case violations and serialized bytes. Timing
measures pre-cut historical reconstruction, relation-view resolution, full
trajectory and corpus read cost separately. Historical candidate reconstruction
is unavailable rather than timed as if it occurred. No
quality, reward, optimal-action, model calibration or Case-age-independent
CPU claim follows from these fixtures. Historical exact Frontier and
Distribution coverage is currently zero because those objects were not
recorded for canonical Decisions.

The retained `trajectory-20260923-final` run reconstructed five visible
Decisions: five partial candidate postures, zero exact candidate sets or
historical W/Distribution, two qualified consequence links and one explicit
review correction. The independent same-Resource Observation produced no
Decision relation; temporal, chronology-as-causality and cross-Case violation
counts were zero. After original Policy backing loss, missing-backing count
was one. At 10/266 Transitions respectively, pre-cut reconstruction took
19.2/38.1 ms, relation-view resolution 22.6/42.2 ms, full trajectory
43.5/79.9 ms, and one-Decision corpus 43.2/77.0 ms; the trajectory serialized
to 16,296 bytes in both fixtures. These are observations, not SLAs.

Legacy archaeology: `yai-dev` at `e9ad7f498` retained a
`episode_yielded_decision` graph edge in
`src/lineage/materialization/runtime_records.c` and a Decision trace
projection boundary in `src/lineage/decision/README.md`. The edge bound a
runtime episode to a Decision node; it did not provide a current-disclosure
qualified Case cut, exact candidate alternatives, or consequence closure.
This wave recovers the useful projection-only ownership rule, not the old C
graph plane or its episode owner.
