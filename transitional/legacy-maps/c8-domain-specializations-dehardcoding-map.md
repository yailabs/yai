# C8 — Domain Specializations Old->New Map

## Path migration
- `governance/specializations/<spec>/manifest.json`
  -> `governance/specializations/materialized/<spec>/manifest.json`
- `governance/specializations/<spec>/(model|policy|evidence|authority|discovery|scenarios)/*`
  -> `governance/specializations/materialized/<spec>/(model|policy|evidence|authority|discovery|scenarios)/*`

## Canonical semantic surfaces
- `governance/specializations/index/specializations.index.json`
- `governance/specializations/index/specializations.descriptors.index.json`
- `governance/specializations/index/specialization.matrix.v1.json`
- `governance/specializations/descriptors/*.descriptor.v1.json`

## Notes
- Materialized per-specialization bundles remain available as derived/runtime support surfaces.
- Runtime semantic resolution must read descriptors/index, not directory presence.
