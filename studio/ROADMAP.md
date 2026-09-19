# YAI Studio implementation roadmap

Authority: Studio-specific implementation progression and dependencies. The
repository [ROADMAP](../ROADMAP.md) remains the sole authority for global YAI
maturity, selection, promotion and interlock. This document cannot promote X04.
The canonical product/frontend architecture remains [docs/studio.md](../docs/studio.md).

Every Studio milestone begins with **Backend Sync Check** against current YAI
`origin/master`. The check records the last backend SHA reconciled by Studio,
the current backend SHA, relevant application/Case contract drift and the reason
for importing or rejecting any changed type. Internal backend types never flow
into Studio automatically.

## Earned foundation

| Milestone | Established boundary |
|---|---|
| Studio isolation / bootstrap | Isolated React, TypeScript, Vite and Tauri build surface inside YAI; core and CLI do not require Node or Tauri. |
| Offline Workbench shell | Case-first desktop regions, deterministic fixture client and explicit visual-regression mode. |
| Information architecture | Start Center, Case entry, Overview/Environment/Knowledge/Memory/Authority/Work/Compute and contextual panel organization. |
| UI foundation | Shared typography, surfaces, controls, graph primitives, navigation history and preview/pinned tabs. |
| Bounded live local Case vertical | Real authorized local Case list/open/summary projections and generation invalidation through the YAI application boundary; fixtures are opt-in only. |

These entries record implemented foundation. Their exact maturity remains the
root ROADMAP's decision.

## CURRENT

### `STUDIO.DESKTOP.SHELL.PTY.0`

Stabilize compact desktop chrome, command/menu plumbing, dense Case Explorer,
professional bottom tools and a real desktop-local integrated PTY. The PTY owns
only shell process, byte-stream and terminal-window mechanics. It does not own
Case, Workflow, execution, effect or provider semantics.

Dependencies: the existing bounded live local Case vertical, Tauri desktop
lifecycle and UI foundation. Browser mode must report that a desktop host is
required and must never emulate a shell.

The shell foundation keeps IDE-grade contribution seams explicit: command
identity is separate from menu/shortcut placement, and Activity Bar view
containers, work tabs, primary/auxiliary sidebars and bottom tools remain stable
regions. This prepares `STUDIO.DESKTOP.INFRA.0` without implementing a plugin or
extension host in the current milestone.

## NEXT

### `STUDIO.MEMORY.INSPECTOR.0`

Target traversable Case-object navigation with no dead nodes or edges, a rich
typed Inspector, horizontal temporal Memory canvas, Experience and Knowledge
graphs, graph ↔ timeline ↔ Inspector navigation and preserved Back/Forward
state. This milestone is recorded as next; no completion is implied here.

## HORIZON

- `STUDIO.CASE.EXPLORATION.0`
- `STUDIO.AUTHORITY.WORKFLOW.0`
- `STUDIO.YVEX.CONTROL.0`
- `STUDIO.DESKTOP.INFRA.0`
- `STUDIO.EXTERNAL.CLIENTS.0`
- `STUDIO.COMPUTER.USE.0`
- `STUDIO.REMOTE.MOBILE.0`

Horizon ordering is planning context, not selection, promotion or an automatic
backend wave. YVEX-specific management must preserve YAI's generic inference
provider boundary.
