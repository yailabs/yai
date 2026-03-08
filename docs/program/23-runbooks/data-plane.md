# RB-DATA-PLANE — Data Plane

> **Status:** Draft | **Revision:** 2 | **Owner:** Runtime | **Effective date:** 2026-03-01  
> **Supersedes:** RB-DATA-PLANE@rev1

---

## Dependencies & References

**Depends on:**
- RB-ROOT-HARDENING
- RB-WORKSPACES-LIFECYCLE
- RB-ENGINE-ATTACH

**ADR refs:**
- `docs/program/22-adr/ADR-006-unified-rpc.md`
- `docs/program/22-adr/ADR-011-contract-baseline-lock.md`
- `docs/program/22-adr/ADR-012-audit-convergence-gates.md`

**Related specs:**
- `deps/yai-law/specs/protocol/include/transport.h`
- `deps/yai-law/specs/protocol/include/auth.h`
- `deps/yai-law/specs/protocol/include/audit.h`
- `deps/yai-law/specs/engine/schema/engine_cortex.v1.json`

**Runbooks:**
- `docs/program/23-runbooks/root-hardening.md`
- `docs/program/23-runbooks/workspaces-lifecycle.md`
- `docs/program/23-runbooks/engine-attach.md`
- `docs/program/23-runbooks/mind-redis-stm.md`

**Test plan:** `docs/40-qualification/test-plans/hardfail.md`

**Tools:** `yai-check-pins` · `yai-verify` · `yai-gate` · `yai-suite`

---

## 1. Purpose

Deliver the staged data-plane rollout for YAI with:

- **Deterministic tenant isolation** (workspace-scoped storage + strict path jail)
- **Storage layout contract** (one canonical location per workspace)
- **Operational verification** with closure gates per phase
- **Audit-ready evidence capture** (command outputs, logs, artifacts)

> **SC102-grade:** no Mind dependency required to ship v5.0–v5.2. v5.3 is optional and **must not** become a runtime hard dependency.

---

## 2. Scope

### In scope
- Workspace storage layout under `~/.yai/run/<ws_id>/`
- Kernel authority store (L1) with minimal stable ABI
- Engine event store (L2) for audit + forensics inputs
- CLI surface that accesses storage **only through Kernel governance**
- Per-phase acceptance, rollback discipline, and artifact evidence

### Out of scope
- SC103 graph causal queries and semantic layers (Mind)
- High-throughput distributed storage / multi-node clustering
- Cross-workspace queries (explicitly disallowed)

---

## 3. Preconditions (Non-negotiable)

All MUST be true before starting any v5 phase:

- [ ] Workspace lifecycle is stable and enforced (create / open / close / destroy)  
  → `docs/program/23-runbooks/workspaces-lifecycle.md`
- [ ] Centralized `ws_id` validation is active (Root / Kernel side)
- [ ] Path jail is active and applied to **all** file operations under a workspace
- [ ] Engine attach is stable and produces real traffic L0 / L1 / L2  
  → `docs/program/23-runbooks/engine-attach.md`
- [ ] Multi-stream logger is operational (kernel + engine logs at minimum)
- [ ] Baseline lock policy is in force for `deps/yai-law` (pins aligned)  
  → `docs/program/22-adr/ADR-011-contract-baseline-lock.md`

---

## 4. Data Plane Model (L1 / L2 / L3)

| Layer | Component | Responsibilities |
|-------|-----------|-----------------|
| **L1** | Kernel | Authority, identity, session, permissions (fast lookup, minimal records) |
| **L2** | Engine | Event store (append + query for audit / metrics / debug) |
| **L3** | Mind *(optional, SC103)* | Short-term memory / context cache (Redis STM) |

**Rule:** L3 must never be required to boot/run SC102. If Redis is absent, system must continue in `DEGRADED_READONLY` where applicable — never undefined behavior.

---

## 5. Canonical Filesystem Layout (Workspace Scoped)

Workspace run directory: `~/.yai/run/<ws_id>/`

```
~/.yai/run/<ws_id>/
├── manifest.json          # authoritative storage inventory
├── authority/
│   ├── data.mdb           # kernel store (LMDB)
│   └── lock.mdb
├── events/
│   ├── events.duckdb      # engine store
│   └── export/            # optional (parquet / jsonl)
├── engine/
│   ├── control.sock       # runtime control surfaces
│   └── engine.pid
└── logs/
    ├── kernel.log
    ├── engine.log
    └── root.log           # optional
```

