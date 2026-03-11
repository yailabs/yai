# gdpr-eu/

`packs/compliance/gdpr-eu/` contains the GDPR/EU compliance pack lineage published under `law`.

This lineage groups versioned machine-readable overlays intended to specialize the canonical YAI law surface for GDPR- and EU-aligned compliance contexts.

## Scope

Artifacts under `gdpr-eu/` are intended to provide scoped compliance material such as:

* pack metadata
* retention defaults
* compliance taxonomies
* legal-basis classification support
* purpose classification support
* data-class classification support

These artifacts are publishable overlays, not replacements for the foundational law of the repository.

## Normative role

The GDPR/EU lineage is normative within its declared scope.

It must remain aligned with:

* `foundation/extensions/compliance/`
* `foundation/invariants/I-006-external-effect-boundary.md`
* `foundation/invariants/I-007-compliance-context-required.md`
* `schema/compliance.context.v1.json`
* `schema/retention.policy.v1.json`
* `contracts/compliance/`
* `contracts/control/` where authority-facing compliance enforcement is relevant

A lineage artifact may refine GDPR/EU-specific behavior for a bounded compliance context, but it must not contradict the canonical law surface of `law`.

## Versioning model

Each child version directory represents a versioned publishable lineage state.

Typical contents include:

* `pack.meta.json`
* `retention.defaults.json`
* taxonomy artifacts for compliance classification

Version directories are intended to be pinned and consumed deliberately by downstream repositories or workflows.

## Current published version

Current published version:

* `2026Q1/`

## Consumer expectation

Consumers should:

* pin a specific lineage version
* validate pack contents before adoption
* treat lineage artifacts as scoped overlays
* avoid assuming that a lineage pack supersedes repository-wide authority

The foundational law, canonical schemas, and canonical registries remain the higher-authority surfaces unless an explicit repository rule states otherwise.
