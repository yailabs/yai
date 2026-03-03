---
id: AUDIT-CONVERGENCE-MATRIX-v0.1.0
status: draft
owner: governance
updated: 2026-03-03
registry: docs/50-validation/audits/claims/infra-grammar.v0.1.json
plan: docs/20-program/audit-convergence/EXECUTION-PLAN-v0.1.0.md
---

# Audit Convergence Matrix (v0.1.0)

This matrix is the single map from audit target-state to execution artifacts.

Legend:
- `A` = Gate A (Core)
- `B` = Gate B (Mind)
- Status baseline: `confirmed | partial | not_present`

## 1) Domain x Grammar Map

| Domain | Trigger | Context | Authority/Contract | Decision | Enforcement | Evidence | Claim IDs | Gate | Baseline |
|---|---|---|---|---|---|---|---|---|---|
| Control Plane | `RB-CONTRACT-BASELINE-LOCK#phase-0-1-1-ci-parity` | `RB-ROOT-HARDENING#phase-0-1-0-protocol-guardrails` | `RB-CONTRACT-BASELINE-LOCK#phase-0-1-0-pin-baseline-freeze` | `RB-ROOT-HARDENING#phase-0-1-2-envelope-authority-gate` | `RB-ROOT-HARDENING#phase-0-1-4-kernel-hard-reject-invalid-ws-id` | `RB-CONTRACT-BASELINE-LOCK#phase-0-1-4-cross-repo-evidence` | `C-SPEC-FIRST-PINNED`, `C-ENVELOPE-HANDSHAKE-REQUIRED`, `C-EVIDENCE-PACK-REPRODUCIBLE` | A | partial |
| Network | `RB-ROOT-HARDENING#phase-0-1-1-byte-perfect-router` | `RB-ROOT-HARDENING#phase-0-1-3-ws-id-validation-centralization` | `RB-SPECS-REFACTOR-FOUNDATION#phase-0-1-6-toolchain-policy` | `RB-ROOT-HARDENING#phase-0-1-2-envelope-authority-gate` | `RB-ROOT-HARDENING#phase-0-1-4-kernel-hard-reject-invalid-ws-id` | `RB-ROOT-HARDENING#phase-0-1-5-test-matrix-torture-suite` | `C-DOMAIN-COVERAGE-NETWORK`, `C-KERNEL-HARD-BOUNDARY-CORE` | A | partial |
| Providers | `RB-ENGINE-ATTACH#phase-engine-attach-v4` | `RB-ENGINE-ATTACH#phase-engine-attach-v4` | `RB-CONTRACT-BASELINE-LOCK#phase-0-1-3-formal-core-sync` | `RB-ENGINE-ATTACH#phase-engine-attach-v4` | `RB-DATA-PLANE` (provider gate evidence) | `RB-DATA-PLANE` + MP evidence | `C-DOMAIN-COVERAGE-PROVIDERS`, `C-CONTEXT-PROPAGATION` | A | partial |
| Storage | `RB-DATA-PLANE` v5.0/v5.1 | `RB-WORKSPACES-LIFECYCLE#phase-0-1-0-workspace-layout` | `RB-SPECS-REFACTOR-FOUNDATION#phase-0-1-7-formal-binding` | `RB-DATA-PLANE` v5.1/v5.2 | `RB-DATA-PLANE` v5.3/v5.4 | `RB-DATA-PLANE` phase evidence set | `C-DOMAIN-COVERAGE-STORAGE`, `C-KERNEL-HARD-BOUNDARY-CORE` | A | partial |
| Resources/Workspace | `RB-WORKSPACES-LIFECYCLE#phase-0-1-1-ws-create-guardrails` | `RB-WORKSPACES-LIFECYCLE#phase-0-1-1-ws-create-guardrails` | `RB-WORKSPACES-LIFECYCLE (phase 0.1.3)` | `RB-WORKSPACES-LIFECYCLE (phase 0.1.3)` | `RB-WORKSPACES-LIFECYCLE (phase 0.1.3)` | `RB-WORKSPACES-LIFECYCLE (phase 0.1.4)` | `C-DOMAIN-COVERAGE-RESOURCE`, `C-CONTEXT-PROPAGATION` | A | partial |
| Audit/Traceability | `RB-SPECS-REFACTOR-FOUNDATION#phase-0-1-5-ci-guardrails` | `RB-SPECS-REFACTOR-FOUNDATION#phase-0-1-2-sanity-links` | `RB-CONTRACT-BASELINE-LOCK#phase-0-1-0-pin-baseline-freeze` | `RB-SPECS-REFACTOR-FOUNDATION#phase-0-1-7-formal-binding` | `RB-CONTRACT-BASELINE-LOCK#phase-0-1-2-no-pass-on-skip` | `RB-CONTRACT-BASELINE-LOCK#phase-0-1-4-cross-repo-evidence` | `C-EVIDENCE-PACK-REPRODUCIBLE`, `C-SKIP-FAIL-MANDATORY` | A | partial |
| Mind (L3) | `RB-MIND-REDIS-STM#phase-mind-proposer` | `RB-MIND-REDIS-STM#phase-mind-proposer` | `RB-MIND-REDIS-STM` + `ADR-005-mind-proposer` | `RB-MIND-REDIS-STM#phase-mind-proposer` | Kernel/Engine mediated path required by runbook closure | End-to-end proposal->enforcement evidence bundle | `C-MIND-PROPOSER-KERNEL-ENFORCER` | B | not_present |

