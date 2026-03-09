---
id: RB-MIND-REDIS-STM
title: Mind Redis STM
status: draft
owner: runtime
effective_date: 2026-02-19
revision: 1
supersedes: []
depends_on:
  - RB-ENGINE-ATTACH
  - RB-DATA-PLANE
adr_refs:
  - docs/program/22-adr/ADR-005-mind-proposer.md
  - docs/program/22-adr/ADR-008-connection-lifecycle.md
decisions:
  - docs/program/22-adr/ADR-005-mind-proposer.md
  - docs/program/22-adr/ADR-008-connection-lifecycle.md
related:
  adr:
    - docs/program/22-adr/ADR-005-mind-proposer.md
    - docs/program/22-adr/ADR-008-connection-lifecycle.md
  specs:
    - ../law/contracts/protocol/include/transport.h
    - ../law/contracts/protocol/include/auth.h
  test_plans:
    - ops/evidence/qualification/test-plans/hardfail.md
  tools:
    - tools/bin/yai-verify
    - tools/bin/yai-gate
tags:
  - runtime
  - mind
  - redis
---

# RB-MIND-REDIS-STM — Mind Redis STM

> Historical note: this runbook contains pre-cutover topology references (`root/kernel/engine/mind`).
> It is not authoritative for current runtime ingress. Canonical flow is `cli -> sdk -> yai` through `~/.yai/run/control.sock`.
> Data-plane role note (DP-2): this runbook is backend-role specific (`BR-4` transient/hot-cache) and is not the canonical Data Plane model source.

## 1) Purpose
Define a tenant-safe, deterministic short-term memory model for Mind using Redis with strict `ws_id` scoping and governed attach workflow.

### DP positioning
- Redis is valid primary for transient cognition / STM only.
- Redis is not authoritative primary for authority state, governance lifecycle truth, canonical event/evidence truth, or graph truth persistence.

## 2) Preconditions
- [ ] Root/Kernel attach path is stable for Mind clients.
- [ ] Redis runtime endpoint and security model are defined.
- [ ] Workspace isolation controls are active.

## 3) Inputs
- Mind runtime/client modules and Redis integration code
- Keyspace schema and TTL policy definitions
- Validation tooling: verify/gate commands + integration vectors

## 4) Procedure
Execute the staged sequence in this document: lifecycle contract, Redis topology, keyspace contract, implementation, and closure checks.

## 5) Verification
- Run startup/attach checks and Redis health checks.
- Validate keyspace isolation, TTL behavior, and degraded-mode behavior.

## 6) Failure Modes
- Symptom: Redis outage causes uncontrolled runtime failure.
  - Fix: enforce deterministic degraded mode with explicit logging.
- Symptom: cross-tenant key leakage appears.
  - Fix: enforce `yai:<ws_id>:...` keyspace guards and reject non-scoped keys.

## 7) Rollback
- Stop mind/redis test processes, restore previous runtime behavior, and re-run baseline control-plane checks.
- Revert only active phase changes before retrying.

## 8) References
- ADR: `docs/program/22-adr/ADR-005-mind-proposer.md`
- Runbooks: `docs/program/23-runbooks/engine-attach.md`, `docs/program/23-runbooks/data-plane.md`
- Test plans: `ops/evidence/qualification/test-plans/hardfail.md`

## Traceability
- ADR refs:
  - `docs/program/22-adr/ADR-005-mind-proposer.md`
  - `docs/program/22-adr/ADR-008-connection-lifecycle.md`
- MPs (to be filled as phases ship):
  - `docs/program/24-milestone-packs/...`

## Appendix — Detailed Operational Notes (Legacy Detailed Content)

### YAI L3 Mind Redis STM v5.3 — Operational Runbook

