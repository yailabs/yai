# Changelog

All notable changes to this project will be documented in this file.

The format is based on Keep a Changelog,
and this project adheres to Semantic Versioning.

## [Unreleased]


### Added
- mind: import L3 Mind plane under /mind as workspace member; add lockfile + Makefile targets; stage yai-mind into build/bin (not in default bundle)

- Introduced `yai-changelog-check` with PR-mode and tag-mode validation for changelog quality gates.
- Added CI workflow `validate-changelog.yml` to enforce incremental changelog policy on pull requests.

### Changed

- kernel: implemented real `yai.kernel.ws` runtime actions (`create|reset|destroy`) on `control.call`, creating/removing workspace layout under `~/.yai/run/{workspace_id}/` with manifest generation.
- core: split command dispatch surfaces into dedicated `core/commands` modules for kernel, root, and engine to keep runtime core paths focused on process/session orchestration.
- release: align `deps/yai-law` and `deps/yai-cli.ref` to yai-law/yai-cli main heads for strict pin gates.
- core: make workspace id validation portable in C11 by removing `strnlen` dependency in `root/include/ws_id.h`.

- Aligned dependency and docs/tooling naming from `yai-specs` to `yai-law` in `yai` (canonical `deps/yai-law`, compatibility alias, sync wrappers, governance suite pinned to `gov-suite-v0.1.0`).
- Consolidated Mind governance/docs paths after mind-integration cutover and aligned architecture component metadata for validation gates.
- Closed Wave0 strict pin blocker by aligning `yai` + `yai-cli` to `yai-specs@30d04d0`, updating `deps/yai-cli.ref`, and refreshing closure evidence/MP snapshots.
- Hardening wave 1: removed non-core local Python test/cache artifacts from `tools/python/yai_tools` (tracked in yai-infra#17).
- Start infra cutover increment: add reusable `verify` workflow via `yai-infra.1.0-rc1`, migrate governance guide pointer, and add `scripts/yai-govern` compatibility wrapper.
- Release workflow now runs strict changelog validation in tag mode before publishing bundle assets.
- `tools/release/bump_version.sh` no longer injects automatic placeholder entries.
- Added proof-pack governance guardrails (`yai-proof-check`, `validate-proof-pack.yml`, `proof-verify`, `release-guards`, `release-guards-dev`) and synchronized branch helper (`yai-dev-branch-sync`).
- Aligned cross-repo pins for `yai-specs` and `yai-cli` on branch `meta/governance-proof-pack-lock` with canonical proof manifest updates.
- Strengthened docs traceability chain by linking proposal -> ADR -> runbook -> milestone pack with explicit evidence pointers.
- Added canonical agent governance guidance and moved draft TRL audits to a private local folder policy.
- Formalized the GitHub PMO governance model for runbook/MP execution, including cross-repo milestone closure rules and Project v2 operating standards.

## [0.1.7] - 2026-02-17

- TODO: summarize release changes.

## [0.1.6] - 2026-02-17

- TODO: summarize release changes.

## [0.1.5] - 2026-02-17

- TODO: summarize release changes.

## [0.1.4] - 2026-02-17

- TODO: summarize release changes.

## [0.1.3] - 2026-02-17

- TODO: summarize release changes.

## [0.1.2] - 2026-02-17

- TODO: summarize release changes.

## [0.1.1] - 2026-02-17

- TODO: summarize release changes.

- No unreleased changes.

## [0.1.0] - 2026-02-17

### Added

- Canonical `build/ -> dist/ -> bundle/` pipeline with reproducible release assets.
- Release bundle artifacts (`tar.gz`, `zip`, `manifest`, `SHA256SUMS`) and CI workflow.

### Changed

- Repository legal/governance hardening for public release readiness.
- Runtime build outputs standardized under `build/bin`.

### Security

- Public disclosure process and hardening checklist documented.

## License

This changelog is part of the Apache-2.0 licensed repository. See `LICENSE` and `NOTICE`.
