# Public Header Grammar

## Purpose
This document defines the single public header grammar for `yai/` during repository refoundation.

Canonical rule:
- public headers live under `include/yai/...`
- internal headers stay near implementations under `lib/...` (or temporary legacy module source roots) and are not exported.

## Public include tree
The public grammar is organized by semantic ownership:
- `include/yai/api/`
- `include/yai/core/`
- `include/yai/exec/`
- `include/yai/data/`
- `include/yai/graph/`
- `include/yai/knowledge/`
- `include/yai/protocol/`
- `include/yai/platform/`
- `include/yai/support/`

This tree replaces the legacy notion that each old top-level domain owns an independent public include surface.

## Public vs internal boundary
### Public
A header is public only if it defines a stable cross-module contract needed outside its implementation unit.

### Internal
A header is internal if it exposes:
- implementation helpers,
- backend state internals,
- parsing internals,
- private utility routines,
- unstable transitory glue.

Internal headers must be kept as module-local (`internal.h`, `*_internal.h`, or equivalent) and must not be mirrored into `include/yai/...`.

Examples intentionally kept internal in this phase:
- `kernel/include/yai_session_internal.h`
- `lib/knowledge/cognition/cognition_internal.h`
- `lib/graph/state/graph_state_internal.h`
- `lib/graph/state/graph_backend.h`

## Domain placement rules
### `api/`
Only globally stable operator/runtime surface contracts:
- version/config/status/runtime front-door contracts.

Do not dump module-specific headers in `api/`.

### `core/`
Authority/workspace/session/dispatch/lifecycle/enforcement/events/vault contracts.

### `exec/`
Execution-plane gates, engine bridge, transport client, execution runtime concerns.

### `data/`
Data-plane store/records/query/binding/lifecycle contracts.

### `graph/`
Graph truth, lineage, materialization and summary contracts.

### `knowledge/`
Cognition/reasoning/memory/provider contracts.

### `protocol/`
Wire/runtime message contracts and codec boundaries.

### `platform/`
OS/FS/UDS/clock wrappers and system adaptation boundaries.

### `support/`
Cross-cutting primitives: ids, errors, logger, paths, strings, arena.

## Transitional policy for this wave
This wave introduces the new grammar and thin public contracts.

Allowed transition approach:
- thin wrappers to legacy headers where migration is not complete,
- minimal placeholder contracts for still-unimplemented boundaries,
- explicit notes where headers are temporary compatibility bridges.

Not allowed:
- promoting internals to public just to simplify includes,
- widening API surface without stable ownership,
- using `api/` as a generic bucket.

## Legacy include sets mapping direction
Legacy include roots are migration sources only:
- `boot/include/`
- `root/include/`
- `kernel/include/`
- `engine/include/`
- `mind/include/`
- `runtime-protocol/include/`

Final public grammar remains `include/yai/...`.

## Next-wave implications
This grammar prepares:
- extraction of `support/platform/protocol` foundations,
- migration of runtime sovereignty into `core`,
- migration of execution plane into `exec`,
- migration of cognition plane into `knowledge` and graph truth into `graph`.
