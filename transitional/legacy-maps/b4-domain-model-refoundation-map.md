# B4 Domain Model Refoundation Map

Scope: absorb and refound classification/domains/control-families/domain-specializations
as schema-first, index-driven runtime model.

## Canonical content absorption

- `../law/classification/*` -> `governance/classification/*`
- `../law/domains/*` -> `governance/domains/*`
- `../law/control-families/*` -> `governance/control-families/*`
- `../law/domain-specializations/*` -> `governance/domain-specializations/*`

## New canonical semantic spine

- `governance/domains/index/domain-model.matrix.v1.json`
- `governance/domains/index/domain-model.matrix.v1.schema.json`

These become canonical runtime lookup surfaces for:

- family -> domain resolution
- family -> specialization candidate/default resolution
- lookup-id -> manifest resolution (family/domain/specialization)

## Engine cutover implemented

- `lib/governance/loader/domain_model_matrix.c`
  - matrix loading
  - lookup-id resolution
  - runtime family listing
  - family specialization/default resolution
- `lib/governance/loader/domain_loader.c`
  - domain manifest lookup through matrix (index-driven first)
- `lib/governance/discovery/domain_discovery.c`
  - runtime families from matrix
  - family->domain mapping from matrix
  - specialization candidate/default from matrix
- `lib/governance/resolution/domain_merge.c`
  - domain/family/default-specialization reconciliation through matrix
- `lib/governance/resolution/stack_builder.c`
  - specialization manifest ref resolved via matrix lookup
- `lib/governance/mapping/domain_to_policy.c`
  - domain -> family bridge via matrix lookup before policy mapping

## Tooling cutover implemented

- generator: `tools/gen/build_domain_model_matrix.py`
- validator: `tools/validate/validate_domain_model_matrix.py`
- governance unit suite runs generator+validator before C tests

## Legacy pattern reduction

Reduced hardcoded runtime assumptions:

- static family->domain hardcodes (`digital/economic/scientific` -> `D1/D5/D8`)
- fixed in-code specialization candidate lists as primary source of truth
- direct specialization manifest path construction as only resolution path

Retained as fallback only where needed for compatibility.
