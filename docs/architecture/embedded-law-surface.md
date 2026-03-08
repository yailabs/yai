# Embedded Law Surface

`embedded/law/` is a generated runtime contract artifact derived from canonical `law` export manifests.

## Contains

- runtime manifests (`law.manifest.json`, `runtime.entrypoints.json`, publish/compat manifests)
- six-layer JSON payload (`classification`, `control-families`, `domain-specializations`, `overlays/*`)
- generated runtime views (`generated/*`)
- optional transitional bridge seed under `transitional/domain-family-seed/`

## Excludes

- broad editorial docs
- authoring prose
- formal/tooling internals
- full canonical repository narrative

## Policy

Embedded is the active runtime path.
Legacy mirror fallback is retired from active runtime and tooling paths.

## Not primary references

Historical docs and legacy mirror material must not be used as primary runtime contract reference.
Primary contract checks must resolve against `embedded/law`.
