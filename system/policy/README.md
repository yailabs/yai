# policy

`sys/policy/` is the L2 policy service root.

## Canonical service

- `yai-policyd/`

## Responsibilities

- policy engine execution and overlays
- grants policy shaping and review surfaces
- system-service level enforcement decisions

## Boundaries

- kernel keeps low-level policy hooks and privileged grants validity checks
- `sys/policy` owns high-level policy composition and enforcement semantics
- does not own governance final authority publication flows
