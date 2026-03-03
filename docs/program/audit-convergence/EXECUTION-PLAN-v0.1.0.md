---
id: EXECUTION-PLAN-AUDIT-CONVERGENCE-v0.1.0
status: draft
owner: governance
effective_date: 2026-02-21
revision: 2
scope: docs-only
issue:
  - https://github.com/yai-labs/yai/issues/140
  - https://github.com/yai-labs/yai/issues/183
  - https://github.com/yai-labs/yai/issues/184
  - https://github.com/yai-labs/yai/issues/185
  - https://github.com/yai-labs/yai/issues/186
related:
  - docs/20-program/audit-convergence/AUDIT-CONVERGENCE-MATRIX-v0.1.0.md
  - docs/50-validation/audits/claims/infra-grammar.v0.1.json
  - docs/20-program/22-adr/ADR-012-audit-convergence-gates.md
---

# Execution Plan - Audit Convergence v0.1.0

## 1) Objective
Build one program-level trajectory where all active runbooks converge to one measurable target:
`trigger -> context -> authority/contract -> decision -> enforcement -> evidence`
across all effect domains, including Mind integration.

This plan is the operating contract for Program Delivery in `yai`.

## 2) Current Baseline (2026-02-21)
- In-flight runbook: `RB-CONTRACT-BASELINE-LOCK`
- Active phase declared by operator: `0.1.1`
- Branch for this convergence work: `feat/audit-convergence-v0.1.0`
- Tracking issue: `yai#140`

Constraint:
- Do not rewrite the active phase scope mid-execution.
- Re-centering applies at phase boundaries and on subsequent phases/MP closure.

## 2.1) Baseline Update (2026-03-03)

Execution spine implementation state:
- `yai-law`: `feat/law-control-call-v1` (`79da14a`)
- `yai`: `feat/runtime-control-call-spine-v1` (`bbf11ab`)
- `yai-sdk`: `feat/sdk-abi-control-call-v1` (`cb82630`)
- `yai-cli`: `chore/cli-bump-sdk-control-call-v1` (`3ff0df3`)

Operational note:
- this update enables deterministic command execution semantics required by Gate A paths,
- claim status remains governed by `infra-grammar.v0.1.json` closure rules.

## 3) What "GREEN" means for v0.1.0 (non-hype)
For v0.1.0, GREEN means:
- hard mediation at runtime boundaries (Root/Kernel/Engine gates) is non-bypassable for governed effects,
- decisions are contract-driven and traceable,
- evidence is reproducible and cannot pass through mandatory-check skips.

Out of scope for v0.1.0 GREEN:
- full OS hardening product level (seccomp/cgroups/LSM completeness),
- production SOC/SIEM maturity claims.

<a id="4-two-official-gates"></a>
## 4) Two Official Gates
### Gate A - Audit Green Core (L0-L2)
Domains: control plane, network, providers, storage, resources/workspaces, audit pipeline.

Done when all are true:
- Core claims in `docs/50-validation/audits/claims/infra-grammar.v0.1.json` are `confirmed`.
- Mandatory evidence commands for core domains have no SKIP closure.
- MPs for core runbook phases include reproducible evidence pointers.

### Gate B - Audit Green Mind (L3)
Domain: Mind (proposer-only) integrated with Kernel enforcement path.

Done when all are true:
- Mind claims are `confirmed`.
- Mind cannot bypass Kernel/Engine authority decisions.
- End-to-end evidence exists from Mind proposal through Kernel/Engine enforcement and audit artifacts.

Program checkpoint policy:
- Gate A GREEN = core platform checkpoint.
- Gate B GREEN = v0.1.0 full Infra Grammar checkpoint (Mind included).

## 5) Canonical Artifacts (single source chain)
1. Claims source of truth:
   `docs/50-validation/audits/claims/infra-grammar.v0.1.json`
2. Convergence matrix:
   `docs/20-program/audit-convergence/AUDIT-CONVERGENCE-MATRIX-v0.1.0.md`
3. Governance decision (gate semantics):
   `docs/20-program/22-adr/ADR-012-audit-convergence-gates.md`

Rule:
- Runbooks/MPs/board cards reference claim IDs from the JSON registry.
- Claims may evolve only via PR that updates registry + matrix + affected runbook refs.

