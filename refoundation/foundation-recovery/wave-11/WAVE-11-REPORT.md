# Wave 11 — temporal governance and durable Case closure

Baseline: `61afaf9c539ab7c41de89a1cb7afaaa4e93c2230`
Intended commit: `feat: add temporal governance and durable case closure`
State in this versioned report: implementation and qualification complete; publication pending.

This report is non-normative. Executable repository source, tests and captured
commands outrank it.

## Direct recovery verdict

Fresh inspection of `ca8e0ffb...`, `82675b14...`, `6bf6807f...`,
`c172c3bf...`, the additional `c513782ef` grant epoch and the final yai-dev
tree found an executable distinction among refresh-required, stale, expired and
revoked material. It also found the useful conservative rule that weaker/older
authority contracts autonomy. Legacy did not provide a durable Case
cancellation/closure or a PREPARE-equivalent temporal cut. It scattered
`time(NULL)`, string states and default windows across daemon/edge owners.

Recovered: explicit validity, conservative contraction, terminal revoke and
unused-Grant termination. Rejected: edge sovereignty, capability-envelope
taxonomy, workspace deletion as Case closure, volatile fixed registries,
daemon lifecycle planes and default/string authority.

## Implemented contract

Policy input/source/artifact v4 carries an immutable explicit bounded or
unbounded validity contract. Bounded windows require
`valid_from <= refresh_after <= expires_at`. Policy lifecycle v2 adds terminal
`Revoked` without mutating artifact bytes. Existing v1-v3 identities/readers
remain supported.

`NormativeReadiness` still answers materializability. `PolicyValidityPosture`
separately answers `Valid`, `NotYetValid`, `RefreshRequired`, `Stale`,
`Expired`, `Revoked`, or `Unavailable`. The weakest active binding controls
future authority. Supersession never moves an exact Case binding; only explicit
replacement refreshes it.

The existing `schema_meta` LMDB database now stores one authority-time floor;
no database was added. Canonical authority writes use one
`max(wall-clock,floor)` value transactionally. Queries display the same derived
value but never advance the floor. Clock rollback therefore cannot expand
authority; a forward jump may contract it early.

DecisionBasis v2 records evaluation time, exact binding validity and earliest
policy expiry. Decision v3 binds that basis. Grant v3 is finite:
`min(30 seconds, earliest policy expiry)` from the decision authority time.
At PREPARE the store atomically rechecks exact generation, Ready+Valid policy
and Grant expiry. Before PREPARE an Issued Grant may become Expired, Revoked or
Abandoned. After PREPARE the effect protocol owns truth and must finalize or
reconcile.

Transition/CaseState v7 add `ReviewInvalidated`,
`ExecutionGrantInvalidated`, `CaseCancellationRequested` and `CaseClosed`.
Cancellation atomically appends its barrier, invalidates usable reviews and
abandons unused Grants. It blocks new operational truth but permits already
external ProviderResult recording and effect settlement. Closure requires
cancellation, no live claim and no unresolved review/Grant/effect; it is
terminal and non-destructive.

## Canonical ownership

Canonical: PolicyArtifact/lifecycle, Case Transition history, DecisionBasis+
Decision, ReviewRequest/Action/Invalidation, ExecutionGrant/Invalidation,
PREPARE/Receipt/reconciliation, cancellation and closure.

Materialized current state: CaseState v7.

Derived: EffectivePolicy, NormativeReadiness, current validity, drift, indexes,
memory, graph, runtime checkpoints/admission.

Ephemeral: observed wall clock before the transactional authority fence and
evaluator temporary evidence state.

## Qualification verdict

The focused Wave11 suite covers explicit validity/window rejection, all current
postures, weakest composition, explicit refresh, revocation, rollback fencing,
review invalidation, finite Grant expiry, PREPARE cut, reconciliation, atomic
cancellation/closure, idempotence, replay and low-level write barriers. H10
forgery/re-derivation and P1→P2 historical replay tests remain green.

The product characterization uses one real Case and exact IDs to show Valid P1,
P1 stale after P2 publication, explicit P2 replacement, Decision/Grant/PREPARE,
P2 revoke, cancellation, unsafe close rejection, reconciliation, safe close,
and zero provider invocation after closure. Deterministic clock advancement is
test-only; no product clock-injection CLI was invented.

One implementation failure was found and fixed: carrier execution required the
global Case generation to remain equal to PREPARE generation, which prevented
settlement after a legitimate cancellation. The corrected invariant binds the
exact materialized Prepared effect and Prepared Grant; later cancellation or
revoke cannot erase it. Two harness failures (sandboxed Unix socket and a
case-sensitive reconciliation assertion/provider fixture wait) were diagnosed
separately and do not alter product semantics.

Final repository qualification then exposed a second real time-of-check/time-
of-use defect: the product derived a Decision in a read transaction and
committed it in a later write transaction, so crossing a millisecond could
produce `authority_decision_time_mismatch` or make Grant re-derivation compare
against a later time. New product Decisions are now derived and appended in one
RW transaction with one authority time. Grant admission re-derives the
historical Decision at its recorded time and separately requires current
Ready+Valid state, exact generation adjacency and unexpired Grant time. The
unchanged 26-turn runtime passed after this fix.

The final human-review smoke also exposed a stale test expectation, not a
product bypass: Wave11 now persists typed `ReviewInvalidated` with
`PolicyBasisChanged`, replacing the older H10-only `review_policy_basis_stale`
message. The characterization now asserts the durable typed event and its
terminal rejection. Full `make check` and `make characterization` then passed.

## Footprint and boundary

No semantic module/daemon/registry/C owner was added. One CLI-only
`case_lifecycle.rs` keeps parsing/rendering out of `main.rs`. Existing Rust
owners remain governance, case policy, admission, effect, transition, LMDB,
runtime and controlled-effect orchestration. No Wave12 identity, scheduler,
resource fencing, second carrier, provider governance, archive/delete or reopen
semantics are present.

The measured source footprint is: `main.rs` 1,931 → 1,949 lines; tracked files
863 → 880 after the 17 new Wave11 files are staged; Rust files 31 → 32; and
C/H/Rust files 155 → 156. The only new Rust file is the CLI-only file above.
No LMDB database was added (one key was added to the existing `schema_meta`
database), and the semantic-owner count is unchanged.

Standard Clippy completed with exit 0 and the same repository warning classes
already visible at the baseline (15 engine and 17 command warnings). Wave11's
temporary `unwrap`/argument-shape warnings were removed during implementation.
This report does not claim a repository-wide `-D warnings` gate.
