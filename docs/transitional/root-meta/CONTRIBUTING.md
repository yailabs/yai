# Contributing

This document defines contribution standards for the `yai` runtime repository.

`yai` is an infrastructure-grade runtime: changes are evaluated on determinism, authority enforcement, and contract alignment. If a change introduces drift between implementation and contracts, it is treated as a defect.

## Scope and Sources of Truth

- Runtime implementation lives in this repository (`boot/`, `root/`, `kernel/`, `engine/`, `runtime/`, `mind/`).
- Canonical normative contracts live in `governance` (canonical) and `governance/runtime-package` (runtime contract surface) (pinned and verified by governance/CI).
- Documentation under `docs/` is operational guidance; contracts remain normative.

## Prerequisites

Required:
- C toolchain: `clang` (macOS) or `gcc/clang` (Linux)
- `make`
- Python 3 (used by verification and governance tooling)
- Rust toolchain (only if touching/building `mind/`)

Recommended:
- `gh` (GitHub CLI) for PR workflow and checks

## Repository Setup

Initialize pinned dependencies and build the runtime:
```bash
git submodule update --init --recursive
make build
```

If you need Mind (optional plane):
```bash
make mind
```

## Contribution Model

### Branching

Use focused branches. One branch = one intent.

Recommended naming: `feat/<topic>`, `fix/<topic>`, `docs/<topic>`, `chore/<topic>`, `ci/<topic>`, `release/<topic>`.

### Commits

Use conventional prefixes: `feat:`, `fix:`, `docs:`, `chore:`, `ci:`, `build:`, `refactor:`.

Keep commits scoped. Avoid mega commits.

## Pull Requests

### Required Metadata

PR bodies must follow the repository PR template and include all required metadata fields (Issue-ID, MP-ID, Runbook, Base-Commit, Classification, Compatibility, Evidence, Commands run). CI enforces this.

If you cannot link an issue, include an explicit `Issue-Reason:` explaining why.

### CI Expectations

PRs must be green unless explicitly declared otherwise and approved. Do not bypass gates by weakening checks to make CI pass.

### Evidence Standard

Evidence is not "I think it works". Evidence is: exact commands executed, expected outputs/artifacts, relevant logs when a gate fails, proof that contract pins and traceability remain intact.

## Verification Requirements

Run what matches your change surface.

**Runtime baseline (typical):**
```bash
make build
./tools/bin/yai-verify
```

**Full operational verification (when touching runtime behavior):**
```bash
./tools/ops/verify-core.sh
./tools/ops/verify-events.sh
```

**Mind (only if relevant):**
```bash
make mind
make mind-check
```

If a check fails, fix it or document the failure with rationale in the PR evidence section.

## Contract-Impacting Changes (Spec-First)

If your change affects any contract-facing surface (control/protocol/control/graph/vault/providers/compliance):

1. Update the contract in `governance` first (or via a tandem PR).
2. Provide versioning/compatibility rationale for the contract change.
3. Update the pinned reference in this repo (submodule/ref/pin policy).
4. Update runtime code to match the new contract.
5. Provide evidence via gates/tests that alignment holds.

Runtime-first changes that alter normative behavior are not accepted.

## Third-Party Code and Licensing

Do not introduce new external dependencies without documenting licensing impact. If vendoring code, keep upstream license headers intact and add/update `THIRD_PARTY_NOTICES.md`. Avoid copying code of unknown provenance.

## Data Contributions

This repository may contain non-sensitive datasets under `data/` for testing/evaluation.

Rules: no PII, credentials, secrets, or tokens; no runtime logs/events/state; provenance and intended usage must be declared for any dataset addition.

See `DATA.md`.

## Security-Sensitive Changes

If a change touches policy/authority/enforcement, control plane surfaces, workspace isolation, or provider gates/external effects — treat it as security-relevant. Provide explicit evidence and keep the change tightly scoped.

For reporting vulnerabilities, follow `SECURITY.md`.

## Pull Request Checklist

- [ ] PR metadata is complete and template placeholders are resolved.
- [ ] No contract/runtime drift introduced (spec-first adhered to where applicable).
- [ ] No new undeclared external dependency or vendored code without notices.
- [ ] No secrets, PII, runtime logs, or generated state committed.
- [ ] Relevant verification commands executed and recorded under "Commands run".
- [ ] Evidence includes positive and negative outcomes (when applicable).

## License

By contributing, you agree that your contributions are licensed under Apache-2.0 (see `LICENSE`).
