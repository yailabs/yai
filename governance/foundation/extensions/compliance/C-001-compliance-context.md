# C-001 Compliance Context Requirement

## Definition
A Compliance Relevant External Effect is any effect that:
- writes, reads, exports, erases, or transmits data outside volatile runtime state
- interacts with external providers
- mutates persistent state (graph, semantic, artifacts)

## Invariant
For any Compliance Relevant External Effect:

`ExternalEffect => HasAuthority AND IsTraceable AND HasComplianceContext`

## Violation
If `ComplianceContext` is absent or invalid, the effect MUST be denied.

## Binding
This extension specializes `I-006-external-effect-boundary`.
