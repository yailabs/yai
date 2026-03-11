# B7 Contracts And Schema Unification Map

B7 unifies contracts and structural schema under canonical governance paths and
connects runtime/tooling consumers to those paths.

## Canonical Contract Surfaces

- `governance/contracts/control/`
- `governance/contracts/protocol/`
- `governance/contracts/vault/`
- `governance/contracts/providers/`
- `governance/contracts/compliance/`
- `governance/contracts/cli/`

## Canonical Structural Schemas

- `governance/schema/policy.v1.schema.json`
- `governance/schema/decision_record.v1.schema.json`
- `governance/schema/governance_review_state.v1.schema.json`
- `governance/schema/evidence_index.v1.schema.json`
- `governance/schema/workspace_governance_attachment.v1.schema.json`
- `governance/schema/retention_policy.v1.schema.json`
- `governance/schema/containment_metrics.v1.schema.json`
- `governance/schema/verification_report.v1.schema.json`

## Engine/Tooling Cutover

- `lib/governance/loader/compatibility_check.c`
  - requires canonical contracts and schema presence
- `include/yai/governance/loader.h`
- `lib/governance/loader/law_loader.c`
  - public `yai_law_read_surface_json` for runtime/tooling consumers
- `tools/validate/validate_governance_contracts_schema.py`
- `tools/dev/gen-vault-abi`
- `tools/dev/check-generated.sh`
- `tools/bin/yai-law-compat-check`

## Validation And Tests

- `tests/unit/governance/test_contracts_schema_loader.c`
- `tests/unit/governance/run_governance_unit_tests.sh`

Legacy law-root contract/schema lookups remain compatibility fallback only.
