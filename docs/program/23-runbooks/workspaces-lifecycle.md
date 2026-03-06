---
id: RB-WORKSPACES-LIFECYCLE
status: draft
revision: 2
owner: runtime
effective_date: 2026-03-03
supersedes:
  - RB-WORKSPACES-LIFECYCLE@rev1
scope_grade: SC102

# Core dependencies for this runbook
# SC103 adds optional mind surfaces.
depends_on:
  - RB-ROOT-HARDENING
  - RB-DATA-PLANE@rev2
  - RB-ENGINE-ATTACH

adr_refs:
  - docs/program/22-adr/ADR-006-unified-rpc.md
  - docs/program/22-adr/ADR-007-workspace-isolation.md
  - docs/program/22-adr/ADR-011-contract-baseline-lock.md
  - docs/program/22-adr/ADR-012-audit-convergence-gates.md

tools:
  - tools/bin/yai-check-pins
  - tools/bin/yai-verify
  - tools/bin/yai-gate
  - tools/bin/yai-suite
---

# RB-WORKSPACES-LIFECYCLE — Workspace Lifecycle (v2)

Status: Draft | Revision: 2 | Owner: Runtime | Effective date: 2026-03-03  
Supersedes: RB-WORKSPACES-LIFECYCLE@rev1  
Scope grade: SC102 (Mind NOT required) — SC103 adds optional Mind surfaces

## Dependencies & references

### Depends on
- `RB-ROOT-HARDENING`
- `RB-DATA-PLANE` (rev2)
- `RB-ENGINE-ATTACH`

### ADR refs
- `docs/program/22-adr/ADR-006-unified-rpc.md` (Unified RPC)
- `docs/program/22-adr/ADR-007-workspace-isolation.md` (Workspace isolation)
- `docs/program/22-adr/ADR-011-contract-baseline-lock.md` (Baseline lock + pins)
- `docs/program/22-adr/ADR-012-audit-convergence-gates.md` (Audit convergence gates)

### Related specs (yai-law)
- `deps/yai-law/contracts/control/schema/control_plane.v1.json`
- `deps/yai-law/contracts/protocol/include/auth.h`
- `deps/yai-law/contracts/protocol/include/audit.h`
- `deps/yai-law/contracts/protocol/include/transport.h`
- `deps/yai-law/runtime/kernel/README.md` (runtime model)
- `deps/yai-law/runtime/engine/schema/engine_cortex.v1.json`
- `deps/yai-law/runtime/mind/graph/schema/graph.v1.json` (optional, SC103)

### Related runbooks
- `docs/program/23-runbooks/root-hardening.md`
- `docs/program/23-runbooks/data-plane.md` (rev2, canonical storage contract)
- `docs/program/23-runbooks/engine-attach.md`
- `docs/program/23-runbooks/mind-redis-stm.md` (optional)

### Tools
- `yai-check-pins`
- `yai-verify`
- `yai-gate`
- `yai-suite`

## 1. Purpose

Define the workspace lifecycle as the canonical mechanism for:
- containment-by-default: a workspace is a containerized execution context (not a client calling middleware)
- scanner semantics: workload passes through a governed perimeter where effects-out are gated
- operational interaction: safe live operations (inspect/authorize/baseline change) without cowboy edits
- layered adoption:
  - SC102: enforcement + evidence without Mind
  - SC103: optional Mind decision support and automation on top

## 2. Definitions

### 2.1 Workspace (WS)

A workspace is a tenant-isolated execution perimeter that hosts:
- pinned law/spec/registry context (baseline/policy)
- storage layout (data plane)
- execution runs and evidence
- optional Mind surfaces (SC103)

Key rule: workloads do not call YAI; they execute inside WS scope where effects-out are governed.

### 2.2 Scanner semantics (airport scanner)

A WS enforces:
- gating on external effects (egress, storage writes, publication, transactions, actuator commands)
- deterministic outcomes: `allow | deny | quarantine`
- evidence bundle materialization and indexing

### 2.3 Gates

A gate is a runtime boundary at an effect commit point and exists to prevent uncontrolled side effects.

### 2.4 Quarantine

Quarantine is persisted pending state for an action/run:
- external effect is NOT committed
- state is checkpointed/parked
- resolution is governed (CLI/SDK/Cockpit), with TTL and audit evidence

