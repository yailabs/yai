# D18 Execution Closeout

## Executed Structural Cutovers
- Applied D18 model/taxonomy/entry-surface/program compression/freeze contracts as repository mutations (not planning artifacts).
- Renamed ADR/RFC corpus to final grammar (`adr-###-*`, `rfc-###-*`) and repaired cross-doc/tool references.
- Removed `docs/program/milestone-packs/` from live root and moved historical corpus to `docs/program/archive/milestone-packs/`.
- Hardened live `docs/program/` shape to: `README`, `adr`, `rfc`, `policies`, `templates`, `reports`, `archive`.
- Added permanent surface-lock policies and validators (`validate_docs_surface_policy.py`, `validate_docs_freeze_contract.py`) and integrated them in standard validate/smoke runners.

## Main Rename/Move Waves
- ADR files renamed from domain-stacked form (`adr-<domain>-<nnn>-...`) to canonical short form (`adr-<nnn>-...`).
- RFC files renamed from domain-stacked form (`rfc-<domain>-<nnn>-...`) to canonical short form (`rfc-<nnn>-...`).
- `docs/program/milestone-packs/**` -> `docs/program/archive/milestone-packs/**`.
- `docs/program/templates/milestone-packs/**` -> `docs/program/archive/legacy/templates-milestone-packs/**`.

## Main Deletions/Collapses
- Program live root no longer hosts milestone-pack streams.
- Empty/obsolete live report shard directory (`docs/program/reports/audit-convergence/`) de-promoted from live reporting surface.
- Residual references to live milestone-pack path replaced with archive-path references.

## Validators and Governance Lock
- Added/updated policy contracts:
  - `docs/README_LIVE_SURFACE.md`
  - `docs/policies/live-docs-admission-policy.md`
  - `docs/policies/live-docs-naming-policy.md`
  - `docs/policies/docs-structure-policy.md`
  - `docs/policies/archive-transfer-policy.md`
- Added validators:
  - `tools/validate/validate_docs_surface_policy.py`
  - `tools/validate/validate_docs_freeze_contract.py`
- Updated existing validators to align with final live shape and grammar.

## Tree Before/After (Synthesized)
### Before (D18 input state)
- Program root still exposed milestone-pack family and delivery-history shape.
- ADR/RFC filenames still domain-stacked and less canonical.
- Live-surface lock policy/enforcement not fully unified under one constitutional contract.

### After (current)
``
docs
docs/architecture
docs/architecture/data-runtime
docs/architecture/distributed-runtime
docs/architecture/mesh/peering
docs/architecture/mesh/policy
docs/architecture/mesh/topology
docs/architecture/governance
docs/architecture/intelligence-runtime
docs/architecture/overview
docs/architecture/protocol
docs/architecture/protocol/control-plane
docs/architecture/protocol/source-plane
docs/architecture/protocol/transport
docs/architecture/runtime
docs/architecture/runtime/core
docs/architecture/runtime/data-sinks
docs/architecture/runtime/enforcement
docs/architecture/runtime/resolution
docs/architecture/system-theory
docs/architecture/workspace
docs/archive
docs/archive/migration
docs/generated
docs/guides
docs/guides/developer
docs/guides/developer/build-test
docs/guides/developer/checklists
docs/guides/developer/debugging
docs/guides/developer/governance
docs/guides/developer/onboarding
docs/guides/developer/operational-guides
docs/guides/developer/tooling
docs/guides/developer/workflow
docs/guides/operator
docs/guides/user
docs/guides/user/getting-started
docs/guides/user/guide
docs/policies
docs/product
docs/product/demos
docs/product/pre-pilot
docs/product/scenarios
docs/program
docs/program/adr
docs/program/archive
docs/program/archive/legacy
docs/program/archive/legacy/templates-milestone-packs
docs/program/archive/milestone-packs
docs/program/archive/milestone-packs/command-coverage
docs/program/archive/milestone-packs/contract-baseline-lock
docs/program/archive/milestone-packs/data-plane
docs/program/archive/milestone-packs/engine-attach
docs/program/archive/milestone-packs/root-hardening
docs/program/archive/milestone-packs/runtime-baselines
docs/program/archive/milestone-packs/specs-refactor-foundation
docs/program/archive/milestone-packs/unified-runtime-closeout
docs/program/archive/milestone-packs/workspace-verticalization-closeout
docs/program/archive/milestone-packs/workspaces-lifecycle
docs/program/archive/reports
docs/program/archive/reports/audit-convergence
docs/program/policies
docs/program/policies/security
docs/program/policies/style
docs/program/reports
docs/program/rfc
docs/program/templates
docs/program/templates/adr
docs/program/templates/rfc
docs/program/templates/runbooks
docs/reference
docs/reference/cli
docs/reference/commands
docs/reference/protocol
docs/reference/protocol/cli
docs/reference/protocol/compliance
docs/reference/protocol/contracts
docs/reference/protocol/control
docs/reference/protocol/protocol
docs/reference/protocol/providers
docs/reference/protocol/vault
docs/reference/registries
docs/reference/schemas
docs/reference/sdk
docs/runbooks
docs/runbooks/demos
docs/runbooks/operations
docs/runbooks/qualification
docs/runbooks/remediation

``

## Residual Minor Mismatches
- None blocking the canonical live-surface lock; full docs validator suite and link scan pass.
