# Shell Surfaces

Operator-facing visual surfaces: panels, navigation, inspector, and workspace/session shell.

No backend implementation code should live here.

## Boundary
- consumes runtime/protocol contracts
- does not implement compositor or renderer internals
- maps operator actions to `sys/` service planes through `sdk/`
