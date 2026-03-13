# Container Image and Bootstrap Layout

`sys/container/image/` defines the base image/layout semantics for container domains.

## Responsibilities

- define base image/reference semantics used by container creation
- define initial domain layout templates consumed by `yai-containerd`
- separate bootstrap image concerns from runtime mutable state

## Boundary

- this is an L2 service-domain contract, not a kernel image subsystem
- final storage-driver/image-format depth is staged in later waves