> **Isolation rule:** no path in the data-plane may escape `~/.yai/run/<ws_id>/` once resolved through path jail.

---

## 6. Phased Rollout (v5.0 → v5.4)

Phases are **sequential**. Each phase MUST close with acceptance + evidence bundle.  
No cross-phase partial merges.

### Suggested branch naming

```
feat/data-plane-v5.0-layout
feat/data-plane-v5.1-authority-lmdb
feat/data-plane-v5.2-events-store
feat/data-plane-v5.3-stm-optional
feat/data-plane-v5.4-cli-surface
```

### Artifact locations

- **Milestone packs:** `docs/program/24-milestone-packs/data-plane/MP-DATA-PLANE-v5.x.y.md`
- **Evidence:** `docs/program/24-milestone-packs/data-plane/evidence/waveX-YYYY-MM-DD/`

### Closure gates (every phase)

At phase closure, capture:

```bash
tools/bin/yai-check-pins
tools/bin/yai-verify list
tools/bin/yai-verify core
```

Also include: hardfail test plans, command transcripts for create/open + storage checks.  
Store outputs as evidence artifacts alongside `.exit` codes.

---

## 7. v5.0 — Layout + Manifest (No DB logic)

### Objective

Freeze the workspace storage contract:
- where things live
- how isolation is guaranteed
- how inventory is declared (manifest)

### Expected changes

**Normative spec in yai-law:**
- `deps/yai-law/specs/runtime/DATA_PLANE.md`

**Workspace layout helpers:**
- Kernel: `kernel/src/core/project_tree.c` (extend) OR new `kernel/src/core/storage_paths.c`
- Header: `kernel/include/yai_storage_paths.h` (new)

**`manifest.json` emitted** on `ws create`.

### manifest.json — Minimum contract

Path: `~/.yai/run/<ws_id>/manifest.json`

```json
{
  "ws_id": "testws",
  "created_at": "2026-03-01T10:30:00Z",
  "storage": {
    "authority": { "type": "lmdb",   "path": "authority/",          "abi": "v1" },
    "events":    { "type": "duckdb", "path": "events/events.duckdb", "abi": "v1" },
    "stm":       { "type": "redis",  "prefix": "yai:testws",         "optional": true }
  }
}
```

### Acceptance v5.0

- [ ] `manifest.json` is created on `ws create`
- [ ] All data-plane paths are built via path-jail helpers
- [ ] No file operation can escape the workspace run root
- [ ] Evidence captured: command output + logs

---

## 8. v5.1 — L1 Kernel Authority Store (LMDB, minimal ABI)

### Objective

Implement a minimal, stable authority store used by Kernel for:
- session → principal
- principal → role / capabilities
- workspace → ownership metadata

No large payloads. No event analytics. Fast lookup only.

### Storage placement

- **Directory:** `~/.yai/run/<ws_id>/authority/`
- **LMDB files:** `data.mdb`, `lock.mdb`

### ABI record v1

```c
#define YAI_AUTHORITY_ABI_V1 0x00000001

struct yai_authority_record_v1 {
    uint32_t abi_version;     // MUST be YAI_AUTHORITY_ABI_V1
    uint64_t created_at_unix; // seconds
    uint32_t permissions;     // bitmask
    uint8_t  role;            // guest / user / operator / sovereign
    uint8_t  reserved[7];     // alignment / future use
    uint8_t  payload[224];    // fixed payload for v1
} __attribute__((packed));
```

> **LMDB safety rule:** LMDB returns pointers valid only while the transaction is alive. Kernel MUST copy any record out of `MDB_val` before closing / aborting transactions.

### Kernel interfaces

**New header:** `kernel/include/yai_storage_lmdb.h`  
**New impl:** `kernel/src/storage/storage_lmdb.c`

Minimum API:

| Function | Description |
|----------|-------------|
| `yai_lmdb_open(ws_id)` | Open / create db for workspace |
| `yai_lmdb_get(key, out)` | Read record |
| `yai_lmdb_put(key, rec)` | Write record |
| `yai_lmdb_del(key)` | Delete record |
| `yai_lmdb_close()` | Close db |

### Acceptance v5.1

- [ ] LMDB opens per workspace and creates files in authority dir
- [ ] get / put / del works on 1K records
- [ ] No dangling pointer usage (copy-before-close enforced)
- [ ] Evidence captured: `yai-verify` outputs + log excerpts

