# init

`sys/init/` owns system bootstrap and shutdown sequencing for L2 services.

## Canonical entrypoints

- `yai-init/`
- `yai-shutdown/`

## Responsibilities

- start order and early userspace initialization handoff
- shutdown ordering and termination discipline
- minimal service boot coordination with `supervisor`

## Boundaries

- does not own kernel boot primitives (owned by `boot/` and `kernel/`)
- does not own orchestration, policy, governance, graph or data engines
- does not own user-facing CLI or shells
