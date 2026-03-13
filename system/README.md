# L2 System Services Plane

`sys/` is the canonical L2 plane of YAI.
It hosts system services that run above `kernel/` primitives and below `user/` interfaces.

## Layer contract

- `kernel/`: privileged primitives, admission, containment, grants validity, registry and lifecycle roots.
- `sys/`: system service logic, lifecycle, recovery, and service-to-service coordination.
- `user/`: operator and developer-facing interfaces (CLI, shells, user libraries, tools).

`sys/` is not a renamed legacy runtime surface and is not a privileged kernel extension.

## Canonical service domains

- `init/`: system bootstrap and shutdown orchestration (`yai-init`, `yai-shutdown`).
- `supervisor/`: admission, registry, lifecycle and recovery control (`yai-supervisord`).
- `container/`: contained domain manager and runtime surfaces (`yai-containerd`).
- `daemon/`: daemon lifecycle and bindings management (`yai-daemond`).
- `orchestration/`: workflow planning, scheduling and execution coordination (`yai-orchestratord`).
- `policy/`: policy engine, grants overlays, enforcement and review (`yai-policyd`).
- `governance/`: authority, escalation, decisions and publication (`yai-governanced`).
- `graph/`: graph materialization, query, lineage and summary (`yai-graphd`).
- `data/`: records, evidence, retention, archive and stores (`yai-datad`).
- `network/`: discovery, topology, routing, transport, mesh and providers (`yai-netd`).
- `observability/`: metrics, audit, traces and reporting (`yai-metricsd`, `yai-auditd`).
- `services/`: shared service registry, sockets and manifests (cross-service substrate).

## Canonical root pattern

Each `sys/<domain>/` root should converge to:

- `README.md` with explicit ownership and boundaries.
- one canonical service entrypoint (`yai-...`) when applicable.
- coherent domain subdirectories (runtime, recovery, registry, etc.).
- no kernel-only privileged semantics.
- no user-facing UX/SDK ownership.

## Transitional policy

Legacy runtime code is allowed only as migration material in `runtime/compatibility/` and is not part of the default L2 canonical surface.
