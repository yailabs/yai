# Canonical Data Plane Model (DP-1)

## Purpose
Define the canonical model for YAI data-plane refoundation before backend-specific implementation work.

This document is the DP-1 source of truth for:
- boundary model,
- storage class model (high-level),
- control/data path responsibilities,
- non-negotiable invariants,
- transition guardrails into DP-2..DP-9.

## Scope Boundary

### In scope (DP-1)
- Canonical terminology and model normalization.
- Separation between control-plane decisions and data-plane persistence actions.
- Workspace-scoped persistence boundary definition.
- High-level storage classes and ownership intent.
- Cross-repo responsibility map (`law`, `sdk`, `cli`, `yai`).

### Out of scope (DP-1)
- Concrete backend implementation (DB/graph engines).
- Distributed topology and replication.
- Full operator query surface and data APIs.
- Performance tuning and production HA design.
- Runtime federation and peer-owner control-plane topology.

## Canonical Model

### 1) Execution Cell
The workspace is the data-plane execution cell.  
All persistence operations are resolved inside a workspace boundary and must remain path-jail compliant.

### 2) Authority Mediation
No component writes durable state without governed mediation.  
`Kernel` is the authority gate; `Engine` performs sink operations only after governed dispatch.
Source-node daemons can feed acquisition payloads but do not own final sink authority.

YD-1 daemon boundary:
- `yai-daemon` can acquire, spool, retry, and deliver source-plane payloads;
- `yai-daemon` can mediate delegated local actions only under owner-issued scope;
- only owner runtime `yai` can accept payloads into canonical records/graph/policy/conflict truth.

RF-0.2 hierarchy rule:
- delegated edge policy plane is derived from owner global policy plane;
- delegated edge state cannot override canonical owner truth classes.

RF-0.3 enforcement outcome baseline:
- daemon-local enforcement outcomes are delegated runtime events
  (`observe_only|allow|block|hold|execute|escalate|defer|deny_due_to_*`);
- canonical policy/conflict adjudication remains owner-side.

RF-0.4 edge observation baseline:
- source-plane input is not asset-only; owner truth is grounded on governed edge
  observation of assets, processes, and runtime signals;
- observation scope does not imply mediation/enforcement scope;
- runtime observables (freshness, spool/retry pressure, connectivity,
  policy/grant staleness) are canonical owner-side decision inputs.

ER-2 local durability/resilience baseline:
- daemon-local spool/retry state is first-class for continuity under intermittent
  connectivity;
- spooled/retried units remain subordinate operational state until owner-side
  acceptance;
- edge continuity never upgrades into owner authority or canonical truth.

SW-1 workspace truth lock:
- owner workspace runtime is canonical truth plane for graph/db/case/provenance
  binding and final conflict adjudication;
- edge-local observations/outcomes/spool/snapshots remain non-canonical
  operational contributors until owner acceptance/canonicalization.

SW-2 workspace-to-edge distribution lock:
- owner distributes delegated operational artifacts (`policy_snapshot`,
  `grant`, `capability_envelope`) as target-aware data-plane records;
- distributed artifacts enable edge execution scope but never transfer owner
  canonical truth or final authority.

SW-3 delegated validity lifecycle lock:
- distributed edge material has explicit lifecycle states
  (`valid|refresh_required|stale|expired|revoked`);
- lifecycle deterioration contracts edge autonomy and fallback modes;
- revoke/expiry handling remains owner-authoritative and non-sovereign edge-side.

MF-A1 mesh foundation lock:
- mesh discovery and coordination are explicit runtime planes;
- mesh visibility/awareness do not imply authority or canonical truth transfer;
- sovereign authority and canonical truth classes remain owner workspace runtime.

MF-1 discovery data-plane lock:
- discovery records and advertisement descriptors are topology/bootstrapping
  artifacts, not trust or authority artifacts;
- owner discovery and peer discovery remain governed visibility classes;
- discovery state is preparatory input for enrollment/trust/coordination planes.

MF-2 coordination data-plane lock:
- workspace peer membership and owner peer-registry records are canonical
  coordination-plane classes;
- coordination classes expose distributed operation signals
  (coverage/overlap/order/replay/conflict pressure) without redefining final
  owner adjudication;
- coordination metadata is owner-anchored and not a sovereign truth transfer.

MF-3 sovereign authority lock:
- legitimacy/enrollment/trust states are canonical authority-plane classes;
- authority scope/suspension/revocation markers constrain participation without
  flattening sovereign authority;