## 2) Runbook Re-centering Order
1. `docs/20-program/23-runbooks/contract-baseline-lock.md` (continue from active `0.1.1`, no mid-phase rewrite)
2. `docs/20-program/23-runbooks/specs-refactor-foundation.md` (backbone binding)
3. `docs/20-program/23-runbooks/root-hardening.md`
4. `docs/20-program/23-runbooks/workspaces-lifecycle.md`
5. `docs/20-program/23-runbooks/engine-attach.md`
6. `docs/20-program/23-runbooks/data-plane.md`
7. `docs/20-program/23-runbooks/mind-redis-stm.md`

## 3) Mandatory Evidence Command Families
- Pins/contracts:
  - `tools/bin/yai-check-pins`
- Proof/traceability:
  - `tools/bin/yai-docs-trace-check`
  - `tools/bin/yai-proof-check`
- Runtime verify/suite:
  - `tools/bin/yai-verify`
  - `tools/bin/yai-suite`

Operational policy:
- If command family is mandatory for the phase, `SKIP` closes as `FAIL`.

## 4) Progress Metrics (no fake percent)
Use claim-based metrics only:
- Core completion = confirmed core claims / total core claims
- Mind completion = confirmed mind claims / total mind claims

Do not report synthetic percentages disconnected from claim status.


## 5) Gate A D1 Progress Snapshot (2026-02-24)
- Qualification harness (`QT-0.1-001-SC102`) executed for `D1-digital/egress-v1` with `baseline-deny`.
- Run command:
  - `cd docs/40-qualification/QT-0.1-001-SC102`
  - `DOMAIN_PACK_ID=D1-digital/egress-v1 BASELINE_ID=baseline-deny ./run/run-three.sh`
- Result: `3/3 PASS` (simulation harness) with indexed evidence per run.
- Canonical finding: `docs/50-validation/audits/findings/FINDING-001-sc102-d1-harness-runset.md`

## 6) Execution Spine Snapshot (2026-03-03)

Cross-repo deterministic execution spine branches prepared and pushed:
- `yai-law` `feat/law-control-call-v1` (`79da14a`)
- `yai` `feat/runtime-control-call-spine-v1` (`bbf11ab`)
- `yai-sdk` `feat/sdk-abi-control-call-v1` (`cb82630`)
- `yai-cli` `chore/cli-bump-sdk-control-call-v1` (`3ff0df3`)

Observed runtime semantics for command path:
- registered command dispatch does not collapse to ad-hoc unknown path
- deterministic non-success path available (`nyi` / `server unavailable`) with stable machine code mapping

Gate impact:
- Strengthens Trigger->Decision->Enforcement chain readiness for Gate A.
- Does not by itself flip claims to `confirmed`; claim registry update remains mandatory closure step.
