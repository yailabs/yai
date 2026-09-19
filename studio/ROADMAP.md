# YAI Studio implementation roadmap

Authority: Studio-specific implementation progression and dependencies. The
repository [ROADMAP](../ROADMAP.md) remains the sole authority for global YAI
maturity, engineering selection, promotion and interlock. This document cannot
promote X03 or X04. The canonical product architecture is
[docs/studio.md](../docs/studio.md); current run/build behavior is documented in
[README.md](README.md).

Every Studio implementation milestone begins with a **Backend Sync Check**
against current YAI `origin/master`. Record the last backend SHA reconciled by
Studio, the current SHA, relevant changes to application projections, mutations,
authority, Recall/W, providers, Sources and Resources, and why each changed type
is imported or rejected. Backend internals never become frontend contracts by
proximity.

Program postures below are implementation planning facts, not root maturity
states. `CURRENT`, `TARGET` and `OPEN` deliberately distinguish executable
behavior from selected architecture and unfinished work.

## Current sequence

```text
architecture formalization
        ↓
resident Local Host foundation
        +
Workbench Kernel foundation
        ↓
Memory / Inspector / Navigation
        ↓
Participants / Environment
        ↓
Authority / Work
        ↓
Compute / Providers
        ↓
YVEX control
```

The host and Workbench Kernel programs may progress in parallel where their
contracts are independent. Later programs may overlap only after their required
application and Workbench seams exist.

- **CURRENT:** `STUDIO.PRODUCT.ARCHITECTURE.0` freezes product topology and the
  internal Workbench architecture. It does not implement either target.
- **NEXT implementation foundations:** S1 Product Host & Lifecycle and S2
  Workbench Kernel.
- **NEXT feature program after those footholds:** S4 Memory / Inspector /
  Navigation.
- **HORIZON:** S5 through S12, subject to root ROADMAP selection and interlocks.

## Earned milestones

| Milestone | Established boundary |
|---|---|
| Studio isolation / bootstrap | React, TypeScript, Vite and Tauri build independently inside YAI; core and CLI do not require Node or Tauri. |
| Offline Workbench shell | Case-first regions, explicit fixtures and deterministic visual-regression routes. |
| Information architecture | Start Center, Case entry and the Overview/Environment/Knowledge/Memory/Authority/Work/Compute perspectives. |
| UI foundation | Shared tokens and controls, navigation history, preview/pinned tabs and graph primitives. |
| Bounded live local Case vertical | Authorized Case list/open/summary projections and generation invalidation through `yai-application`; fixtures remain opt-in. |
| Desktop shell and PTY | Compact desktop chrome, command/menu plumbing and a real transient local PTY on the qualified Linux path. |

These are bounded implementation facts. Their maturity remains whatever the
root ROADMAP says.

## S0 — Product Foundation

**Posture:** established foundation; architectural target selected.

- **Purpose:** keep Studio Case-first, inside the YAI product, and free of
  duplicate semantic ownership.
- **Established:** isolated builds, bounded LiveClient/FixtureClient split,
  UI foundation, typed local read vertical and explicit product documentation.
- **Target properties:** one documented product topology, stable ownership
  boundaries and consistent `CURRENT / TARGET / OPEN` language.
- **Dependencies:** executable YAI owners and root ROADMAP authority.
- **Current gaps:** application coverage and lifecycle remain bounded.
- **Completion boundary:** documentation and implementation agree on ownership;
  future programs can extend known seams without redefining Case truth.
- **Non-goals:** product maturity promotion or a new semantic layer.

## S1 — Product Host & Lifecycle

**Posture:** target selected; implementation open.

- **Purpose:** provide one resident YAI Local Host per `YAI_HOME` / local
  profile for Studio, CLI and future clients.
- **Established:** in-process `yai-application`, a bounded local
  `RuntimeInstance`, and historical evidence for secure local discovery and IPC.
- **Target properties:** discover/start/attach, explicit readiness and shutdown,
  application service, supervised RuntimeInstance, scoped update service,
  attachments and real host telemetry.
- **Dependencies:** current application owners, transport/security
  qualification and the `yaid` compatibility archaeology.
- **Current gaps:** no resident product host, host commands, autostart, complete
  event hub, CLI convergence or multi-client lifecycle qualification.
- **Completion boundary:** independently launched clients share one secured
  same-profile host without moving Case semantics into transport or desktop code.
