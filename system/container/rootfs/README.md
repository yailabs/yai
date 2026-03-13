# Container Root Projection Model

`sys/container/rootfs/` owns the container projected-root model at L2.

## Responsibilities

- define and apply projected operational root semantics
- keep root tree/path resolution container-scoped by default
- coordinate root projection with kernel rootfs/mount primitives
- maintain the distinction between:
  - host filesystem
  - container backing material
  - projected operational root

## Implemented surfaces

- `root_projection.c`: projected root construction and status
- `paths.c`: container-relative path context and resolution
- `tree.c`: container tree structure and visibility helpers

## Boundary

- kernel enforces low-level rootfs/mount isolation primitives
- `sys/container/rootfs` defines how a container domain lives on top of them
