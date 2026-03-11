# B6 Manifest And Publish Unification Map

This map tracks B6 cutover from legacy manifest loading to canonical
`governance/manifests` surfaces.

## Canonical Source Of Truth

- `governance/manifests/law.manifest.json`
- `governance/manifests/runtime.entrypoints.json`
- `governance/manifests/publish.index.json`
- `governance/manifests/publish.layers.json`
- `governance/manifests/compatibility.matrix.json`
- `governance/manifests/domain-resolution-order.json`
- `governance/manifests/compliance-resolution-order.json`
- `governance/manifests/governance-attachability.constraints.v1.json`
- `governance/manifests/customer-policy-packs.index.json`
- `governance/manifests/enterprise-custom-governance.index.json`

## Engine Cutover

- `lib/governance/loader/manifest_loader.c`
  - reads canonical manifest files via governance surface reader
  - canonicalizes runtime refs under `governance/`
- `lib/governance/loader/compatibility_check.c`
  - enforces canonical manifest spine presence

## Tooling And Tests

- `tools/validate/validate_governance_manifest_spine.py`
- `tests/unit/governance/test_governance_manifest_surface_contract.py`
- `tests/unit/governance/test_manifest_loader.c`
- `tests/unit/governance/test_contract_surface.c`

## Legacy Notes

- Embedded/export bridge remains compatibility-only.
- Canonical runtime behavior must resolve from `governance/manifests`.
