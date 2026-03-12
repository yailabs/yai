# registry/

`registry/` contains canonical machine-readable governance registries for the
unified YAI system.

## Ontology policy

Registry metadata must reflect primary ontology:
- `core`
- `exec`
- `data`
- `graph`
- `knowledge`
- `protocol/platform/support` as cross-cutting layers

Historical labels may remain only as compatibility metadata or migration aliases.
They must not be interpreted as primary runtime ontology.

## Scope

- `primitives.v1.json`
- `agent-safe-primitives.v1.json`
- `commands.v1.json`
- `commands.surface.v1.json`
- `commands.topics.v1.json`
- `artifacts.v1.json`
- `governable-objects.v1.json`
- `ids/*`
- `schema/*.v1.schema.json`

## Compatibility

Command IDs and existing schema lines are kept stable unless explicit breaking migration is approved.
Semantic interpretation can be realigned without renaming stable public IDs.


Unified governable objects include:

- `customer_policy_pack`
- `enterprise_custom_governance_object`
- `overlay`
- `compliance_module`
- `specialization_policy_pack`
- `template_object`
