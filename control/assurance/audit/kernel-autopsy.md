# Kernel Autopsy: `yai_runtime_legacy.tla`

## Findings
- Mixed concerns in a single file:
  - bootstrap/runtime state machine
  - authority constraints
  - compliance flag gates
  - energy accounting
  - external effect checks
- Imported `yai_ids_legacy` for vault offsets but with no module decomposition.
- Contained no explicit runtime policy/grant/containment submodels.

## Why It Is No Longer Canonical
- System architecture is no longer kernel-centric.
- Runtime is split into clear domains (`authority`, `policy`, `grants`, `containment`, `workspace`, `dispatch`).
- Governance resolution and protocol control semantics require independent modules.

## Action Taken
- Kernel monolith moved to `docs/transitional/formal-archive/formal-legacy/tla/yai_runtime_legacy.tla`.
- Replaced by decomposed module set under `control/assurance/modules/`.
- New root model: `control/assurance/models/yai_system.tla`.
