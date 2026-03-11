# C7 Control Families De-Hardcoding Map

Old primary model:
- `control-families/<family>/manifest.json`
- `control-families/<family>/family.manifest.json`

New primary model:
- `control-families/descriptors/<family>.descriptor.v1.json`
- `control-families/index/families.descriptors.index.json`
- `control-families/index/family.matrix.v1.json`

Role split:
- descriptors/index/schema = canonical semantic model
- per-family folders = materialized views / compatibility surfaces
