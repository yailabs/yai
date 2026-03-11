# A6 Migration Marker - Governance Spine Destination Map

Purpose: lock the canonical destination map before full content absorption.

Canonical destination root: `governance/`

Top-level destination blocks:

- `authority/`
- `foundation/`
- `grammar/`
- `registry/`
- `classification/`
- `domains/`
- `families/`
- `specializations/`
- `compliance/`
- `overlays/`
- `manifests/`
- `contracts/`
- `schema/`
- `ingestion/`
- `packs/`
- `formal/`
- `vectors/`

Source families to absorb in Block B:

- external `yai-law` repository content
- `embedded/law` runtime-facing transitional payloads
- governance normative artifacts currently referenced from legacy docs

Boundary rule:

- `governance/` is canonical.
- `embedded/law` is transitional only.