## 3. Scope

### In scope
- workspace lifecycle commands: create/open/close/destroy
- workspace identity and isolation rules
- execution run lifecycle inside workspace
- per-workspace gates configuration model
- live interaction model: inspect / authorize / baseline change / resume
- pending/quarantine queue semantics (persisted, TTL, resolvable)
- evidence conventions (aligned to RB-DATA-PLANE)

### Out of scope
- multi-node clustering, cross-workspace queries
- SC103 advanced graph causal queries
- high-throughput distributed event storage

## 4. Non-negotiable invariants

1. Workspace isolation: no cross-workspace storage, queries, or side effects
2. Path jail: all file operations resolve under `~/.yai/run/<ws_id>/`
3. Fail-closed on effects-out: no proof => no effect commit
4. Evidence first-class: every decision touching effects-out emits evidence refs + hashes
5. Mind never SC102 dependency: SC102 must run without Redis/LLM/Mind

## 5. Data Plane alignment (RB-DATA-PLANE rev2)

Canonical storage contract:

Runtime workspace metadata root: `~/.yai/run/<ws_id>/`
Workspace working root (`root_path`): `~/.yai/workspaces/<ws_id>` by default, or explicit `--root`.

Minimum layout (rev2):

```text
~/.yai/run/<ws_id>/
├── manifest.json
├── authority/           # LMDB (L1) - kernel store
├── events/              # DuckDB (L2) - engine store
├── engine/              # sockets/pid
└── logs/
```

Rules:
- `ws create` MUST create/validate layout + `manifest.json`
- `manifest.json` MUST persist `ws_id`, `state`, `root_path`, `created_at`, `updated_at`
- `ws open` MUST validate `manifest.json` and ABI compatibility
- CLI MUST NOT touch storage directly:
  - `CLI -> Root -> Kernel -> storage`

## 6. Lifecycle model

### 6.1 States
- `NON_EXISTENT`
- `CREATED` (layout exists, not active)
- `OPEN` (active session + runtime surfaces available)
- `CLOSING` (draining)
- `CLOSED` (inactive but retained)
- `DESTROYED` (removed, irreversible)

### 6.2 Allowed transitions
- `NON_EXISTENT -> CREATED -> OPEN -> CLOSED -> DESTROYED`
- `OPEN -> CLOSING -> CLOSED`

Forbidden:
- `OPEN -> DESTROYED` directly (must close first)
- `CLOSED -> OPEN` without re-validation
- cross-tenant operations

## 7. Control plane responsibilities

### Root
- entrypoint for CLI/SDK/Cockpit
- validate workspace id and auth context
- route to Kernel
- enforce handshake lifecycle

### Kernel (L1 authority)
- workspace identity + isolation
- path jail enforcement for storage ops
- authority checks for privileged operations (authorize/break-glass/baseline change)
- expose controlled surfaces to engine/mind

### Engine (L2 event store)
- append events and decision records
- produce/export metrics where applicable

### Mind (optional, SC103)
- diagnosis and remediation suggestions
- planning support
- final commits still governed/evidenced

## 8. Gates model (effect commit points)

### 8.1 Gate categories (examples)
- D1: network egress
- D2: actuator command
- D5: economic transaction authorization
- D7: publication enforcement
- D8: reproducibility / parameter lock
- D6: incident response action

### 8.2 Gate outcomes
- `allow`: effect may commit
- `deny`: effect must not commit
- `quarantine`: effect must not commit; action becomes pending

Rule: quarantine requires checkpoint/park, never infinite sleep.

## 9. Execution runs inside a workspace

### 9.1 Run concept

A run is a bounded unit of activity in a workspace and MUST have:
- `trace_id` (or equivalent)
- decision records + timeline (append-only)
- evidence index references
- baseline/policy hash linkage

### 9.2 Two-phase execution
- propose: evaluate action + evidence context
- commit effect: only if gate returns allow (or quarantine resolved)

## 10. Live interaction model (production-safe)

### 10.1 Principle
YAI governs external effects at gates; deny/quarantine transitions into known operational state.

### 10.2 Standard handling modes
- `ABORT`
- `RETRY` (bounded retries + backoff)
- `WAIT_FOR_AUTH` (quarantine persisted pending)

