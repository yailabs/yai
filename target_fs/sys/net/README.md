# net

`target_fs/sys/net/` is the network service surface.

## Scope

This domain keeps only the canonical service entry shell for the network plane.

## What stays here

- `cmd/netd/`: canonical network entrypoint
- `netd/README.md`: service-facing identity note
- this README as service-surface documentation

## What does not stay here

Discovery, topology, routing, mesh, transport and provider runtime
implementation belong to `target_fs/krt/net/`.

## Boundary

`sys/net` exposes the network process surface.
`krt/net` owns the active network runtime.
