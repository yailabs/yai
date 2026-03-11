# C7 — Control Families De-Hardcoding Pass I

State:
- Canonical model moved to descriptor/index surfaces under `governance/control-families/`.
- Runtime/loader now supports index-driven control-family descriptor loading.

Primary semantic surfaces:
- `control-families/index/families.descriptors.index.json`
- `control-families/index/family.matrix.v1.json`
- `control-families/descriptors/*.descriptor.v1.json`
- `control-families/schema/*.v1.schema.json`

Compatibility:
- Per-family `control-families/<family>/manifest.json` remains materialized and secondary.
- `manifest_ref` stays present in `families.index.json` for transitional consumers.
