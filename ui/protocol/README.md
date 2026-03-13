# Protocol

Defines stable contracts for surface identity, widget identity, event propagation, and state snapshots.

This layer is transport-neutral and backend-neutral.

## Boundary
- no rendering backend implementation
- no compositor loop logic
- no direct sys-plane calls
- consumed by `runtime/`, `shell/`, and `apps/`
