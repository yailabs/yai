# C8 — Domain-Specializations De-Hardcoding Pass II

Status: implemented

## Cutover summary
- Canonical specialization model moved to descriptor/index surfaces.
- Added `specializations/descriptors/` as primary semantic layer.
- Added schemas and templates for descriptor-first authoring.
- Moved per-specialization subtree bundles under `specializations/materialized/`.
- Updated loaders, generators, validators, and unit coverage.

## Canonical source-of-truth
- `specializations/index/specializations.index.json`
- `specializations/index/specializations.descriptors.index.json`
- `specializations/index/specialization.matrix.v1.json`
- `specializations/descriptors/*.descriptor.v1.json`
