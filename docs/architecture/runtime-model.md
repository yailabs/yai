# Runtime Model

`yai` is the runtime host of the platform and executes the internal modules `core`, `exec`, `data`, `graph`, and `knowledge`.

For distributed source acquisition v1, `yai` remains the centralized owner
runtime while edge acquisition/execution is handled by standalone subordinate
`yai-daemon` runtimes under owner-issued scope.

## YD-1 Topology Lock

The YD-1 refoundation slice locks these invariants:

- `yai` is the only owner runtime and canonical workspace source of truth.
- `yai-daemon` is the standalone subordinate edge runtime.
- topology is `distributed acquisition / centralized control`.
- daemon state is operational/advisory and never canonical workspace/policy/graph/conflict truth.
- `exec` is the active mediation layer for owner/daemon source-plane flows.
- edge delegated behavior is owner-issued, scope-limited, and revocable.
- delegated local enforcement follows explicit RF-0.3 outcome semantics.
- edge observation includes assets, processes and runtime signals (RF-0.4).

## Canonical runtime flow

1. Ingress request reaches runtime control surface.
2. Workspace identity/binding/context is resolved from active runtime workspace state.
3. Runtime builds classification context from the operation.
4. Embedded law is loaded and validated.
5. Discovery selects family/specialization policy context.
6. Normative stack is resolved (specialization + overlays + authority/evidence composition).
7. Final effect is handed to enforcement.
8. Decision/evidence trace shape is returned for downstream handling and workspace inspect surfaces.

## Repository boundaries

- Normative source of truth is in sibling repo `law`.
- `yai` consumes the runtime-facing export under `embedded/law/`.
- `ops` is qualification/publication bureau and not runtime normative authority.

## Scope note

This runtime model covers the unified runtime topology target where:
- `brain` is not a canonical subsystem.
- execution actors and orchestration are in `exec`.
- graph truth state is in `graph`.
- cognition/memory/provider substrate is in `knowledge`.
- persisted records/query/lifecycle are in `data`.
- source-plane transport mediation/handoff is an active `exec` responsibility.

Workspace model details are defined in `docs/architecture/workspace-model.md`.
Distributed acquisition topology details are defined in
`docs/architecture/distributed-acquisition-plane-model.md`.
YD-1 daemon refoundation lock is defined in
`docs/architecture/daemon-architecture-refoundation-model.md`.
RF-0.1 semantic refoundation lock is defined in
`docs/architecture/source-plane-model-refoundation-rf01.md`.
RF-0.2 policy hierarchy lock is defined in
`docs/architecture/global-to-edge-policy-hierarchy-model.md`.
RF-0.3 delegated edge enforcement model is defined in
`docs/architecture/delegated-edge-enforcement-model.md`.
RF-0.4 edge observation model is defined in
`docs/architecture/process-and-asset-runtime-observation-model.md`.
ER-2 edge resilience/state model is defined in
`docs/architecture/daemon-local-runtime-model.md`.
ER-3 edge binding/action-point model is defined in
`docs/architecture/edge-binding-and-action-point-model.md`.
SW-1 workspace authority/truth plane lock is defined in
`docs/architecture/workspace-authority-and-truth-plane-model.md`.
SW-2 workspace-to-edge policy distribution model is defined in
`docs/architecture/workspace-to-edge-policy-distribution-model.md`.
SW-3 delegated validity/expiry/refresh model is defined in
`docs/architecture/edge-policy-validity-expiry-refresh-model.md`.
MF-A1 governed sovereign mesh foundation is defined in
`docs/architecture/governed-sovereign-mesh-model.md`.
MF-1 mesh discovery foundation is defined in
`docs/architecture/mesh-discovery-foundation-model.md`.
MF-2 mesh coordination foundation is defined in
`docs/architecture/mesh-coordination-foundation-model.md`.
MF-3 sovereign mesh authority foundation is defined in
`docs/architecture/sovereign-mesh-authority-foundation-model.md`.
MT-1 secure overlay transport plane is defined in
`docs/architecture/secure-overlay-transport-plane-model.md`.
MT-2 owner remote peer ingress model is defined in
`docs/architecture/owner-remote-peer-ingress-model.md`.
MT-3 overlay integration model is defined in
`docs/architecture/overlay-integration-model.md`.
QG-1 source/edge query surface model is defined in
`docs/architecture/source-and-edge-query-surfaces-model.md`.
QG-2 unified graph model is defined in
`docs/architecture/unified-graph-workspace-edge-model.md`.
QG-3 AI grounding model is defined in
`docs/architecture/ai-grounding-governed-case-state-model.md`.
Source-plane entity/contract model is defined in
`docs/architecture/source-plane-model.md`.
Owner ingest runtime flow is defined in
`docs/architecture/source-owner-ingest-model.md`.
Daemon local runtime behavior is defined in
`docs/architecture/daemon-local-runtime-model.md`.
Source-plane query/graph read surfaces are defined in
`docs/architecture/source-plane-read-model.md`.

