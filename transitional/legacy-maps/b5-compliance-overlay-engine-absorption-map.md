# B5 Compliance and Overlay Engine Absorption Map

Scope: absorb `../law/compliance`, `../law/overlays`, `../law/packs` and cut over
governance engine runtime loading to canonical `governance/` paths.

## Canonical content absorption

- `../law/compliance/*` -> `governance/compliance/*`
- `../law/overlays/*` -> `governance/overlays/*`
- `../law/packs/*` -> `governance/packs/*`

## Engine cutover implemented

- canonical-first reader introduced:
  - `lib/governance/loader/law_loader.c`
  - `yai_law_read_governance_surface_file(rt, rel_path, ...)`
- compliance loading now canonical-first:
  - `lib/governance/loader/compliance_loader.c`
  - `compliance/index/compliance.index.json` preferred
- overlay loader validation now canonical-first:
  - `lib/governance/loader/overlay_loader.c`
  - validates regulatory/sector attachment matrices + contextual index
- compatibility check now includes canonical compliance/overlay surfaces and pack metadata:
  - `lib/governance/loader/compatibility_check.c`
- overlay/compliance matrix reads in resolver stack use canonical-first reader:
  - `lib/governance/resolution/stack_builder.c`

## Canonical-only runtime mode

- env `YAI_GOVERNANCE_CANONICAL_ONLY=1` forces canonical governance paths
  (no fallback to embedded/legacy paths)
- governance unit and integration suites run with canonical-only mode enabled

## Tooling updates

- generator: `tools/gen/build_overlay_compliance_runtime_view.py`
- validator: `tools/validate/validate_overlay_compliance_runtime_view.py`
- generated view: `governance/overlays/index/overlay-compliance.runtime.v1.json`

## Runtime-test evidence

- `tests/unit/governance/run_governance_unit_tests.sh` -> PASS
- `tests/integration/governance/run_governance_resolution_smoke.sh` -> PASS
