# B6 Migration Marker

Status: complete (implementation cutover)

## Applied

- Canonical manifest spine imported into `governance/manifests`.
- Governance runtime manifest loader moved to canonical manifest surface reads.
- Compatibility checks enforce canonical manifest spine presence.
- Publish/index/layer/runtime manifest validations wired into governance unit suite.

## Remaining Legacy

- Embedded export/sync tooling remains transitional compatibility bridge.
- Removal of bridge-only paths is deferred to later cutover phases.