## 6) Mandatory Policy: SKIP = FAIL
For mandatory evidence commands in a phase:
- `PASS`: command executed and criteria satisfied.
- `FAIL`: command executed and criteria not satisfied.
- `SKIP`: treated as `FAIL` for closure.

Exception handling:
- Temporary waivers require explicit issue link, owner, expiry date, and alternative control.
- Waiver never grants milestone closure by itself.

## 7) Wave Plan (start -> full green)

### Wave 0 - Stabilize current in-flight baseline
Primary runbook:
- `docs/20-program/23-runbooks/contract-baseline-lock.md`

Target from current execution point:
- close phases `0.1.1` -> `0.1.4` with strict evidence.

Required outputs:
- completed MP sequence under `docs/20-program/24-milestone-packs/contract-baseline-lock/`
- explicit no-skip enforcement for mandatory checks

Exit:
- baseline lock claims in registry move to `confirmed`.

### Wave 1 - Backbone convergence (specs + governance bindings)
Primary runbook:
- `docs/20-program/23-runbooks/specs-refactor-foundation.md`

Scope:
- keep structural phases deterministic,
- bind claims/evidence obligations to phase gates,
- align pinning and traceability tooling with closure requirements.

Required outputs:
- matrix-driven phase checklist references in MP closures,
- phase-to-claim linkage stabilized across `0.1.2+`.

Exit:
- specs governance claims and traceability claims are `confirmed`.

### Wave 2 - Root boundary hardening (core mediation)
Primary runbook:
- `docs/20-program/23-runbooks/root-hardening.md`

Scope:
- envelope validation,
- deterministic reject paths,
- authority gate behavior,
- torture/repeatability evidence.

Exit:
- core trigger/context/decision/enforcement claims show hard mediation evidence.

### Wave 3 - Workspace lifecycle as governed effects
Primary runbook:
- `docs/20-program/23-runbooks/workspaces-lifecycle.md`

Scope:
- governed create/list/destroy,
- authority + arming/role checks,
- side-effect boundaries and rollback behavior.

Exit:
- resource domain row in matrix reaches confirmed across grammar chain.

### Wave 4 - Engine attach integration
Primary runbook:
- `docs/20-program/23-runbooks/engine-attach.md`

Scope:
- attach handshake path,
- identity/context propagation,
- authority mediation continuity Root->Kernel->Engine.

Exit:
- control+engine attach claims confirmed with deterministic evidence.

### Wave 5 - Data plane governance
Primary runbook:
- `docs/20-program/23-runbooks/data-plane.md`

Scope:
- storage contract surfaces,
- domain-specific enforcement and isolation checks,
- evidence repeatability for data-side effects.

Exit:
- storage/network/provider rows reach Gate A criteria.

### Wave 6 - Mind integration (proposer-only)
Primary runbook:
- `docs/20-program/23-runbooks/mind-redis-stm.md`

Scope:
- Mind proposer path only,
- no direct bypass of Kernel decisions,
- Redis STM treated as L3 memory, not authority.

Exit:
- Mind row confirmed for Gate B criteria.

### Wave 7 - Final audit closure pack
Scope:
- final claims review,
- final evidence bundle verification,
- claims vs reality status = all GREEN for v0.1.0 per gate definitions.

Exit:
- Gate A and Gate B both green.

## 8) Execution Controls
- Work type: docs-only in this branch.
- No runtime code changes in this convergence branch.
- No direct edits under `deps/*` in consumer repos; specs changes happen in `yai-law` branch and pins are aligned only at merge closure.
- Each wave closure must reference:
  - runbook anchor,
  - MP artifact,
  - claim IDs,
  - command evidence list.

## 9) Board/Issue Operations
- Master issue: `https://github.com/yai-labs/yai/issues/140`
- Program board target: `YAI Program Delivery` (issue `#140` linked).

Required issue fields:
- Track
- Phase
- Runbook Ref
- MP-ID
- Gate Status
- Claim IDs

## 10) Success Criteria for this Plan Artifact
This plan is considered implemented when:
- registry + matrix + ADR are published and cross-linked,
- runbook/ADR index pages reference this convergence backbone,
- operators can execute wave closures without ad-hoc scope drift.