**Branch:** `feat/data-plane-v5.3-redis`  
**Dependencies:** v2/v3 (ws_id validation + path jail) + v4 (handshake+attach real) + v5.1 (LMDB authority) recommended  
**Objective:** Mind stateless + STM in Redis, always tenant-scoped (`ws_id`), zero cross-tenant leakage.

---

## Objective

Mind becomes stateless process with Redis as Short-Term Memory:
- Mind is **workspace-scoped** and has **no authority**
- All state lives in Redis with strict `ws_id` namespacing
- Proposals-only workflow through Root→Kernel
- Graceful degradation if Redis unavailable

---

## Pre-flight (Always)

```bash
make clean
make
pkill -f yai-root-server || true
pkill -f yai-boot || true
pkill -f yai-engine || true
pkill -f yai-mind || true
pkill -f redis-server || true  # if using dedicated instance
yai-boot --master
yai root ping
```

---

## STEP 0: Mind Lifecycle Contract (ADR-008 + ADR-005)

### Mandatory startup sequence

Mind is a "workspace-scoped" process with **no authority**.

**Startup sequence (MUST follow exactly):**

1. Read `ws_id` (from env or argument)
2. Connect to Root socket: `~/.yai/run/root/control.sock`
3. Handshake (`YAI_CMD_HANDSHAKE`)
4. Attach (`YAI_CMD_ATTACH`) with `ws_id` + `client_kind = MIND`
5. Health check Redis (socket/URL)
6. Load "pinned context" from Redis (if exists)
7. Enter cognitive loop (propose-only)

**Rule:** If Redis is down → Mind can start in "degraded mode" (zero STM), but MUST log WARN and reduce capabilities (no caching).

### Files to document

**ADR:** `docs/program/22-adr/ADR-005-mind-proposer.md`

Document the contract: Mind never operates without successful attach to Kernel.

### Acceptance
- [ ] Mind exits with error if attach fails
- [ ] Mind continues in degraded mode if Redis unavailable
- [ ] All startup steps logged with ws_id

---

## STEP 1: Redis Topology (Safe, Simple, Tenant-Safe)

### Recommended choice
**Single local Redis (shared instance) + rigid ws_id namespacing**

### Endpoint (recommended)
- **Socket:** `~/.yai/run/redis/redis.sock`
- **Config:** `YAI_REDIS_SOCK=~/.yai/run/redis/redis.sock`

**Why Unix socket:**
- Lower latency
- Better security (no network exposure)
- Easier path jail validation

### Alternative (heavier)
- Socket per workspace: `~/.yai/run/<ws_id>/redis.sock`
- Requires spawning Redis per WS (defer to v6/v7 if really needed)

### Redis startup

**If using dedicated instance:**

```bash
# Create redis config
mkdir -p ~/.yai/run/redis

cat > ~/.yai/run/redis/redis.conf <<EOF
unixsocket ~/.yai/run/redis/redis.sock
unixsocketperm 700
port 0
bind 127.0.0.1
daemonize no
save ""
appendonly no
maxmemory 256mb
maxmemory-policy allkeys-lru
EOF

# Start Redis
redis-server ~/.yai/run/redis/redis.conf &
```

### Acceptance
- [ ] Redis accessible via Unix socket
- [ ] `redis-cli -s ~/.yai/run/redis/redis.sock ping` returns PONG

---

## STEP 2: Canonical Keyspace (L3)

### Prefix convention
```
P = yai:<ws_id>
```

### STM (Volatile - Short-Term Memory)
```
P:stm:msg:<conv_id>:<seq>     # string/json, TTL 10-60m
P:stm:working:<trace_id>      # hash, TTL 10-60m
P:stm:summary:<conv_id>       # string, TTL 1-6h
```

### Context (Pinned - Persistent)
```
P:context:active              # hash/json, NO TTL (or long like 7d)
P:context:pins                # set, NO TTL
```

### Index/Health
```
P:meta:last_seen_ts           # string, TTL 24h
P:meta:mind_pid               # string, TTL 24h
P:meta:schema_version         # string, NO TTL
```

