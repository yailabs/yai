---
id: RB-DP-STORAGE-CLASSES
status: draft
owner: runtime
effective_date: 2026-03-09
revision: 1
depends_on:
  - RB-DATA-PLANE
---

# Canonical Storage Classes and Backend Role Model (DP-2)

## 1) Purpose
Define canonical storage classes by semantics first, then map each class to backend roles.

This runbook prevents:
- backend-first design,
- one-store calderone,
- cache/truth confusion,
- query-first cutovers.

## 2) Canonical storage classes

### SC-1 Workspace Operational State
Includes workspace identity/binding, declared/inferred context, active attachment refs, and last operational state.

### SC-2 Authority State
Includes authority records, authority constraints, and enforcement-relevant authority transitions.

### SC-3 Governance/Compliance Object State
Includes parsed governance refs, normalized IR refs, candidate/approved objects, and object metadata.

### SC-4 Review/Approval/Apply State
Includes lifecycle state, review/approval outcomes, apply eligibility, attachment/apply state, supersede/withdraw state.

### SC-5 Event Log
Includes runtime/workspace/execution/governance lifecycle events.

### SC-6 Resolution/Evidence Log
Includes decision records, effect summaries, evidence refs, audit lineage, and trace refs.

### SC-7 Artifact Metadata
Includes artifact refs, ownership/scope metadata, and governance/artifact linkage metadata.

### SC-8 Brain Graph Memory
Includes semantic/episodic graph truth and domain/authority/evidence-linked edges.

### SC-9 Transient Cognition / STM
Includes activation state, hot graph neighborhoods, short-term working sets, volatile cognitive cache.

## 3) Semantic profile model
Each class is profiled with:
- `authority_level`
- `owner_layer`
- `persistence_class`
- `mutation_pattern`
- `query_pattern`
- `scope`
- `durability_expectation`
- `cross_workspace_sensitivity`
- `operational_criticality`
- `debug_visibility`
- `exportability`

## 4) Ownership mapping

| Storage Class | Primary Owner | Contributors | Main Consumers |
| --- | --- | --- | --- |
| SC-1 Workspace Operational State | core | exec, governance lifecycle | cli, sdk, brain (indirect) |
| SC-2 Authority State | core | review/apply lifecycle | exec, law mappings, inspect |
| SC-3 Governance/Compliance Object State | law-facing governance lifecycle | parsing engine, authoring surface | core, exec, workspace apply |
| SC-4 Review/Approval/Apply State | governance lifecycle + core apply mediation | core/session/workspace | cli, sdk, inspect/effective/debug |
| SC-5 Event Log | exec | runtime producers | operator query, evidence linkers, brain hooks |
| SC-6 Resolution/Evidence Log | law mappings + runtime resolution | exec/core | audit/debug/inspect/query, graph hooks |
| SC-7 Artifact Metadata | core | runtime artifact emitters | governance linkage, inspect/query |
| SC-8 Brain Graph Memory | brain | evidence/authority/domain graph hooks | reasoning and graph summaries |
| SC-9 Transient Cognition/STM | brain | runtime cognition loops | cognition/orchestration assist layers |

## 5) Backend role classes

### BR-1 Embedded Authoritative KV
For high-authority, controlled mutation, workspace-owned state.

### BR-2 Append and Tabular Query Store
For append logs, resolution/evidence history, operator-oriented query surfaces.

### BR-3 Persistent Graph Store
For graph truth persistence: nodes, edges, semantic/episodic relations.

### BR-4 Transient Memory / Hot Cache Store
For volatile STM, activation state, hot neighborhoods.

### BR-5 Export / Artifact Surface
For files, dumps, interop bundles, debug snapshots. Non-authoritative by default.

## 6) Mapping storage class -> backend role

| Storage Class | Primary Backend Role | Secondary Role(s) | Not primary on |
| --- | --- | --- | --- |
| SC-1 Workspace Operational State | BR-1 | BR-5 | BR-4 |
| SC-2 Authority State | BR-1 | BR-5 | BR-2, BR-4 |
| SC-3 Governance/Compliance Object State | BR-1 or BR-2 (hybrid discipline, phased) | BR-5 | BR-4 |
| SC-4 Review/Approval/Apply State | BR-1 | BR-2 (query mirror), BR-5 | BR-4 |
| SC-5 Event Log | BR-2 | BR-5 | BR-1 as sole primary |
| SC-6 Resolution/Evidence Log | BR-2 | BR-3 hooks, BR-5 | BR-4 |
| SC-7 Artifact Metadata | BR-1 | BR-2 mirror, BR-5 | BR-4 |
| SC-8 Brain Graph Memory | BR-3 | BR-2 assist, BR-4 cache | BR-4 as truth |
| SC-9 Transient Cognition/STM | BR-4 | BR-5 snapshot | BR-1, BR-2, BR-3 as primary truth |

## 7) Technology candidates by role

- BR-1 candidate: LMDB-class embedded KV.
- BR-2 candidate: DuckDB-class append/tabular query store.
- BR-3 candidate: graph persistence backend (selection deferred until graph contract hardening).
- BR-4 candidate: Redis-class transient/hot cache store.
- BR-5 candidate: filesystem-based export/artifact surfaces.

No technology is "the Data Plane". Technology is role fit.

## 8) Redis position (explicit)

Redis is valid primary for BR-4 only:
- STM,
- activation state,
- hot graph cache.

Redis is not valid primary for:
- authority state,
- governance object truth,
- review/approval/apply truth,
- canonical event/evidence truth,
- persistent graph truth.

Guiding line:
- Redis for graph heat, not for graph sovereignty.

## 9) Separation rules
1. BR-4 never replaces BR-1 authoritative state.
2. BR-2 query store never replaces BR-1 truth for critical operational state.
3. BR-5 export never replaces any primary backend role.
4. BR-3 graph truth is distinct from BR-4 hot cache.
5. Governance lifecycle state cannot be primary-only in transient or export layers.

## 10) Repo anchors for implementation handoff
- `lib/core/workspace/`
- `lib/core/authority/`
- `lib/core/session/`
- `lib/exec/runtime/`
- `lib/exec/gates/storage_gate.c`
- `lib/law/mapping/decision_to_evidence.c`
- `lib/law/mapping/decision_to_audit.c`
- `lib/brain/memory/`
- `lib/brain/memory/graph/`
- `data/global/knowledge.db`
- `docs/program/23-runbooks/mind-redis-stm.md`

## 11) What is intentionally not decided yet
- final physical schema and all table definitions,
- final graph backend contract,
- full storage topology and migration plan,
- full operator query model,
- federation/replication/HA strategy.

## 12) Handoff to DP-3+
This runbook unlocks:
- DP-3 topology/layout,
- DP-4 event/evidence hardening,
- DP-5 governance persistence integration,
- DP-6 authority/artifact metadata integration,
- DP-7 brain graph + transient cognition backend integration.
