# A6 - Governance Placeholder Spine

This document fixes the canonical governance destination topology in `yai`
before Block B content absorption.

## Canonical governance topology

`governance/` contains the official placeholder spine for:

- `authority/`
- `foundation/`
- `grammar/`
- `registry/`
- `classification/`
- `domains/`
- `control-families/`
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

## Structural rule

- `governance/` is native and canonical.
- `lib/governance/` is runtime implementation.
- `include/yai/governance/` is public API surface.
- `embedded/law` is transitional compatibility only.

## Block B readiness

The A6 spine is the stable destination map for:

- authority/foundation absorption
- grammar/registry/schema absorption
- compliance/overlays/manifests absorption
- contracts/ingestion/formal/vectors absorption
