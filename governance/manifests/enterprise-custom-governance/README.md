# Enterprise Custom Governance Objects

Canonical surfaces:

- index: `manifests/enterprise-custom-governance.index.json`
- schema: `grammar/schema/enterprise_custom_governance.v1.schema.json`
- examples: `manifests/enterprise-custom-governance/examples/`
- templates: `manifests/enterprise-custom-governance/templates/`

Lifecycle distinction:

- `template` / `example`: authoring-only, non-operational
- `candidate`: review state, not yet approved for runtime consumption
- `approved`: accepted enterprise governance object
- `applied`: approved and currently attached/active
- `deprecated`: discoverable for migration/audit only