### Iron-clad rules
1. **No key without ws_id in name** (EVER)
2. **Never use generic patterns** like `yai:*` in production (only test/admin)
3. **All STM keys have TTL** (garbage collection built-in)
4. **Pinned context versioned** with schema_version

### Files to create

**Spec:** `../law/storage/REDIS_KEYSPACE.md`

Document the canonical keyspace with examples and TTL policies.

### Acceptance
- [ ] Keyspace spec committed
- [ ] All keys include ws_id
- [ ] TTL policies documented

---

## STEP 3: Mind Implementation (Rust) — Real File Targets

### A) Config module

**File:** `mind/src/config/mod.rs`

```rust
use std::env;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct MindConfig {
    pub ws_id: String,
    pub root_sock: String,
    pub redis_sock: Option<String>,
    pub redis_url: Option<String>,
    pub stm_ttl: Duration,
    pub context_ttl: Duration,
}

impl MindConfig {
    pub fn from_env() -> Result<Self, String> {
        let ws_id = env::var("YAI_WS_ID")
            .map_err(|_| "YAI_WS_ID not set")?;
        
        let root_sock = env::var("YAI_ROOT_SOCK")
            .unwrap_or_else(|_| {
                format!("{}/.yai/run/root/control.sock", 
                        env::var("HOME").unwrap_or("/tmp".into()))
            });
        
        let redis_sock = env::var("YAI_REDIS_SOCK").ok();
        let redis_url = env::var("YAI_REDIS_URL").ok();
        
        Ok(Self {
            ws_id,
            root_sock,
            redis_sock,
            redis_url,
            stm_ttl: Duration::from_secs(3600),      // 1h
            context_ttl: Duration::from_secs(604800), // 7d
        })
    }
}
```

### B) Kernel client (Root socket)

**File:** `mind/src/runtime/kernel_client.rs`

```rust
use std::os::unix::net::UnixStream;
use std::io::{Read, Write};

pub struct KernelClient {
    stream: UnixStream,
    ws_id: String,
}

impl KernelClient {
    pub fn connect(sock_path: &str, ws_id: String) -> Result<Self, String> {
        let stream = UnixStream::connect(sock_path)
            .map_err(|e| format!("Failed to connect: {}", e))?;
        
        Ok(Self { stream, ws_id })
    }
    
    pub fn handshake(&mut self) -> Result<(), String> {
        // Implement YAI_CMD_HANDSHAKE protocol
        // (use protocol from ../law/contracts/protocol/include/transport.h)
        todo!("Implement handshake")
    }
    
    pub fn attach(&mut self, client_kind: &str) -> Result<(), String> {
        // Implement YAI_CMD_ATTACH with ws_id + client_kind=MIND
        todo!("Implement attach")
    }
    
    pub fn send_proposal(&mut self, proposal: &[u8]) -> Result<(), String> {
        // Send proposal through kernel
        todo!("Implement proposal send")
    }
}
```

### C) Redis STM module

**File:** `mind/src/cache/redis_stm.rs`

