# B1 Governance Content Import Map (`yai-law` -> `yai`)

Owner: `yai`
Status: ready for execution (mapping baseline)
Scope: governance content absorption planning for B2-B8

## Purpose

Define explicit destination for each `yai-law` macro-area before content migration,
so absorption executes by controlled tranche and not by ad-hoc copy.

Rule of truth:

- Canonical policy-engine content -> `governance/`
- Central architecture/reference documentation -> `docs/`
- Historical/compatibility/migration-only material -> `transitional/`

## Canonical Import Matrix

| `yai-law` source area | `yai` destination | Content type | Landing state | Notes |
|---|---|---|---|---|
| `authority/` | `governance/authority/` | authority model, roles, scopes | canonical | B2 primary intake |
| `foundation/` | `governance/foundation/` | axioms, invariants, boundaries, terminology | canonical | B2 primary intake |
| `grammar/` | `governance/grammar/` | grammar rules, semantics, grammar schema | canonical | B3 primary intake |
| `registry/` | `governance/registry/` | ids, governance registry indexes | canonical | B3 primary intake |
| `schema/` | `governance/schema/` | governance schemas | canonical | B7 primary intake |
| `classification/` | `governance/classification/` | domain/family classification surfaces | canonical | B4 primary intake |
| `domains/` | `governance/domains/` | domain definitions and shared domain context | canonical | B4 primary intake |
| `control-families/` | `governance/control-families/` | family-level policy content | canonical | B4 primary intake |
| `domain-specializations/` | `governance/domain-specializations/` | specialization-level policy content | canonical | B4 primary intake |
| `compliance/` | `governance/compliance/` | compliance overlays, controls, obligations | canonical | B5 primary intake |
| `overlays/` | `governance/overlays/` | regulatory/sector/context overlays | canonical | B5 primary intake |
| `packs/` | `governance/packs/` | governance/compliance packs | canonical | B5/B6 tie-in |
| `manifests/` | `governance/manifests/` | publish/runtime manifests | canonical | B6 primary intake |
| `contracts/` | `governance/contracts/` | control/protocol/provider/vault contracts | canonical | B7 primary intake |
| `formal/` | `governance/formal/` | formal artifacts, configs, schemas | canonical | B6/B7 tie-in |
| `vectors/` | `governance/vectors/` | vectorized governance artifacts | canonical | B6/B8 tie-in |
| `ingestion/` | `governance/ingestion/` | authoring, normalization, review ingestion pipeline | canonical | B8 primary intake |

## README Import Rules

For README and docs-like files from `yai-law`:

1. Structural README close to canonical governance content
- destination: same area under `governance/`
- role: local structure/operator guide for that governance subtree

2. Architecture/reference narrative with cross-domain scope
- destination: `docs/`
- role: central architecture/reference documentation

3. Historical, compatibility-only, migration notes
- destination: `transitional/`
- preferred locations:
  - `transitional/legacy-docs/`
  - `transitional/legacy-maps/`
  - `transitional/migration-markers/`

## Import Rules by Artifact Type

- manifests -> `governance/manifests/`
- schema -> `governance/schema/`
- contracts -> `governance/contracts/`
- overlays -> `governance/overlays/`
- compliance -> `governance/compliance/`

These classes are non-interchangeable in migration placement.

## Transitional and Non-Canonical Material

The following stays outside canonical governance content:

- embedded export/sync historical maps
- repo-sync notes and cutover checklists
- compatibility fallback inventories
- old-to-new relocation maps

Landing area:

- `transitional/legacy-maps/`
- `transitional/embedded-law/`
- `transitional/migration-markers/`

## Embedded Constraint

From B1 onward:

- no new canonical content is introduced under `embedded/law`
- `embedded/law` is compatibility/export surface only until B10 removal execution
- canonical destination for imported governance content is always `governance/`

## Block B Execution Hooks

- B2: import `authority/` and `foundation/`
- B3: import `grammar/` and `registry/`
- B4: import `classification/`, `domains/`, `control-families/`, `domain-specializations/`
- B5: import `compliance/`, `overlays/`, `packs/`
- B6: unify `manifests/` (+ formal/vector publish ties)
- B7: unify `contracts/` and `schema/`
- B8: integrate `ingestion/` authoring surfaces

## Definition of Done Checkpoints (B1)

- every `yai-law` macro-area has a single explicit target destination
- destination class is explicit (`canonical`, `docs`, `transitional`)
- README handling rules are explicit
- manifest/schema/contracts separation is explicit
- compliance/overlay separation is explicit
