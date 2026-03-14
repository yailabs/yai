# dmn

`target_fs/sys/dmn/` is the daemon service surface.

## Scope

This domain keeps only the daemon process entry shell.

## What stays here

- `cmd/daemond/`: canonical daemon entrypoint
- this README as service-surface documentation

## What does not stay here

Daemon control ticks, health ticks, runtime state, runtime config, source-plane
integration, replay logic and technical internals belong to `target_fs/krt/dmn/`.

## Boundary

`sys/dmn` exposes the daemon service process.
`krt/dmn` owns daemon runtime implementation.