### 10.3 Quarantine queue
Quarantined actions MUST be:
- persisted
- observable (`list/inspect`)
- resolvable (`authorize/deny/baseline change`)
- bounded by TTL

TTL minimum:
- each pending has TTL
- on expiry: abort or compensating action (policy-defined)
- TTL expiry emits evidence + reason codes

### 10.4 Interaction surfaces
- CLI (now)
- Cockpit UI (future, via SDK)
- SDK (embedded/pro)

No ad-hoc scripts as operational control plane.

### 10.5 Three authoritative live operations
A) Inspect (read-only)
- pending queue, decision records, reason codes
- baseline/policy hash in force
- evidence index and timeline

B) Authorize/resolve (run-scoped)
- emit authorization event for quarantined action
- scoped to run/action
- includes who/when/why/authority

C) Baseline/policy change (forward-scoped)
- bump pinned baseline/policy version
- affects future decisions (or explicit reevaluation path)
- evidence-captured governance change

### 10.6 Break-glass override
Only governed action with:
- strict single run/action scope
- bounded time window
- mandatory justification
- automatic postmortem hooks
- high-visibility evidence

## 11. CLI operational contract (v2, aligned to expanded registry)

Exact command names may evolve, semantics MUST stay stable.

### 11.1 Source-of-truth rules
- Canonical interface is `command_id` from `yai-law/registry/commands.v1.json`.
- CLI aliases are convenience surfaces and MUST resolve to the same canonical `command_id`.
- For workspace lifecycle in SC102, aliases below are the required operational surface.

### 11.2 Implemented-now operational aliases (SC102 core)

```bash
yai root ping
yai kernel ping

# Workspace lifecycle via kernel
yai kernel ws create --ws-id <ws_id>
yai kernel ws reset  --ws-id <ws_id>
yai kernel ws destroy --ws-id <ws_id>

# Stack lifecycle
yai lifecycle up      [--ws <ws_id>] [--no-engine] [--no-mind] [--detach]
yai lifecycle restart [--ws <ws_id>] [--force]
yai lifecycle down    [--ws <ws_id>] [--force]

# Inspection
yai inspect status [--ws <ws_id>] [--json]
yai inspect logs <component> [--ws <ws_id>] [--follow]
yai inspect monitor
yai inspect events [--ws <ws_id>]

# Verification
yai verify verify <target>
yai verify test <target> [--ws <ws_id>] [--timeout-ms <ms>]
```

### 11.3 Canonical IDs backing current aliases
- `yai.root.ping`
- `yai.kernel.ping`
- `yai.kernel.ws` (argument `action=create|reset|destroy`)
- `yai.lifecycle.up`
- `yai.lifecycle.restart`
- `yai.lifecycle.down`
- `yai.inspect.status`
- `yai.inspect.logs`
- `yai.inspect.monitor`
- `yai.inspect.events`
- `yai.verify.verify`
- `yai.verify.test`

### 11.4 Expanded WS-related registry families (now cataloged)

The registry is already expanded (200 IDs per group). Workspace lifecycle v2 MUST evolve by implementing these families progressively without changing contract semantics:

- `lifecycle` families: `workspace_*`, `stack_*`, `runtime_*`, `service_*`, `session_*`, `daemon_*`, `engine_*`, `mind_*`, `plane_*`, `profile_*`.
- `kernel` families: `ws_*`, `boundary_*`, `enforce_*`, `policy_*`, `route_*`, `session_*`, `resource_*`, `quota_*`, `mount_*`, `audit_*`.
- `inspect` families: `status_*`, `logs_*`, `events_*`, `health_*`, `metrics_*`, `trace_*`, `routes_*`, `sessions_*`, `jobs_*`, `alerts_*`.
- `verify` families: `gate_*`, `policy_*`, `evidence_*`, `proof_*`, `trace_*`, `bundle_*`, `report_*`, `suite_*`, `vector_*`, `check_*`.
- `control` families used by live operations: `authority_*`, `dispatch_*`, `route_*`, `session_*`, `provider_*`, `target_*`, `policy_*`, plus compatibility IDs (`call`, `root`, `kernel`, `chat`, `shell`, `dsar`, `providers`, `sessions`).