```rust
use redis::{Client, Connection, Commands};
use std::time::Duration;

pub struct RedisStm {
    conn: Connection,
    ws_id: String,
    enabled: bool,
}

impl RedisStm {
    pub fn connect(sock_or_url: &str, ws_id: String) -> Result<Self, String> {
        let client = Client::open(sock_or_url)
            .map_err(|e| format!("Redis client error: {}", e))?;
        
        let conn = client.get_connection()
            .map_err(|e| format!("Redis connection error: {}", e))?;
        
        Ok(Self {
            conn,
            ws_id,
            enabled: true,
        })
    }
    
    pub fn connect_degraded(ws_id: String) -> Self {
        // No-op client for degraded mode
        Self {
            conn: unsafe { std::mem::zeroed() }, // FIXME: use Option
            ws_id,
            enabled: false,
        }
    }
    
    fn key(&self, kind: &str, parts: &[&str]) -> String {
        let mut k = format!("yai:{}:{}", self.ws_id, kind);
        for p in parts {
            k.push(':');
            k.push_str(p);
        }
        k
    }
    
    pub fn put_stm_msg(&mut self, conv_id: &str, seq: u64, data: &[u8], ttl: Duration) 
        -> Result<(), String> 
    {
        if !self.enabled { return Ok(()); }
        
        let key = self.key("stm:msg", &[conv_id, &seq.to_string()]);
        self.conn.set_ex(&key, data, ttl.as_secs() as usize)
            .map_err(|e| format!("Redis set error: {}", e))
    }
    
    pub fn get_recent(&mut self, conv_id: &str, limit: usize) 
        -> Result<Vec<Vec<u8>>, String> 
    {
        if !self.enabled { return Ok(vec![]); }
        
        // Scan for keys matching pattern
        let pattern = self.key("stm:msg", &[conv_id, "*"]);
        // ... implement scan logic ...
        todo!("Implement get_recent")
    }
    
    pub fn load_pinned_context(&mut self) -> Result<Option<Vec<u8>>, String> {
        if !self.enabled { return Ok(None); }
        
        let key = self.key("context", &["active"]);
        let data: Option<Vec<u8>> = self.conn.get(&key)
            .map_err(|e| format!("Redis get error: {}", e))?;
        Ok(data)
    }
    
    pub fn save_pinned_context(&mut self, data: &[u8]) -> Result<(), String> {
        if !self.enabled { return Ok(()); }
        
        let key = self.key("context", &["active"]);
        self.conn.set(&key, data)
            .map_err(|e| format!("Redis set error: {}", e))
    }
    
    pub fn touch_meta(&mut self, pid: u32) -> Result<(), String> {
        if !self.enabled { return Ok(()); }
        
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let ts_key = self.key("meta", &["last_seen_ts"]);
        let pid_key = self.key("meta", &["mind_pid"]);
        
        self.conn.set_ex(&ts_key, now.to_string(), 86400)?;
        self.conn.set_ex(&pid_key, pid.to_string(), 86400)?;
        
        Ok(())
    }
}
```

### D) Workspace awareness (path jail for Mind)

**File:** `mind/src/workspace/mod.rs`

```rust
use std::path::{Path, PathBuf};

pub struct WorkspaceCtx {
    pub ws_id: String,
    pub ws_root: PathBuf,
}

impl WorkspaceCtx {
    pub fn new(ws_id: String) -> Result<Self, String> {
        let home = std::env::var("HOME")
            .map_err(|_| "HOME not set")?;
        
        let ws_root = PathBuf::from(format!("{}/.yai/run/{}", home, ws_id));
        
        if !ws_root.exists() {
            return Err(format!("Workspace does not exist: {}", ws_id));
        }
        
        Ok(Self { ws_id, ws_root })
    }
    
    pub fn ensure_path_inside(&self, target: &Path) -> Result<PathBuf, String> {
        let canonical = target.canonicalize()
            .map_err(|e| format!("Path canonicalize error: {}", e))?;
        
        let ws_canonical = self.ws_root.canonicalize()
            .map_err(|e| format!("Workspace canonicalize error: {}", e))?;
        
        if !canonical.starts_with(&ws_canonical) {
            return Err(format!("Path jail violation: {:?} not under {:?}", 
                              canonical, ws_canonical));
        }
        
        Ok(canonical)
    }
}
```

<a id="phase-mind-proposer"></a>
### E) Proposer (proposal-only)

**File:** `mind/src/proposals/mod.rs`

