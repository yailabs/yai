# Kernel/Enforcement Alignment Audit

## Scope

This audit links formal semantics to kernel-side enforcement attachment points.

## Kernel Enforcement Inputs

- session admission validity
- grant validity and revocation
- containment state and escape/breach control
- lifecycle/readiness gating
- policy hook allow/deny/defer/privileged-path outcomes

## Required Formal Counterparts

- authority admissibility invariants
- grants validity/revocation invariants
- containment mode/state invariants
- lifecycle transition and readiness invariants
- policy application precedence invariants

## Alignment Outcome

Formal coverage exists for the above classes and is linked through
`control/assurance/traceability/enforcement-linkage.json`.

## Transitional Note

Any runtime-local/workspace semantics remaining under `runtime/compatibility/`
are migration artifacts and must not be interpreted as kernel ownership.
