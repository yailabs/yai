# YAI Documentation — Platform and Program Entrypoint

This directory is the primary documentation entrypoint for the YAI platform implementation repository.

## Start here

- Platform architecture and runtime model: `docs/platform/`
- Developer guides and workflows: `docs/developer/`
- Interface pointers to pinned law contracts: `docs/interfaces/`
- Program governance (ADR/RFC/runbooks/MP): `docs/program/`
- Cross-repo pointers and supporting references: `docs/pointers/`

## Program-critical anchors

- Audit convergence: `docs/program/audit-convergence/`
- Workspaces lifecycle packs: `docs/program/24-milestone-packs/workspaces-lifecycle/`
- Audit-convergence governance ADR: `docs/program/22-adr/ADR-012-audit-convergence-gates.md`

## Cross-repo alignment notes

- Normative contract authority is pinned from `deps/yai-law/`.
- Operational evidence bundles and field collateral are maintained in `yai-ops`.
- CLI and SDK command/control surfaces are maintained in `yai-cli` and `yai-sdk`.

## Interpretation rule

Docs in this repository are implementation/program-facing.
Normative contract disputes are resolved against pinned `yai-law` artifacts.
