# Container Session Binding Surface

`sys/container/session/` owns session-domain binding for container operations.

## Responsibilities

- bind/unbind/rebind sessions to a container domain
- materialize container-scoped interactive context for bound sessions
- expose session-domain runtime handles and binding state
- coordinate with kernel admission primitives without replacing them

## Implemented surfaces

- `session_binding.c`: canonical session-to-container binding flow
- `bindings.c`: binding helpers and binding state transitions

## Boundary

- kernel owns privileged session admission and revocation roots
- `sys/container/session` owns how admitted sessions live inside a container domain
