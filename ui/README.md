# YAI Desktop Subsystem

`desktop/` is the native visual subsystem of YAI.

## Role
- provide native visual surfaces for operators
- host shell panels, navigation, inspector, and command surfaces
- expose backend-agnostic runtime contracts for compositor, renderer, input, and session state

## Boundary
- `kernel/`: execution substrate and low-level primitives
- `sys/`: system service planes consumed by desktop surfaces
- `user/`: command shell and operator command surfaces
- `sdk/`: typed client contracts consumed by desktop runtime and shell panels

`desktop/` must not contain:
- web app stacks
- react/tsx runtime logic
- external app-host topology

## Legacy Surface Mapping
- `OverviewView.tsx` -> `shell/panels/overview/`
- `EventsView.tsx` -> `shell/panels/events/`
- `GraphView.tsx` -> `shell/panels/graph/`
- `LawView.tsx` -> `shell/panels/policy/`
- `LogsView.tsx` -> `shell/panels/logs/`
- `MindView.tsx` -> `shell/panels/overview/`
- `ProvidersView.tsx` -> `shell/panels/providers/`
- `ShellView.tsx` -> `shell/panels/command/`
- `ChatView.tsx` -> `shell/panels/command/`
- `CommandCenter.tsx` -> `shell/panels/command/`
- `InspectorDrawer.tsx` -> `shell/inspector/`
- `LeftRail.tsx` -> `shell/navigation/`
- `Topbar.tsx` -> `shell/navigation/`
- `FeedList.tsx` -> `shell/panels/events/`
