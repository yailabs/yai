# Law Consumption Model

## Contract boundary

- Canonical normative source: `law` repository.
- Runtime contract artifact: `embedded/law/`.

## Active six-layer consumption

`yai` consumes embedded payloads by layer:
- `classification/`
- `control-families/`
- `specializations/`
- `overlays/regulatory/`
- `overlays/sector/`
- `overlays/contextual/`

Runtime loaders validate layer indexes and generated summaries before resolution.

## Surface classes

- `active runtime-facing`: `embedded/law`
- `canonical source`: sibling `law` repository
- `bridge/transitional tolerated`: optional seed payload (export opt-in only)
- `historical/reference-only`: legacy debug/report docs
