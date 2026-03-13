# yai-containerd

`yai-containerd` is the canonical userspace system service for container domain management.

## Role

Entrypoint process of the `sys/container` L2 plane.

## Responsibilities

- create/open/attach/initialize/recover/seal/destroy container domains
- coordinate runtime/rootfs/mounts/session/recovery/image subdomains
- publish container runtime/service surfaces for other layers
- mediate container operations through kernel-owned primitives

## Non-responsibilities

- kernel privileged containment/admission/grants roots
- high-level policy composition/governance/compliance engines
- orchestration planning ownership
- daemon control-plane ownership
