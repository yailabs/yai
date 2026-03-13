# container

`sys/container/` is the canonical L2 container plane of YAI.

It owns the system-service runtime management of governed execution domains.
The legacy workspace model is migration-only and not a canonical domain center.

## Canonical service entrypoint

- `yai-containerd/`

## Plane responsibilities

- container identity/config/state/lifecycle management
- container runtime service surface publication
- root projection coordination (`rootfs/`) and mount coordination (`mounts/`)
- session-domain binding coordination (`session/`)
- container recovery orchestration (`recovery/`)
- container registry and domain health/readiness synthesis
- image/layout coordination for container bootstrap (`image/`)

## Subdomain roles

- `runtime/`: core L2 container domain model and runtime views
- `rootfs/`: projected operational root, path/tree model, root-to-backing relation
- `mounts/`: governed mount and attach surface for container domains
- `session/`: container-bound session attach/bind/rebind logic
- `recovery/`: domain recovery, reopen, reconcile and seal handoff
- `image/`: base image/layout semantics and bootstrap template contract
- `yai-containerd/`: service process entrypoint of the container plane

## Boundary with `kernel/container/`

- `kernel/container/*` owns low-level primitives only: namespaces, rootfs/mount primitives, limits and isolation hooks.
- `sys/container/*` owns domain lifecycle and operational management above those primitives.
- kernel makes containment possible; `sys/container` makes the domain manageable and live.

## Boundary with upper layers

- CLI, orchestration and daemon layers consume container surfaces.
- CLI, orchestration and daemon do not replace or bypass `yai-containerd`.