- **Non-goals:** global cross-environment authority or automatic provider/model
  startup.

## S2 — Workbench Kernel

**Posture:** partial shell seams; kernel implementation open.

- **Purpose:** make desktop regions and frontend infrastructure stable hosts for
  internally authored YAI contributions.
- **Established:** activity rail, sidebar, work tabs, context bar, bottom panel,
  command/menu foothold, navigation history and shared visual primitives.
- **Target properties:** Platform services, registered view containers/views,
  editor inputs/groups, panels, context keys, keybindings, menus, configuration,
  lifecycle and layout services.
- **Dependencies:** S0 ownership rules and existing UI foundation.
- **Current gaps:** feature components still compose several regions directly;
  context keys and generalized editor/view contribution seams are absent.
- **Completion boundary:** built-in features contribute through stable internal
  seams while the Kernel remains ignorant of Case feature semantics.
- **Non-goals:** public extension SDK, marketplace or third-party compatibility.

## S3 — Desktop Shell & Terminal

**Posture:** partial, advanced bounded implementation.

- **Purpose:** provide compact native desktop chrome and a truthful integrated
  human shell.
- **Established:** application menus, icon Activity Bar, dense Explorer,
  resizable panels, xterm renderer, terminal-specific Tauri bridge and
  multi-instance `portable-pty` lifecycle.
- **Target properties:** qualified native behavior on supported platforms and
  integration with the stable S2 panel/command infrastructure.
- **Dependencies:** Tauri lifecycle, shared controls and platform-specific PTY
  qualification.
- **Current gaps:** only Linux is interactively qualified; sessions do not
  persist and shells are not Case-attached.
- **Completion boundary:** supported desktop targets pass terminal lifecycle,
  resize, input, cleanup and native-shell acceptance through the Kernel.
- **Non-goals:** PTY as Case execution/evidence, Open in Terminal or automatic
  command injection.

## S4 — Memory / Inspector / Navigation

**Posture:** next feature program after S1/S2 footholds.

- **Purpose:** make Case objects and relations traversable across Memory,
  graphs, Inspector and work surface.
- **Established:** derived read projections, graph primitives, local
  Back/Forward and preview/pinned tabs.
- **Target properties:** typed inspectors, no dead objects or edges, horizontal
  temporal canvas, Experience and Knowledge graphs, and preserved contextual
  navigation.
- **Dependencies:** adequate typed application projections and S2 editor,
  Inspector, navigation and context seams.
- **Current gaps:** current inspectors and temporal views remain bounded and
  several relations cannot resolve a navigable endpoint.
- **Completion boundary:** every displayed meaningful object/relation resolves
  through a typed Inspector and coordinated timeline/graph/work navigation.
- **Non-goals:** a canonical frontend graph store or causality inferred from
  chronology.

## S5 — Participants / Environment

**Posture:** horizon; bounded read foundation exists.

- **Purpose:** expose who participates and what material/operational world is
  attached to the Case.
- **Established:** real Participant summaries and Source/Resource/file
  presentation from qualified application views.
- **Target properties:** rich Participant, Source and Resource inspection,
  acquisition posture, provenance and explicit unavailable backing.
- **Dependencies:** S1 application coverage, S2 views and S4 Inspector
  traversability.
- **Current gaps:** mutations, external observation and complete resource
  families are not available.
- **Completion boundary:** ordinary Case exploration needs no filesystem scan or
  private persistence read in Studio.
- **Non-goals:** browser-owned source admission or a second filesystem truth.

## S6 — Authority / Work

**Posture:** horizon; bounded read foundation exists.

- **Purpose:** present who may do what and how governed work progresses.
- **Established:** limited policy/review/decision/workflow/execution/effect facts
  can appear in live summaries.
- **Target properties:** typed authority chains, review actions, workflow
  definition versus progression, executions, effects, artifacts and handoffs.
- **Dependencies:** shared application mutations, S2 contributions and S4 typed
  relation navigation.
- **Current gaps:** no mutation-complete Studio API or qualified general live
  workflow/event stream.
- **Completion boundary:** Studio and CLI expose the same admitted operations
  and current authority without duplicating evaluation.
- **Non-goals:** policy evaluation, Workflow semantics or effect authority in
  React.

## S7 — Compute / Providers

**Posture:** horizon; generic read posture is partial.

- **Purpose:** expose provider/model/runtime capability through generic YAI
  provider semantics.
