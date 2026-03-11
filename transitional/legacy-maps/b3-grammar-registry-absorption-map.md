# B3 Grammar and Registry Absorption Map

Scope: absorb `../law/grammar` and `../law/registry` into canonical
`governance/grammar` and `governance/registry`.

## Source -> Destination (Grammar)

- `../law/grammar/rules/*` -> `governance/grammar/rules/*`
- `../law/grammar/semantics/*` -> `governance/grammar/semantics/*`
- `../law/grammar/schema/*` -> `governance/grammar/schema/*`
- `../law/grammar/README.md` -> `governance/grammar/README.md` (normalized)

## Source -> Destination (Registry)

- `../law/registry/primitives.v1.json` -> `governance/registry/primitives.v1.json`
- `../law/registry/agent-safe-primitives.v1.json` -> `governance/registry/agent-safe-primitives.v1.json`
- `../law/registry/governable-objects.v1.json` -> `governance/registry/governable-objects.v1.json`
- `../law/registry/artifacts.v1.json` -> `governance/registry/artifacts.v1.json`
- `../law/registry/commands.v1.json` -> `governance/registry/commands.v1.json`
- `../law/registry/commands.surface.v1.json` -> `governance/registry/commands.surface.v1.json`
- `../law/registry/commands.topics.v1.json` -> `governance/registry/commands.topics.v1.json`
- `../law/registry/ids/*` -> `governance/registry/ids/*`
- `../law/registry/schema/*` -> `governance/registry/schema/*`
- `../law/registry/README.md` -> `governance/registry/README.md` (normalized)

## Landing classification

- all imported grammar/registry artifacts: canonical governance content
- migration traces and cutover notes: `transitional/migration-markers/`

## Normalization applied

- grammar/registry readmes updated to unified-system framing
- ids README upgraded from placeholder to usable identifier guidance
- domains semantics path updated to `governance/domains`

## Compatibility note

Schema `$id` and legacy-compatible naming inside JSON artifacts are retained in
B3 unless explicitly changed by a dedicated compatibility tranche.
