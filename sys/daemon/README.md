# daemon

sys/daemon/ is the canonical L2 daemon plane.

## Canonical service

- yai-daemond/: executable daemon manager entrypoint (`yai-daemond`)

## Canonical module layout

- bindings/: runtime/network binding logic and action-point modeling
- mediation/: daemon mediation surfaces
- replay/: spool/replay process surfaces
- health/: observation and health surfaces
- internal/: daemon technical internals
- include/yai/daemon/: canonical public headers

## DR-5 hard-cut status

- active daemon implementation relocated from runtime/compatibility/lib/daemon/
- compatibility daemon subtree no longer hosts primary implementation
- no parallel compatibility-centered daemon plane remains active
