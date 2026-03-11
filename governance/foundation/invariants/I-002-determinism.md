# I-002 — Determinism

Determinism is scope-bound and governance-bound.

## Invariant

Given identical declared scope inputs, policy material, and authority context, governed outcomes MUST remain behaviorally equivalent.

## Plane obligations

- `core`: authority outcomes and reason codes must be deterministic under declared scope.
- `exec`: effect execution paths must be reproducible within declared nondeterminism bounds.
- `brain`: cognitive proposal variance is allowed only when bounded and evidence-linked; sovereign outcomes remain deterministic.

## Prohibited patterns

- unbounded nondeterminism that changes governed outcomes
- opaque adaptation that prevents replay-level reasoning
- hidden environment dependencies not declared in evidence