```rust
use crate::runtime::KernelClient;

pub struct Proposal {
    pub ws_id: String,
    pub operation: String,
    pub payload: Vec<u8>,
}

impl Proposal {
    pub fn new(ws_id: String, operation: String, payload: Vec<u8>) -> Self {
        Self { ws_id, operation, payload }
    }
    
    pub fn to_bytes(&self) -> Vec<u8> {
        // Serialize to protocol format
        todo!("Implement serialization")
    }
}

pub fn send_proposal(client: &mut KernelClient, proposal: Proposal) 
    -> Result<(), String> 
{
    let bytes = proposal.to_bytes();
    client.send_proposal(&bytes)
}
```

### F) Main entry point

**File:** `mind/src/main.rs`

```rust
mod config;
mod runtime;
mod cache;
mod workspace;
mod proposals;

use config::MindConfig;
use runtime::KernelClient;
use cache::RedisStm;
use workspace::WorkspaceCtx;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Load config
    let config = MindConfig::from_env()?;
    
    println!("Mind starting for ws_id: {}", config.ws_id);
    
    // 2. Connect to Root/Kernel
    let mut kernel = KernelClient::connect(&config.root_sock, config.ws_id.clone())?;
    
    // 3. Handshake
    kernel.handshake()?;
    println!("Handshake OK");
    
    // 4. Attach
    kernel.attach("MIND")?;
    println!("Attached as MIND to ws: {}", config.ws_id);
    
    // 5. Connect to Redis (with degraded fallback)
    let mut stm = if let Some(sock) = config.redis_sock {
        match RedisStm::connect(&format!("unix://{}", sock), config.ws_id.clone()) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("WARN: Redis unavailable: {}", e);
                eprintln!("Running in degraded mode (no STM)");
                RedisStm::connect_degraded(config.ws_id.clone())
            }
        }
    } else {
        RedisStm::connect_degraded(config.ws_id.clone())
    };
    
    // 6. Load pinned context
    if let Some(ctx) = stm.load_pinned_context()? {
        println!("Loaded pinned context: {} bytes", ctx.len());
    }
    
    // 7. Touch meta (heartbeat)
    stm.touch_meta(std::process::id())?;
    
    // 8. Enter cognitive loop
    loop {
        // TODO: Implement proposal loop
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
```

---

## STEP 4: Robustness — Failure Modes (Non-Negotiable)

### Redis down
- Log WARN + set `stm_enabled=false`
- Mind continues without STM (does not crash loop)
- Proposals continue (but lower quality)

### Redis data corrupted / schema mismatch
If `P:meta:schema_version` != expected:
- Log WARN
- Fallback: ignore STM and recreate minimal "active context"
- Update `schema_version`

### ws_id mismatch / attach reject
- Mind MUST exit with error (must not operate without attach)

### Acceptance
- [ ] Mind starts in degraded mode if Redis unavailable
- [ ] Mind logs WARN clearly
- [ ] Mind exits if attach fails

---

## STEP 5: Integration Points

### Mind does NOT read/write LMDB/DuckDB
- Mind talks to Root/Kernel only
- Redis is "L3 memory only", not authority and not audit

### Mind is workspace-aware by construction
- Every Redis key contains ws_id
- Every proposal contains ws_id
- Every path in proposal passes path-jail (Mind-side)

### Acceptance
- [ ] No direct LMDB/DuckDB access from Mind
- [ ] All proposals include ws_id
- [ ] Path jail validates all paths in proposals

---

## STEP 6: Test Matrix v5.3

### Smoke tests
1. Mind start with valid ws_id + Redis up → OK
2. Mind start with invalid ws_id → FAIL FAST
3. Mind start with Redis down → OK (degraded) + WARN log
4. Mind attach → ok (kernel logs show `client_kind=MIND`)
5. STM write: set/get + TTL verified
6. Pinned context persist: write → restart → reload ok
7. Cross-tenant check: start 2 minds (ws A/B) → keys don't collide

### Scripts

**A) Test script:** `tests/e2e/test_mind_stm.sh`

