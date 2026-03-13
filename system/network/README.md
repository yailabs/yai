# network

`sys/network/` is the L2 network control root.

## Canonical service

- `yai-netd/`

## Responsibilities

- discovery, topology and routing control
- transport and mesh coordination
- provider-facing network integration surfaces
- service entrypoint ownership via `yai-netd/`

## Active domains

- `discovery/`
- `topology/`
- `mesh/`
- `routing/`
- `transport/`
- `providers/`
- `include/yai/network/` canonical network headers

## Boundaries

- kernel owns low-level socket/net primitives
- `sys/network` owns high-level control-plane behavior
- does not own orchestration workflow semantics

## Cutover status

DR-2 hard cut applied: active network implementation moved out of
`runtime/compatibility/lib/network/` into this root.
