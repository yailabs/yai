# Domain Specializations

Canonical specialization model for domain verticalization.

Primary semantic surfaces:

- `index/specializations.index.json`
- `index/specializations.descriptors.index.json`
- `index/specialization.matrix.v1.json`
- `index/specialization-taxonomy.json`
- `index/subdomain-taxonomy.json`
- `index/scenario-classes.json`
- `descriptors/*.descriptor.v1.json`
- `schema/specialization-*.v1.schema.json`
- `templates/specialization.*.template.v1.json`

Materialized bundles live under `materialized/<specialization-id>/` and are
derived support surfaces. Canonical lookup and runtime routing is descriptor
and index driven.
