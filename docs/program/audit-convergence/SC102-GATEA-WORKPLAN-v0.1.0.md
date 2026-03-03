---
id: SC102-GATEA-WORKPLAN-v0.1.0
status: active
owner: program-delivery
effective_date: 2026-02-24
revision: 2
issue:
  - https://github.com/yai-labs/yai/issues/183
  - https://github.com/yai-labs/yai/issues/184
  - https://github.com/yai-labs/yai/issues/185
  - https://github.com/yai-labs/yai/issues/186
related:
  - docs/30-catalog/scenarios/SC-102.md
  - docs/30-catalog/domains/packs/D1-digital/egress-v1/pack.meta.json
  - docs/40-qualification/QT-0.1-001-SC102/README.md
  - docs/50-validation/audits/claims/infra-grammar.v0.1.json
  - docs/20-program/audit-convergence/AUDIT-CONVERGENCE-MATRIX-v0.1.0.md
---

# SC-102 Gate A Workplan (v0.1.0)

This is the operational workplan to move Gate A from partial to green using:
- scenario: `SC-102`
- first domain pack: `D1-digital/egress-v1`
- qualification gate: `QT-0.1-001-SC102`

## 1) Fixed Scope

In scope:
- Core gate only (A-core)
- one pack first (`D1-digital/egress-v1`)
- claim closure with reproducible evidence

Out of scope:
- SC-103 and Mind integration (Gate B)
- cross-domain D2..D9 execution (planned after D1 green)

## 2) Entry Checklist

1. Program anchors reviewed:
   - `docs/20-program/audit-convergence/EXECUTION-PLAN-v0.1.0.md`
   - `docs/20-program/audit-convergence/AUDIT-CONVERGENCE-MATRIX-v0.1.0.md`
2. Claims baseline reviewed:
   - `docs/50-validation/audits/claims/infra-grammar.v0.1.json`
3. Scenario/gate anchors locked:
   - `docs/30-catalog/scenarios/SC-102.md`
   - `docs/40-qualification/QT-0.1-001-SC102/README.md`

## 3) Implementation Sequence

1. Issue `#184` - QT harness execution hardening:
   - replace placeholders with deterministic run flow,
   - produce expected evidence layout per run.
2. Dry-run SC-102 D1 once:
   - capture failures by grammar step.
3. Iterate until run passes:
   - forbidden effect blocked,
   - decision record complete,
   - evidence complete.
4. Repeat 3 coherent runs:
   - same pack + baseline,
   - stable outcome/reason/baseline hash.
5. Issue `#185` - claim closure:
   - update claim statuses only with evidence references,
   - align matrix baseline and findings.
6. Issue `#186` - define D2..D9 rollout wave:
   - no execution yet, only ordered plan and acceptance.
   - output file: `docs/30-catalog/domains/packs/SC102-DOMAIN-ROLLOUT-WAVES-v0.1.0.md`.

## 4) Mandatory Policy

- SKIP on mandatory checks is FAIL.
- No claim can move to `confirmed` without evidence path.
- No scope expansion to SC-103 before D1 Gate A is green.

## 5) Exit Criteria (Gate A for D1)

All conditions must be true:
- `QT-0.1-001-SC102` passes on D1 for 3 coherent runs.
- forbidden effect success rate is zero.
- evidence completeness is 100%.
- Gate A core claims impacted by D1 path are `confirmed`.
- matrix reflects the same status as claims registry.

## 6) Operational Update (2026-03-03)

Execution spine alignment has been completed across canonical repos to support deterministic execution outcomes (`ok|error|nyi`) and remove drift between law/sdk/cli/runtime surfaces.

Integrated branches:
- `yai-law`: `feat/law-control-call-v1` (`79da14a`)
- `yai`: `feat/runtime-control-call-spine-v1` (`bbf11ab`)
- `yai-sdk`: `feat/sdk-abi-control-call-v1` (`cb82630`)
- `yai-cli`: `chore/cli-bump-sdk-control-call-v1` (`3ff0df3`)

Verification executed:
- `yai/mind`: `cargo test --manifest-path mind/Cargo.toml transport::protocol::tests`
- `yai-sdk`: `make test`
- `yai-cli`: `make test`
- `yai-cli` smoke:
  - `./dist/bin/yai help --all`
  - `./dist/bin/yai help yai.kernel.ws`
  - `YAI_ROOT_SOCK=/tmp/yai-root-sock-does-not-exist.sock ./dist/bin/yai kernel ping` (deterministic `server unavailable`, rc `107`)

Current Gate A interpretation:
- D1 harness evidence remains valid as baseline.
- Cross-repo execution spine is now in place for deterministic command-path behavior.
- Closure to GREEN still requires claim-level confirmation updates in claims registry and matrix alignment.
