# net

`target_fs/sys/net/` is the network service surface.

## Scope

This domain keeps only the canonical network service entry shell.

## What stays here

- `cmd/netd/`: canonical network entrypoint
- this README as network surface contract

## What does not stay here

Discovery, routing, topology, mesh, transport and provider runtime
implementation belong to `target_fs/krt/net/`.
