# Grammar Layer

`grammar/` defines the normative governance language of the unified YAI system.

It contains schema, semantics, and composition rules used by
domains/compliance/manifests.
It does not host domain-specific policy payloads.

Structure:

- `rules/`: composition, precedence, override, fallback, conflict and stacking rules
- `semantics/`: normative interpretation of discovery, overlays, evidence and effects
- `schema/`: grammar schemas consumed by governance runtime/tooling surfaces

Customer policy pack contract schema:

- `schema/customer_policy_pack.v1.schema.json`
