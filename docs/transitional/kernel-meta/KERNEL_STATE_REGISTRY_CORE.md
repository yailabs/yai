# K-4 Kernel State and Registry Core

## Minimal kernel-owned state

`kernel/state/kernel_state.c` is the privileged root state for:
- kernel lifecycle phase (`bootstrap`, `init`, `ready`, `degraded`, `recovery`, `shutdown`)
- kernel capability state (root caps + subsystem readiness flags)
- registry roots (kernel, container, session, capability, containment, trace, grants)
- privileged counters (admitted sessions, active containers, audit channels)

## Privileged registries

- `kernel_registry` (`kernel/model/registry/kernel_registry.c`)
- `container_registry` (`kernel/model/registry/container_registry.c`)
- `session_registry` (`kernel/model/registry/session_registry.c`)
- `capability_registry` (`kernel/model/registry/capability_registry.c`)
- `containment_registry` (`kernel/model/registry/containment_registry.c`)
- `trace_registry` (`kernel/model/registry/trace_registry.c`)
- `grants_registry` (`kernel/model/registry/grants_registry.c`)

## Ownership discipline

Only kernel-owned code paths can mutate privileged registries.
Higher layers may consume read surfaces through mediated kernel interfaces, but must not:
- create containers bypassing kernel registry roots
- bind sessions bypassing kernel admission/registry roots
- mutate containment state directly
- mutate privileged grants root directly

## Non-goals

This core does not implement high-level policy/governance/orchestration/data/graph logic.
Registries stay minimal, structural, and privileged.
