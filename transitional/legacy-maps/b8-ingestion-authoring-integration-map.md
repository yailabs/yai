# B8 Ingestion And Governance Authoring Integration Map

B8 makes `governance/ingestion/` the canonical authoring supply chain.

## Canonical Ingestion Pipeline

- `governance/ingestion/sources/`
- `governance/ingestion/parsed/`
- `governance/ingestion/normalized/`
- `governance/ingestion/candidates/`
- `governance/ingestion/review/`
- `governance/ingestion/templates/`
- `governance/ingestion/examples/`

## Engine/Tooling Cutover

- `tools/gen/deterministic_governance_ingestion.py`
  - grammar schema + ingestion roots moved to `governance/*`
  - deterministic repo-relative statement references
- `tools/bin/yai-govern`
  - source/parse/normalize/build/validate/review lifecycle
  - canonical `governance/ingestion/*` paths
- `tools/validate/validate_ingestion_cli.py`
- `tools/validate/validate_review_lifecycle.py`
- `tools/validate/validate_governance_ingestion_pipeline.py`

## Runtime/Validation Linkage

- `lib/governance/loader/compatibility_check.c`
  - requires canonical ingestion assets
- ingestion pipeline links with:
  - `governance/grammar/schema/*` ingestion schemas
  - `governance/manifests/runtime.entrypoints.json`
  - `governance/registry/governable-objects.v1.json`

## Tests

- `tests/unit/governance/run_governance_unit_tests.sh`
- `tests/integration/governance/test_ingestion_pipeline.sh`
- `tests/integration/governance/run_governance_resolution_smoke.sh`
- `tests/integration/workspace/workspace_agent_safe_boundaries_v1.sh`
