# C system source map

Authority: local navigation for current C implementation only. The executable
topology is owned by [`docs/architecture.md`](../docs/architecture.md).

`system/` contains the C implementation needed by `yaid` plus separately built
component-characterization mechanics: base, Case/subject compatibility types,
control, filesystem/process effects, observations, records/journal,
projection, hot state, and daemon IPC.

The production archive has an explicit 16-source membership; `yaid` adds its
entrypoint, IPC, and core loop. Component tests link a separate archive. C
graph/index/memory mirrors, synthetic dispatch/registry families, and the
smoke-only Rust bridge were removed because they had no product caller or
unique uncharacterized property. The Rust command remains the operational
center.

Subdirectory names are compatibility/source organization and carry no
canonical semantic authority.
