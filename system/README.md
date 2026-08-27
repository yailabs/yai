# C system source map

Authority: local navigation for current C implementation only. The executable
topology is owned by [`docs/architecture.md`](../docs/architecture.md).

`system/` implements the C ABI components declared under `include/yai/`:
base, Case/subject compatibility types, control, effects/carriers,
observations, records/journal, graph/index/memory/projection/reconciliation,
hot state, daemon IPC, and a Rust bridge adapter.

Most components are proven by C smoke tests. Normal `yaid` static linking pulls
the narrow daemon/fixture dependency path, not the complete archive. The Rust
CLI independently implements the current operational center. These facts make
the C tree behavior to characterize, not the automatic target topology.

Subdirectory names are compatibility/source organization and carry no
canonical semantic authority.
