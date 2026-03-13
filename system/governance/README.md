# governance

`sys/governance/` is the L2 governance authority root.

## Canonical service

- `yai-governanced/`

## Responsibilities

- authority resolution and escalation paths
- governance decisions lifecycle
- publication of governance outcomes for consuming services

## Boundaries

- complements `sys/policy`; does not duplicate policy engine internals
- does not own orchestration execution internals
- does not own kernel privileged primitives