Secure peering baseline (NP-1):
- local runtime control ingress and remote peer ingress are distinct concerns.
- binary protocol semantics remain application-level and ride over secure peering transport.
- recommended non-local deployment path is private overlay peering (WireGuard/equivalent).

Secure overlay operationalization (NP-4):
- private overlay integration is the canonical deployment model for customer-grade
  non-local owner/peer operation.
- owner/peer bootstrap sequence is defined in
  `docs/program/23-runbooks/owner-peer-overlay-bootstrap.md`.

## SW-2 distribution lock

Owner runtime distributes delegated operating material to edge runtimes as
explicit source-plane artifacts (grant, snapshot, capability envelope, target
association, delegated scopes). Distribution enables bounded edge behavior but
does not transfer owner truth, sovereign policy ownership, or final
adjudication.

## SW-3 validity lifecycle lock

Delegated edge material (grants/snapshots/capability envelopes) carries explicit
validity lifecycle state (`valid|refresh_required|stale|expired|revoked`) and
lifecycle timestamps. As delegated material weakens, edge autonomy contracts
(`full_delegated` -> `restricted_hold_escalate` -> `observe_only` /
`disabled_by_revoke`); it never expands.

## MF-A1 governed sovereign mesh lock

YAI is mesh-native in topology (discovery + coordination across owner/peer
nodes) while remaining sovereign in authority. Discovery/visibility/peer
awareness do not transfer canonical policy truth, final adjudication, or
workspace sovereignty from owner runtime.

## MF-1 discovery lock

Mesh discovery is an explicit governed visibility/bootstrap plane. Node
discoverability and topology visibility enable bootstrap and coordination
handoff, but do not imply trust, enrollment completion, delegated permission,
or sovereign authority transfer.

## MF-2 coordination lock

Mesh coordination is a distinct plane that manages governed membership,
owner-anchored registry state, awareness metadata, and distributed
coverage/overlap/order/replay coordination signals. Coordination improves
distributed operation but does not transfer final authority or canonical truth
from owner runtime.

## MF-3 sovereign mesh authority lock

Sovereign authority within mesh context remains owner-anchored. Enrollment
finalization, trust bootstrap, legitimacy recognition, authority scope
limitation, and final adjudication do not migrate to peer visibility or
coordination planes.

## MT-1 secure overlay transport lock

Secure remote connectivity is a dedicated transport plane. Private overlay
reachability and endpoint visibility enable owner/peer communication, but do
not imply enrollment completion, trust legitimacy, delegated authority, or
canonical truth transfer.

## MT-2 owner remote ingress lock

Owner remote peer endpoint is a governed ingress boundary. Reachability enables
peer presentation, while ingress logic decides accept/restrict/defer/reject
based on legitimacy, delegated scope, validity state, and contribution class.

## MT-3 overlay integration lock

Private secure overlay is integrated as native distributed deployment substrate
for owner/peer runtime targeting and path-state visibility. Overlay identity
and reachability remain transport concerns and never replace enrollment, trust,
delegated scope, ingress governance, or workspace sovereignty.

## QG-1 source/edge query lock

Owner runtime exposes governed inspect/query surfaces for source, edge, mesh,
delegated scope, ingress, transport, and overlay state. Structured inspect
visibility is distinct from raw signals and from final canonical truth.

## QG-2 unified graph lock

Owner runtime projects a unified graph that correlates sovereign workspace/case
anchors with governed distributed state (edge/mesh/delegation/transport/
ingress/overlay). Graph representation keeps observed/accepted/canonicalized
adjudication boundaries explicit.

## QG-3 governed AI grounding lock

AI/agent execution context is projected from owner-governed inspect and graph
surfaces as task-scoped grounded context. Grounded AI remains non-sovereign and
cannot replace owner final authority/canonical truth.
