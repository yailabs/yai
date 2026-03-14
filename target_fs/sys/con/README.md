# con

`target_fs/sys/con/` is the container service surface.

## Scope

This domain keeps only the canonical container service entry shell.

## What stays here

- `cmd/containerd/`: canonical container entrypoint
- this README as container surface contract

## Notes

Container image/bootstrap documentation can be folded here when no active
service-side implementation exists under `sys/con/image/`.

## What does not stay here

Container runtime logic, workspace binding, recovery/state, mounts and
runtime views belong to `target_fs/krt/con/`.