---

## 9. v5.2 — L2 Engine Event Store (DuckDB baseline)

### Objective

Introduce an append-first event store usable for:
- **Audit convergence** (who did what, when, with what decision)
- **Operational metrics** (deny / quarantine counts, latency)
- **Debugging** (queryable state snapshots if needed)

### Storage placement

- **Main file:** `~/.yai/run/<ws_id>/events/events.duckdb`
- **Optional export:** `~/.yai/run/<ws_id>/events/export/*`

### Why DuckDB

- SQL inspection is critical while stabilizing schemas
- Handles moderate workloads well
- Easy export to Parquet later

> RocksDB / LSM can be introduced later if throughput demands it.

### Minimal schema v1

**Table `events`** (append-only):

| Field | Type | Description |
|-------|------|-------------|
| `ts_unix_ms` | BIGINT | Timestamp in ms |
| `trace_id` | TEXT | Trace identifier |
| `ws_id` | TEXT | Workspace identifier |
| `plane` | TEXT | `root` / `kernel` / `engine` |
| `action` | TEXT | Command id or action name |
| `decision` | TEXT | `allow` / `deny` / `quarantine` |
| `reason_code` | TEXT | From yai-law |
| `actor` | TEXT | Principal / session |
| `payload_json` | TEXT | Compact JSON |

**Minimum indexes:**
- `events(ts_unix_ms)`
- `events(trace_id)`
- `events(action)`

### Writer model (engine)

- One writer thread per workspace OR a shared writer with per-ws queues
- Batch inserts (e.g., 256–1024 events per transaction)
- Flush policy: time-based (e.g., 250ms) and size-based

### Acceptance v5.2

- [ ] Engine can append events for a live run
- [ ] `SELECT COUNT(*)` and `WHERE trace_id=...` works
- [ ] Storage ops are path-jail safe
- [ ] Evidence captured: SQL query output + logs

---

## 10. v5.3 — L3 STM (Redis, OPTIONAL)

### Objective

Provide a short-term memory cache for SC103 without becoming an SC102 runtime dependency.

### Rules

- Redis absence MUST NOT break SC102
- If Redis missing: return `DEGRADED_READONLY` for STM endpoints and continue normally

### Keying

| Category | Pattern | TTL |
|----------|---------|-----|
| STM keys | `yai:<ws_id>:stm:*` | 10–60 minutes |
| Pinned keys | `yai:<ws_id>:context:*` | None (or long TTL) |

### Acceptance v5.3

- [ ] Keys are workspace-prefixed
- [ ] TTL enforced on STM keys
- [ ] Absence of Redis yields controlled degraded behavior (no crash)

---

## 11. v5.4 — Unified Data CLI (Kernel-governed)

### Objective

CLI never touches storage directly. All storage access goes through:

```
CLI → Root → Kernel → (storage) → response
```

### Initial commands

```bash
yai data stats --ws testws
yai data authority get --ws testws --key "session:sess_001"
yai data events tail   --ws testws --limit 100
yai data events query  --ws testws --sql "SELECT decision, COUNT(*) FROM events GROUP BY 1"
```

### Acceptance v5.4

- [ ] CLI commands work end-to-end through Kernel governance
- [ ] Authorization enforced (`DENY_AUTH` for unauthorized role)
- [ ] Evidence captured: command outputs + logs

---

## 12. Failure Modes (Must fail-closed)

| Failure | Action |
|---------|--------|
| Cross-tenant path leakage or side effects | Block merges, fix path-jail, rerun test plans |
| Storage schema drift between docs/specs and implementation | Update spec + implementation atomically, rerun gates |
| Redis unavailable (v5.3) | Must degrade safely, **never** crash SC102 |

---

## 13. Rollback

1. Revert only the active phase branch
2. Restore last known green data-plane baseline
3. Rerun build + verify + hardfail tests before reopening the phase

---

## 14. Traceability

**ADR refs:**
- `docs/program/22-adr/ADR-003-kernel-authority.md`
- `docs/program/22-adr/ADR-004-engine-execution.md`
- `docs/program/22-adr/ADR-006-unified-rpc.md`
- `docs/program/22-adr/ADR-011-contract-baseline-lock.md`
- `docs/program/22-adr/ADR-012-audit-convergence-gates.md`

**Milestone packs:** `docs/program/24-milestone-packs/data-plane/` — to be created per phase

---

*RB-DATA-PLANE rev2 — 2026-03-01*