# YAI — Enterprise Governed Execution Platform

YAI is a governed execution platform for high-assurance AI/runtime operations with explicit authority, deterministic control surfaces, and auditable evidence.

## Platform posture

YAI is not a demo runtime.
It is the implementation and program center of a contract-governed platform stack.

Authority and dependency chain:

`yai-law` -> `yai-sdk` -> `yai-cli` -> `yai` -> `yai-ops`

## Repository role

This repository owns:
- Runtime implementation surfaces (Boot, Root, Kernel, Engine, Mind)
- Program governance artifacts (RFCs, ADRs, runbooks, milestone packs, validation plans)
- Platform integration logic constrained by pinned law contracts

This repository does not own:
- Cross-repo governance standards and reusable governance tooling (`yai-infra`)
- Normative law contracts (`yai-law`)

## Enterprise operating principles

- Authority-first execution
- Deterministic gates at effect boundaries
- Evidence and traceability as first-class artifacts
- Pinned contract consumption with explicit upgrade discipline
- Mind surfaces governed as part of platform architecture (scope-dependent in program phases)
- Workspace-first runtime operations with kernel-owned workspace state

## Program anchors

- Audit convergence matrix: `docs/program/audit-convergence/AUDIT-CONVERGENCE-MATRIX-v0.1.0.md`
- Audit convergence index: `docs/program/audit-convergence/README.md`
- Workspaces lifecycle MP index: `docs/program/24-milestone-packs/workspaces-lifecycle/README.md`
- Governance ADR: `docs/program/22-adr/ADR-012-audit-convergence-gates.md`

## Quick start

Build:

```bash
make build
make dist
```

Verify:

```bash
make verify
```

## Documentation entrypoints

- `docs/00-dashboard.md`
- `docs/README.md`

## Law and dependency pinning

This repository consumes canonical law as a pinned dependency:
- `deps/yai-law/`

It also tracks SDK pin alignment:
- `deps/yai-sdk.ref`

If implementation behavior diverges from pinned law contracts, the implementation must be corrected.

## License

Apache-2.0. See `LICENSE`, `NOTICE`, and `THIRD_PARTY_NOTICES.md`.
