# ifc

`target_fs/sys/ifc/` is the canonical interface and protocol-contract surface.

## Scope

This domain owns only interface artifacts, protocol contracts and compatibility
bridges required by system services and runtime components.

## Contains

- `abi/`: control-plane ABI contracts
- `proto/`: protocol-facing headers and control-plane contract surfaces
- `wire/`: wire-level schema artifacts
- `legacy/`: narrow compatibility bridge code only
- `src/`: minimal interface-side helpers tied to protocol identifiers

## Does not contain

Runtime engines, service orchestration logic, policy execution, graph/data
implementation or daemon runtime state do not belong here.

## Boundary

`sys/ifc` defines and exposes contracts.
`krt/*` and `sys/*/cmd` consume those contracts.
