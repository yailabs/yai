# YAI Governance Glossary (Normative)

This glossary defines controlled normative vocabulary.

## Primary terms

### `core`
Sovereign runtime plane for authority, lifecycle, workspace sovereignty, dispatch, and enforcement baseline.

### `exec`
Execution plane for constrained effect realization and resource interaction.

### `brain`
Cognitive plane for planning, cognition, memory-driven proposal, and governed reconfiguration.

### `protocol`
Cross-cutting contract layer for wire/runtime interoperability.

### `platform`
Cross-cutting host abstraction layer (OS/FS/UDS/clock surfaces).

### `support`
Cross-cutting neutral primitive layer (ids, errors, logging, paths/strings, generic helpers).

### `authority`
Explicit sovereign decision legitimacy evaluated in `core`.

### `execution`
Governed committed transition path realized through `exec` after authority evaluation.

### `cognition`
Proposal and model-formation activity in `brain`; not sovereign authority.

## Historical terms

These are historical role labels and not primary normative ontology:
- `boot`
- `ingress`
- `root`
- `kernel`
- `engine`
- `mind`

When used, they MUST include primary mapping:
- `boot` -> startup stage alias under `core` lifecycle semantics
- `ingress` -> control-entry stage alias under `core` dispatch semantics
- `root` -> historical ingress authority alias under `core`
- `kernel` -> historical sovereign runtime alias under `core`
- `engine` -> historical execution alias under `exec`
- `mind` -> historical cognition alias under `brain`

## Deprecated terms / migration aliases

Deprecated-as-primary:
- `core boundary` (replace with `sovereign core boundary`)
- `exec boundary` (replace with `execution boundary`)
- `brain boundary` (replace with `cognitive boundary`)

Allowed usage:
- migration notes
- compatibility pointers
- formal artifact continuity where not yet realigned

Disallowed usage:
- primary normative domain naming
- authority model definition based on historical package names

## Ambiguity guard

Do not conflate:
- repo packaging names
- runtime role names
- normative concept names
- formal artifact labels

Normative decisions are governed by primary ontology terms.

## Canonical registry references

- `model/registry/primitives.v1.json`
- `model/registry/commands.v1.json`
- `model/registry/artifacts.v1.json`
