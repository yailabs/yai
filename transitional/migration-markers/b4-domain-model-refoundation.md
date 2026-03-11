# B4 Marker — Domain Model Refoundation and Absorption

Status: completed

Implemented outcomes:

- absorbed `classification/`, `domains/`, `control-families/`, `domain-specializations/`
  into canonical `governance/` tree
- introduced schema/index-driven domain model matrix
- wired loader/discovery/resolution/mapping to read matrix for runtime decisions
- added generator + validator and executed them in governance unit test flow
- preserved compatibility fallback paths without keeping them as semantic center

Primary runtime evidence:

- `tests/unit/governance/run_governance_unit_tests.sh` -> PASS
- `tests/integration/governance/run_governance_resolution_smoke.sh` -> PASS

Traceability map:

- `transitional/legacy-maps/b4-domain-model-refoundation-map.md`
