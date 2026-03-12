# Control Families

Canonical operational family model.

Families are governed primarily through:

- `index/families.index.json`
- `index/families.descriptors.index.json`
- `index/family.matrix.v1.json`
- `index/family-hierarchy.json`
- `index/family-naming-registry.json`
- `descriptors/*.descriptor.v1.json`
- `schema/family-*.v1.schema.json`

Per-family folders remain supported for materialized manifests, but index
and descriptor surfaces are the primary semantic contract consumed by runtime logic.
