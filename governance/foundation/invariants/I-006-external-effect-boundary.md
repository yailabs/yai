# I-006 — External Effect Boundary

External effects require explicit strengthened governance.

## Invariant

Transitions classified as external-effect MUST be explicitly identified before execution and processed through `core` authority with `exec` boundary enforcement.

## Required properties

- pre-execution classification predicate is explicit
- strengthened authority applies to external-effect transitions
- augmented evidence includes target, effect class, risk, and mitigation note
- compliance context requirement is enforced (see I-007)
- side-channel bypass is prohibited

## Ontology alignment

`exec` is the primary boundary for external effect realization.
`core` is the primary authority boundary for allowing or denying those effects.
`brain` may propose but cannot directly perform external effects.
