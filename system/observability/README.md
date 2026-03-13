# observability

`sys/observability/` is the L2 observability root.

## Canonical services

- `yai-metricsd/`
- `yai-auditd/`

## Responsibilities

- metrics collection and reporting
- audit trails and trace surfaces
- service health visibility for supervision and operations

## Boundaries

- observability does not own business/domain decisions
- observability consumes and publishes signals, it does not authorize effects
- kernel trace hooks remain kernel-owned primitives
