# Domains

Canonical schema/index-driven domain model root.

`domains/` is not a passive compatibility bucket. It is the canonical place for
the domain model used by governance loader/discovery/resolution surfaces.

Primary surfaces:

- `index/domains.index.json`
- `index/discovery-matrix.json`
- `index/domain-hierarchy.json`
- `index/domain-families.json`
- `index/domain-model.matrix.v1.json`
- `index/domain-model.matrix.v1.schema.json`
- shared taxonomies under `shared/`

Model rule:

- filesystem folders can materialize payloads
- semantic truth is carried by indexes, matrices, and descriptors
