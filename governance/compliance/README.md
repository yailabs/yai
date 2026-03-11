# compliance

Canonical compliance model for the unified governance engine.

## Current role

- Source manifests and mappings are canonical here.
- Runtime composition is resolved with overlay indexes/matrices plus compliance index metadata.
- Compliance is exposed as governable object catalog entries via:
  `registry/governable-objects.v1.json`.

## Read first

1. `index/compliance.index.json`
2. `index/applicability.matrix.json`
3. `index/domain-crosswalk.json`
4. `index/authority-crosswalk.json`
5. `overlays/regulatory/index/regulatory.index.json`
6. `docs/architecture/governance-layers/compliance-layer-model.md`

## Operational distinction

- Operational manifests: `*/manifest.json` and `sector-overlays/*/manifest.json`
- Index surfaces: `index/*`
- Pack materialization: `governance/packs/compliance/*`

## Do not confuse

- Compliance manifests are not the same as overlay stack indexes (`overlays/*/index/`).
- Templates/examples are never attachable runtime objects unless listed as operational in:
  `registry/governable-objects.v1.json`.
- Customer/workspace attachment must use object IDs from `registry/governable-objects.v1.json`
  through CLI surface (`yai ws policy attach|detach`) and not direct path references.
