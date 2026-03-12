# Formal Audit: Current State

Date: 2026-03-11

## Inventory Before Refoundation
- Primary executable model was `control/assurance/tla/yai_runtime_legacy.tla` (monolith).
- `yai_precedence_legacy.tla` and `yai_resolution_legacy.tla` were placeholders.
- `yai_ids_legacy.tla` was generated from vault ABI and mixed with runtime semantics.
- Config axis was kernel-centric (`yai_runtime_legacy*.cfg`).
- Traceability file referenced split-era paths (`contracts/*`, `brain`, `exec`).

## Structural Gaps
- Formal layer did not reflect runtime canonical domains (`policy`, `grants`, `containment`).
- No explicit bridge matrix between formal invariants and runtime enforcement outcomes.
- No modular ownership by architecture domain.
- No formal check entrypoints for quick/deep/enforcement-focused runs.

## Canonical Target
- Module-driven formal spine aligned to current runtime/governance/protocol/workspace.
- Explicit enforcement linkage artifacts.
- Kernel framing relegated to `docs/transitional/formal-archive/formal-legacy/` only.