```bash
#!/bin/bash
set -e

WS_ID=${1:-testws}

echo "=== Testing Mind STM for ws: $WS_ID ==="

# Start Redis if needed
if ! redis-cli -s ~/.yai/run/redis/redis.sock ping 2>/dev/null; then
    echo "Starting Redis..."
    redis-server ~/.yai/run/redis/redis.conf &
    sleep 1
fi

# Start runtime
yai-boot --master

# Create workspace
yai kernel ws create $WS_ID

# Start Mind
YAI_WS_ID=$WS_ID yai-mind &
MIND_PID=$!

sleep 2

# Verify keys exist
echo "Checking Redis keys for ws: $WS_ID"
redis-cli -s ~/.yai/run/redis/redis.sock --scan --pattern "yai:$WS_ID:*"

# Verify meta keys
META_PID=$(redis-cli -s ~/.yai/run/redis/redis.sock get "yai:$WS_ID:meta:mind_pid")
echo "Mind PID in Redis: $META_PID"
echo "Actual Mind PID: $MIND_PID"

# Cleanup
kill $MIND_PID
yai kernel ws destroy $WS_ID --arming --role operator

echo "=== Test PASS ==="
```

**B) Cross-tenant test:** `tests/e2e/test_mind_isolation.sh`

```bash
#!/bin/bash
set -e

echo "=== Testing Mind cross-tenant isolation ==="

# Start 2 workspaces
yai kernel ws create ws_alice
yai kernel ws create ws_bob

# Start 2 minds
YAI_WS_ID=ws_alice yai-mind &
ALICE_PID=$!

YAI_WS_ID=ws_bob yai-mind &
BOB_PID=$!

sleep 2

# Count keys per workspace
ALICE_KEYS=$(redis-cli -s ~/.yai/run/redis/redis.sock --scan --pattern "yai:ws_alice:*" | wc -l)
BOB_KEYS=$(redis-cli -s ~/.yai/run/redis/redis.sock --scan --pattern "yai:ws_bob:*" | wc -l)

echo "Alice keys: $ALICE_KEYS"
echo "Bob keys: $BOB_KEYS"

# Verify no cross-contamination
CROSS_KEYS=$(redis-cli -s ~/.yai/run/redis/redis.sock --scan --pattern "yai:ws_alice:*ws_bob*" | wc -l)
if [ $CROSS_KEYS -gt 0 ]; then
    echo "FAIL: Cross-tenant key contamination detected!"
    exit 1
fi

# Cleanup
kill $ALICE_PID $BOB_PID
yai kernel ws destroy ws_alice --arming --role operator
yai kernel ws destroy ws_bob --arming --role operator

echo "=== Test PASS: Isolation verified ==="
```

### Acceptance
- [ ] All smoke tests pass
- [ ] Cross-tenant isolation test passes
- [ ] No key collisions between workspaces
- [ ] Degraded mode works without Redis

---

## STEP 7: Definition of Done (v5.3)

- [ ] Mind does handshake+attach always
- [ ] Redis STM active with canonical keyspace
- [ ] Pinned context survives Mind restart
- [ ] Degraded mode if Redis down
- [ ] No cross-tenant collision
- [ ] Clear logs (ws_id always present)
- [ ] Test matrix passes

---

## Next Enhancement (v5.3.1)

**Versioned pinned context:**

Make pinned context a versioned object with `schema_version` + `abi_version`, so it doesn't break when you change struct.

```rust
#[derive(Serialize, Deserialize)]
struct PinnedContext {
    abi_version: u32,      // 0x00000001
    schema_version: u32,   // incremental
    created_at: u64,
    data: serde_json::Value,
}
```

---

## Current State Check

**Tell me:**
1. Where does your Mind entry point live today? (`mind/src/main.rs`?)
2. Do you already have Root client code? (If so, which file?)
3. Are you using `redis-rs` crate or different Redis client?

I'll map your existing code to this runbook's structure.
