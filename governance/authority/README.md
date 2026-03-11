# authority/

`authority/` defines the operational governance authority layer of the unified
YAI repository.

If `foundation/`, `contracts/`, `registry/`, `schema/`, `formal/`, and `packs/`
define normative governance content, `authority/` defines how that content is
published, classified, evolved, and interpreted.

This directory is the meta-governance surface of the repository.

## Scope

Artifacts under `authority/` define rules for:

* publication
* status classification
* compatibility interpretation
* deprecation handling
* repository-level authority boundaries

These documents do not replace canonical governance content surfaces.
They govern how those surfaces are maintained and consumed.

## Role in the repository

`authority/` exists to answer questions such as:

* What is canonical?
* What is informative?
* What is deprecated?
* What requires compatibility review?
* What requires explicit publication discipline?
* How should downstream consumers interpret a change?

Without this layer, the repository would contain contracts but not a disciplined authority model.

## Relationship to the rest of the repository

`authority/` governs repository interpretation, but it does not originate the technical contracts themselves.

It is therefore upstream of repository policy, but not a substitute for:

* `foundation/` as normative core
* `contracts/` as public interface authority
* `registry/` and `schema/` as machine-readable canonical references
* `formal/` as proof-support and traceability support
* `packs/` as published overlays

## Contents

* `charter.md` — repository authority purpose and mandate
* `publication-policy.md` — rules for publication and canonical inclusion
* `compatibility-policy.md` — interpretation of compatibility impact
* `deprecation-policy.md` — rules for deprecating canonical artifacts
* `status-model.md` — status classes used across repository surfaces

## Consumer expectation

Consumers of governance policy content are expected to read `authority/`
together with:

* `README.md`
* `COMPATIBILITY.md`
* `VERSIONING.md`
* `REGISTRY.md`
* `SPEC_MAP.md`

Authority is not only what is published, but also how published artifacts are classified and evolved.
