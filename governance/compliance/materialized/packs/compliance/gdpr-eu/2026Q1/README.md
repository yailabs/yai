# 2026Q1

`packs/compliance/gdpr-eu/2026Q1/` contains the published `2026Q1` GDPR/EU compliance pack for governance.

This versioned pack provides a scoped, machine-readable compliance overlay intended for deliberate downstream pinning and controlled adoption.

## Scope

This pack version contains:

* `pack.meta.json` — pack identity, scope, and metadata
* `retention.defaults.json` — default retention posture for the declared compliance scope
* `taxonomy.data_classes.json` — canonical data-class taxonomy for the pack scope
* `taxonomy.legal_basis.json` — legal-basis taxonomy for the pack scope
* `taxonomy.purposes.json` — purpose taxonomy for the pack scope

These artifacts are intended to support compliance-aware validation, policy interpretation, and governed downstream use.

## Normative role

This pack is normative within its declared scope.

It must remain aligned with:

* `foundation/extensions/compliance/`
* `foundation/invariants/I-006-external-effect-boundary.md`
* `foundation/invariants/I-007-compliance-context-required.md`
* `schema/compliance.context.v1.json`
* `schema/retention.policy.v1.json`
* `contracts/compliance/`
* `contracts/control/` where authority-facing enforcement is relevant

This pack does not replace the foundational governance, canonical schemas, or canonical registries of the repository.
It specializes them for a bounded GDPR/EU compliance context.

## Consumer expectation

Consumers should treat this version as a pinned compliance artifact set.

Expected discipline includes:

* pinning the exact pack version
* validating contents before adoption
* reviewing compatibility and policy impact before upgrade
* treating pack material as a scoped overlay rather than a standalone authority source

## Change discipline

Any change to this pack version that affects retention defaults, taxonomy meaning, or compliance interpretation must update, as applicable:

* the pack artifacts in this directory
* `CHANGELOG.md`
* `SPEC_MAP.md`
* `REGISTRY.md` where canonical references are affected
* relevant compatibility or publication notes when scope changes

Silent drift between this pack and the canonical governance surfaces it depends on is non-compliant by definition.
