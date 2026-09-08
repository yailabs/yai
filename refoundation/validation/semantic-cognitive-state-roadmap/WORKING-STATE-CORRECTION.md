# Working-state and computational-update target correction

Authority: bounded documentation correction evidence, not a second roadmap.
Baseline: `09a5c08161733b469a0707c4921ac50c8dab465f`; clean physical master with
HEAD/origin/master/remote master reconciled before mutation. Intended commit:
`docs: clarify working-state compilation and computational updates`.
The final handoff records the containing SHA and publication equality.

## Requested correction

Only ROADMAP and its linked target doctrine change. Historical evidence remains
unaltered, including the original roadmap-refoundation report. These are adopted
target clarifications, not new implemented capabilities or maturity promotions:

1. `W_t = Compile(S_t, I_t, A_t, B_t)` precedes
   `M_t = Lower(W_t, Model, StateProfile)`. YAI compiles derived active working
   state under intent, authority/disclosure and budget; YVEX does not gain the
   whole Case. W_t owns no independent canonical history. Lower describes
   initialization/reconstruction, not reinitialization at every runtime step.
2. `Exec(X_t, M_t) → (Y_t, M_t+1, P_t optional)`: internal computational updates
   may proceed without a semantic event. Only explicit P_t requests semantic
   admission. Existing governed execution/result/effect evidence is not removed;
   recorded execution and admitted semantic consequence remain different facts.
3. Persistent KV alone proves no cognitive State Read. Intentional scoped W_t
   compilation, qualified persistent-prefix/KV lowering and observed model reuse
   remain a possible bounded experiment, including without new training. Source,
   scope, exact model/profile, reuse and invalidation/reconstruction need proof.
4. Primary programs for the selected refoundation are R/K/S/C/M/Q. A/E/O/W/X
   are protected invariants/consumers to preserve or reconnect, not permission
   to redesign all YAI. Selection remains SELECTED_NOT_STARTED; I07 unselected.

Snapshot, diagrams/equations, C11–C13 limits, Spectrum, horizon, traceability and
promotion/nonclaim wording are reconciled together. The target doctrine adds
future falsifiers for bounded KV realization and computational update without
a proposal. State Read/Update remain OPEN. Counts remain 28 ESTABLISHED,
22 PARTIAL, 14 OPEN, 4 LATER: 68 rows, no changed verdict.

## Archaeology and ownership

Re-read current ROADMAP, linked target, constitutional continuity boundary,
documentation guard and roadmap regression. The prior source-refoundation
evidence remains valid: this correction changes no executable mechanism.
Reinspect legacy `yai-dev` maturity/claim-boundary document at `69e4aa6a3`:
retain its rule that published text alone cannot promote capability. No legacy
owner, source tree, runtime mechanism or YVEX protocol is imported. The equations
and narrowed program scope come from the operator's explicit architectural
correction, not speculative reconstruction of legacy behavior.

Runtime/schema delta 0; semantic/operational owners +0; LMDB +0, still 37/40.
Transition v18 / CaseState v15 unchanged. No guard or test implementation change.
README, Constitution, REPLAI pin, product commands and ZERO-TO-CURRENT unchanged.
No runtime refactor, I07 or cross-repository implementation starts here.

## Validation / pre-publication closure

Use the existing documentation-only topology: documentation/layout/roadmap
guards, roadmap unit regression, topology assertions and static reachability
audit. Retain actual command/cwd/environment, run/order/pre-state, exit and
bounded unedited output in [working-state-validation.jsonl](working-state-validation.jsonl)
through the existing provider-connect-hardening capture utility. No product IDs
or provider calls are produced.

Run `working-state-correction-20260908`, repository-root cwd, TMPDIR=/tmp:

| Order | Command / invariant | Actual outcome |
|---|---|---|
| 1 | `make check-docs check-layout test-roadmap test-topology` | exit 0, 1.08s; documentation/layout/counts PASS, 8 roadmap + 10 topology assertions PASS, no_provider |
| 2 | `python3 tools/validation/topology.py audit --static` | exit 0, 0.06s; 71 old smoke leaves preserved, 86 current Make leaves, zero combined duplicates; no runtime tests claimed |
| 3 | `git diff --exit-code HEAD -- . ':(exclude)ROADMAP.md' ':(exclude)docs/semantic-state-execution-target.md'` | exit 0, empty output; every previously tracked file outside the two docs unchanged; the only new files are this correction record and its command evidence |

Documentation correction and qualification are complete. Pre-publication state:
ready for isolated staged-diff inspection and commit/push; final handoff records
the actual publication identity and remote equality, not a containing SHA here.

Golden local is NOT_AFFECTED (documentation-only). Golden external YVEX and
canary NOT_RUN; HUMAN_GOLDEN_CASE = PENDING_OPERATOR. Cumulative runbook
unchanged because no product behavior changes. YVEX EXTERNAL FINDINGS: no new
findings or live qualification; prior setup cost/Case 413 remain unresolved.
Cross-roadmap reconciliation still awaits the YVEX roadmap and must inspect
public property/consumer boundaries, not assume a deployed State Read contract.
