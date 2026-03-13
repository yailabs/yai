# Container Mount and Attach Surface

`sys/container/mounts/` owns container mount semantics at L2.

## Responsibilities

- define governed mount objects and mount sets for a container domain
- coordinate resource attachments into container mount namespace
- expose mount readiness/attachability to runtime surfaces
- preserve visibility and policy classes for mounted resources

## Implemented surfaces

- `mounts.c`: mount attach/detach and mount-set coordination

## Boundary

- kernel enforces low-level mount and isolation primitives
- `sys/container/mounts` owns service-side mount semantics for the domain
