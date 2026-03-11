# B8 Migration Marker

Status: complete (implementation cutover)

## Applied

- Ingestion content absorbed into `governance/ingestion/`.
- Deterministic parse/normalize/build pipeline moved to governance canonical paths.
- Governance review lifecycle validator moved in-repo and tied to canonical ingestion review state.
- Governance unit/integration tests now execute ingestion pipeline checks.

## Residual Compatibility

- Legacy `law-*` ingestion wrappers remain out-of-band and are no longer canonical.