### 11.5 Contract guarantees for all registered commands
- A registered `command_id` MUST be invocable through CLI/SDK path.
- If runtime handler is missing, response MUST be deterministic (`nyi`/mapped equivalent), never `unknown command`.
- Help surfaces MUST expose both overview and full catalog views:
  - `yai help` -> overview (compact)
  - `yai help <group>` -> compact group overview
  - `yai help <group>:all` -> full group command expansion

### 11.6 Mediation hard rule
CLI access is always mediated by governance plane:

`CLI -> Root -> Kernel -> storage/effect`

Direct storage mutation from CLI is forbidden.

### 11.7 Complete command map (all groups)
The complete catalog (all `2800` command_id) is tracked in:

- `docs/program/23-runbooks/workspaces-lifecycle-command-map.v2.md`

This runbook stays normative on lifecycle semantics; the map file is the exhaustive command inventory source for planning waves.
Workspace-lifecycle MPs are scoped under `docs/program/24-milestone-packs/workspaces-lifecycle/`.
Non-WS command expansion packs are tracked under `docs/program/24-milestone-packs/command-coverage/`.

## 12. Mind integration (optional, SC103)

### 12.1 Modes
- Mode 1 (SC102 default): governance-only, no Mind dependency
- Mode 2 (SC103): active control support from Mind, commits still gated/evidenced

### 12.2 Mind action constraints
Mind-driven actions MUST:
- be represented as actions/events in workspace
- be policy/authority authorized
- be evidence-captured like human actions

### 12.3 STM/Redis behavior
If Redis absent:
- safe degradation (`DEGRADED_READONLY` for STM endpoints)
- SC102 enforcement continues unaffected

## 13. Evidence & audit convergence

### 13.1 Minimum evidence per run (SC102-grade)
A run touching effects-out MUST produce:
- decision record(s) with baseline hash
- events timeline
- containment metrics (where applicable)
- minimal system_state snapshot
- evidence index

### 13.2 Audit convergence gates
Lifecycle transitions and resolve actions MUST:
- emit audit events (engine append)
- be queryable by trace/run id
- be verifiable (hash + required fields)

## 14. Failure modes (fail-closed)

| Failure | Required behavior |
|---|---|
| Path jail violation | block op, emit evidence + reason code |
| Cross-workspace access | deny, audit event, no partial reads |
| Baseline/policy hash mismatch | deny/quarantine; never commit effect |
| Pending TTL expiry | policy abort/compensation + evidence |
| Mind/Redis unavailable (SC103) | safe degrade, SC102 unaffected |
| Unauthorized resolve/override | deny + audit event |

## 15. Rollback & recovery

### 15.1 Operational rollback
- run-level: abort pending, rollback to safe checkpoint, or deterministic rerun
- policy-level: explicit pinned rollback, never silent revert

### 15.2 Recovery guarantees
Pending/quarantine MUST survive restart.
Open workspace restart MUST revalidate:
- manifest ABI compatibility
- authority store presence
- events store integrity (openability at minimum)

## 16. Acceptance criteria (v2)

- [ ] `ws create/open/close/destroy` enforce state machine + audit events
- [ ] data-plane layout created/validated per RB-DATA-PLANE rev2
- [ ] deterministic gate outcomes at effect commit points
- [ ] quarantine persisted pending with TTL
- [ ] live ops inspect/authorize/baseline-change are governed + auditable
- [ ] CLI never directly touches storage
- [ ] SC102 works with Mind absent
- [ ] evidence bundle generated and verifiable per run

### Closure gates
```bash
tools/bin/yai-check-pins
tools/bin/yai-verify list
tools/bin/yai-verify core
```

## 17. Traceability

### ADR refs
- `docs/program/22-adr/ADR-006-unified-rpc.md`
- `docs/program/22-adr/ADR-007-workspace-isolation.md`
- `docs/program/22-adr/ADR-011-contract-baseline-lock.md`
- `docs/program/22-adr/ADR-012-audit-convergence-gates.md`

### Runbooks
- `docs/program/23-runbooks/data-plane.md` (rev2)
- `docs/program/23-runbooks/root-hardening.md`
- `docs/program/23-runbooks/engine-attach.md`
- `docs/program/23-runbooks/mind-redis-stm.md` (optional)