- final authority and canonical adjudication remain owner workspace runtime
  classes.

MT-1 secure transport lock:
- remote overlay transport state is a first-class operational data-plane input
  (`endpoint availability`, `reachability`, `path freshness`, `degradation`,
  `reconnect required`);
- transport classes do not imply trust/enrollment/authority classes and cannot
  promote canonical truth ownership;
- secure connectivity enables delivery paths, not sovereign decision transfer.

MT-2 owner ingress lock:
- owner remote ingress classes are boundary-governed acceptance classes
  (`ready|degraded|restricted|unavailable` baseline);
- ingress decisions are legitimacy/scope/validity/contribution-class aware and
  may accept/restrict/defer/reject;
- ingress acceptance does not imply canonicalization, final adjudication, or
  sovereign truth transfer.

MT-3 overlay integration lock:
- overlay descriptors, target associations, endpoint mutation and path-state
  transitions are first-class distributed runtime classes;
- overlay-associated classes remain transport/deployment operational classes and
  cannot be interpreted as legitimacy/trust/authority classes;
- overlay-aware targeting improves remote operability without relocating owner
  canonical truth or final authority.

QG-1 query-surface lock:
- source/edge/mesh/delegation/transport/ingress/overlay classes are queryable as
  first-class owner-side inspect families;
- query summaries are normalized operational visibility planes, not final
  canonical adjudication planes;
- summary-oriented read models are required alongside low-level tails/signals.

QG-2 unified-graph lock:
- graph projection unifies workspace sovereign anchors with distributed runtime
  classes and relation families;
- projection must preserve state-layer semantics
  (`observed|accepted|canonicalized|degraded/conflict`);
- graph unification cannot be interpreted as distributed sovereign consensus.

QG-3 AI-grounding lock:
- governed grounding contexts are derived from owner-side inspect summaries and
  unified graph projections, not from unscoped raw runtime exhaust;
- grounding contexts include provenance/freshness/validity/adjudication markers;
- AI/agent outputs remain non-sovereign and cannot bypass owner adjudication.

### 3) Control/Data Path Separation
Mandatory path:

`cli/sdk -> runtime ingress -> kernel authority -> engine sink -> kernel -> reply`

Data-plane storage is not an independent side-channel and cannot bypass policy/lifecycle gates.

### 4) Storage Class Intent (DP-1 baseline)
DP-1 introduces storage class intent only (detailed role model in DP-2):
- authority state,
- governance/compliance state,
- event/evidence sinks,
- artifact/metadata state,
- transient cognition/graph sink state.

### 5) Runtime Contract Intent
All data-plane actions must expose deterministic outcomes through canonical reply semantics:
- explicit success/failure,
- stable reason code,
- trace/evidence pointer when available.

Source-plane extension (YD-3 baseline):
- source records are canonical runtime classes (`source_node`,
  `source_daemon_instance`, `source_binding`, `source_asset`,
  `source_acquisition_event`, `source_evidence_candidate`,
  `source_action_point`, `source_owner_link`) and stay owner-side persisted
  truth.

## Cross-Repo Responsibility Baseline

- `law`: canonical contracts, schemas, policy/lifecycle constraints.
- `yai`: authority mediation and runtime sink orchestration.
- `sdk`: consumer-safe data surface contracts.
- `cli`: operator-safe command surface and readable outcome summaries.

## Non-Negotiable Invariants

1. Workspace scope is mandatory for durable operations.
2. No direct backend write from CLI/SDK.
3. Policy/lifecycle gates cannot be bypassed by data sinks.
4. Deterministic error semantics are required.
5. Traceability links are mandatory for qualification claims.

## DP Transition Rules

### DP-1 -> DP-2
Allowed when storage class names and ownership boundaries are frozen.

### DP-1 -> DP-3
Allowed when topology contract is expressed as canonical layout and migration-ready model.

### DP-1 -> DP-4..DP-9
Allowed only through phase-specific closure checks; no backend shortcuts.

## Verification Hooks (DP-1 baseline)
- model consistency checks between runbook and architecture docs,
- drift checks on key boundary terms (`workspace`, `kernel authority`, `sink mediation`),
- contract anchor presence checks in docs/program.

DP-1 does not claim backend completion; it claims model refoundation readiness for DP-2.

Secure peering dependency note (NP-1):
- data-plane source records may be ingested over local trusted paths today.
- Internet/multi-site ingestion must be treated as dependent on secure peering plane
  readiness (separate from protocol payload correctness).
