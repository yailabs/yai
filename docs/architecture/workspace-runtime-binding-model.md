# Workspace Runtime Binding Model (6/8)

## Binding Source of Truth

Binding resolution order:

1. `YAI_ACTIVE_WORKSPACE` (explicit override)
2. cwd-mapped workspace (`~/.yai/run/*/manifest.json`, deepest `root_path` match)
3. `~/.yai/session/active_workspace.json` (compatibility fallback)

Binding status values:

- `active`
- `no_active`
- `stale`
- `invalid`

## Runtime Consumption

Runtime uses binding to resolve workspace context before control-call resolution.

Workspace commands (`current/status/inspect/domain/policy/debug/run`) consume this binding model.

In distributed acquisition v1, this binding truth remains owner-runtime (`yai`)
only; source nodes and subordinate `yai-daemon` runtimes do not become
independent binding truth.

YD-1 boundary lock:
- runtime binding is owner-side only;
- `yai-daemon` never sets canonical active workspace truth.

RF-0.2 delegation lock:
- edge-local binding/cache artifacts are delegated execution inputs only;
- they do not become workspace policy or binding truth.

RF-0.3 enforcement lock:
- local daemon binding/cache validity gates delegated enforcement eligibility;
- invalid/stale binding context cannot elevate local enforcement authority.

RF-0.4 observation lock:
- daemon can emit runtime/process observation state under owner binding context;
- observation visibility does not imply local action authority.

ER-3 edge binding/action-point lock:
- edge bindings are typed (`observational` vs `mediable`);
- binding, observation, mediation and enforcement scopes are explicit and distinct;
- action points are first-class source-plane entities, not implicit event side effects.

SW-1 authority/truth lock:
- workspace binding context is part of owner-side sovereignty and canonical truth
  boundary;
- edge-local binding/context caches are operational inputs only and cannot become
  final authority or canonical state.

SW-2 distribution lock:
- workspace binding context is the anchor for owner-to-edge policy snapshot and
  capability envelope targeting;
- distributed edge material remains delegated execution scope, never workspace
  binding truth ownership.

SW-3 validity lock:
- delegated material used under workspace binding context is time-bounded and
  revocable;
- stale/expired/revoked delegated state restricts edge mediation/enforcement
  behavior and never expands edge authority.

MF-A1 mesh lock:
- mesh peer visibility and coordination state can be consumed under workspace
  binding context;
- mesh awareness does not change workspace binding sovereignty or owner final
  authority semantics.

MF-1 discovery lock:
- workspace binding context can scope discovery visibility and bootstrap
  targeting;
- discovery-scoped node visibility does not imply enrollment/trust completion
  or delegated control authority.

MF-2 coordination lock:
- workspace binding context anchors coordinated peer membership and registry
  interpretation;
- coordination metadata (coverage/overlap/freshness/backlog/order/replay) is
  operational context and does not alter owner authority boundaries.

MF-3 authority lock:
- workspace binding context remains the owner-side anchor for enrollment/trust
  legitimacy interpretation;
- legitimacy or authority-scope contraction states (suspended/revoked) are
  sovereign boundary inputs, not peer-owned authority decisions.

## Workspace-local vs Runtime-global

Workspace-local:

- declared context
- inferred/effective summaries
- trace refs
- inspect summaries

Runtime-global (current tranche):

- law embedded corpus
- runtime process primitives
- service-level ingress

Runtime-aware, workspace-scoped:

- effective resolution snapshot
- authority/evidence summaries
- effect outcome references