- **Established:** bounded sanitized target/model/locality/availability facts.
- **Target properties:** provider discovery, configuration, qualification,
  capability, refusal and evidence through typed YAI application operations.
- **Dependencies:** S1 host/application coverage, provider governance and S2/S4
  presentation seams.
- **Current gaps:** no complete live provider setup or management contract.
- **Completion boundary:** generic providers can be inspected and configured
  without secrets, invented telemetry or provider-brand execution branches.
- **Non-goals:** making provider runtime state canonical Case truth.

## S8 — YVEX Control

**Posture:** horizon; unimplemented.

- **Purpose:** add first-party management of public YVEX capabilities while
  preserving generic inference.
- **Established:** YVEX can appear only through the generic provider posture
  currently exposed by YAI.
- **Target properties:** model inventory, admitted packages, engines,
  load/unload, devices, deployment, compilation, residency, sessions and
  authoritative evidence/telemetry.
- **Dependencies:** S7 generic plane plus real versioned YVEX management
  contracts and permissions.
- **Current gaps:** those management contracts are not exposed to Studio.
- **Completion boundary:** qualified native management coexists with unchanged
  generic cognitive execution.
- **Non-goals:** direct YVEX-internal reads or YVEX ownership of Case semantics.

## S9 — Desktop Infrastructure

**Posture:** horizon; basic native shell exists.

- **Purpose:** make Studio a maintainable installed desktop product.
- **Established:** Tauri boot/window/build and Linux desktop PTY qualification.
- **Target properties:** packaging, signing, updater policy, crash diagnostics,
  platform services and resident-host integration/autostart.
- **Dependencies:** S1 lifecycle and stable S2/S3 shell contracts.
- **Current gaps:** production packaging/signing/updating and macOS/Windows
  qualification.
- **Completion boundary:** supported platforms install, launch, update and
  recover through explicit secure lifecycle policy.
- **Non-goals:** application semantics in native adapters.

## S10 — External Clients

**Posture:** horizon; unimplemented as a public contract.

- **Purpose:** let structured first-party or qualified external clients consume
  the same application meaning as Studio and CLI.
- **Established:** bounded internal Rust application foothold only.
- **Target properties:** versioned contracts, conformance, authentication,
  disclosure and explicit compatibility policy for selected clients.
- **Dependencies:** S1 resident host and root-level interface selection.
- **Current gaps:** no stable exported interface package, SDK or remote
  transport qualification.
- **Completion boundary:** a real independent consumer passes conformance
  without parsing CLI output or reading persistence.
- **Non-goals:** automatic public plugin platform or semantic delegation.

## S11 — Computer Use

**Posture:** horizon; unimplemented.

- **Purpose:** present governed computer targets, observations, actions and
  receipts as a universal fallback carrier.
- **Established:** product/authority principles only.
- **Target properties:** qualified targets, frames, admitted actions, review,
  evidence and execution receipts observed through Studio.
- **Dependencies:** YAI Computer Use owners, S6 authority/work and S4 Inspector.
- **Current gaps:** no selected action/observation/transport contract.
- **Completion boundary:** Studio can observe and control admitted Computer Use
  without renderer-owned authority.
- **Non-goals:** replacing structured integrations with mouse/screenshot
  automation.

## S12 — Remote / Mobile

**Posture:** horizon; unimplemented.

- **Purpose:** extend Case access beyond the trusted local desktop while keeping
  the same application meaning.
- **Established:** none beyond local ownership and target architecture.
- **Target properties:** remote authentication, transport security, disclosure,
  reconnect/resync, mobile interaction and multi-client qualification.
- **Dependencies:** S1 and S10 plus independent security/interlock selection.
- **Current gaps:** all remote serving and mobile delivery contracts.
- **Completion boundary:** selected remote/mobile clients pass authorization,
  ordering, revocation and recovery tests against the same Case owners.
- **Non-goals:** LAN exposure by default or global authority across YAI_HOME
  environments.

## Planning invariants

- Closing Studio must not become Case closure, host shutdown or background-work
  cancellation.
- A PTY is a human shell surface, not a Case execution or evidence channel.
- Provider processes have independent lifecycle from the resident host.
- The Workbench is internally contribution-driven; no public plugin platform is
  selected.
- Program completion never promotes X03/X04 without root ROADMAP evidence and
  an explicit root decision.
