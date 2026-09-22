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

Every Studio implementation milestone also includes a **Product Quality Pass**
for the Workbench regions and Surfaces it materially changes. The pass checks
component ownership, visual hierarchy, typography, spacing, icon alignment,
semantic color, focus/hover/selection, surface separation, keyboard behavior,
empty/error/unavailable states, resizing, duplicated controls and proportionate
visual regression. It is not permission to redesign unrelated regions, and it
does not replace executable semantic evidence.

Program postures below are implementation planning facts, not root maturity
states. `CURRENT`, `TARGET` and `OPEN` deliberately distinguish executable
behavior from selected architecture and unfinished work.

Canonical status vocabulary, shared with the root YAI/YVEX roadmaps:

| Mark | Meaning |
|---|---|
| 🟢 **ESTABLISHED** | Executable bounded foundation is published and qualified. |
| 🟡 **PARTIAL** | Useful executable behavior exists; named closure work remains. |
| 🔴 **OPEN** | Selected work or a required contract is not established. |
| ⚪ **LATER** | Explicit horizon; not selected for current implementation. |

## Studio execution board

**ESTABLISHED — Root reliability.** Window-local
Case interaction sessions now retain drafts, tabs and navigation across
attachment/refetch failures. Exact material identity and bytes gate buffer
initialization; dirty previews pin, duplicate opens reuse the existing tab,
and editor undo survives renderer changes. PTYs survive Panel/Case switching;
native close shares the unsaved-draft guard. Startup fits the monitor work area;
CodeMirror reuses the Tauri style nonce under the production CSP. Modal focus and menu shortcuts use
shared Workbench owners. The reliability suite qualifies these interactions at
1600×960, 1440×900, 1280×800 and 1000×650; its instrumented desktop bridge is
separate from native PTY acceptance.

**ESTABLISHED — Populated Case navigation and presentation.** Knowledge exposes
searchable, bounded document/unit/entity/topic/contradiction collections. Every
qualified graph endpoint is retained, including explicitly unresolved references;
pages and neighborhoods state their visible/total relation counts. Inspector
shows exact projected Knowledge text and navigable provenance. Overview now
provides compact perspective destinations; Authority includes already-projected
grants and the latest recorded decision, without implying a new authorization.
Activity discloses its latest-20 limit and links to the full Timeline.
This is a bounded read/product-quality foothold, not S4 completion.

**CURRENT — Capability Surface convergence and broader product quality.** The
last reconciled publication is `430b5cb8a32b7da77b461824454d2793ea8343cb` while the
execution-lifecycle worktree changes are independently owned. The running Host
can predate the checkout: discover its actual catalog before offering new
operations. The first authored action slice consumes Case create/cancel/close,
Participant role/self-link setup, Review approve/deny/defer and Workflow
HumanInput. `tests/studio/application-actions.mjs` qualifies these against a
real published Host on a disposable profile, including authority/bounds refusals,
projection refresh, replay and lost-acknowledgement non-resubmission. Settings >
Advanced shows actual advertised versus UI-integrated operations. This is
partial convergence, not governed Save, runtime supervision or full UI parity.

Environment now authors Source declarations (discovery and exposed SQLite/HTTP
names) and revocation through `source.declare`/`source.revoke`. Real Host tests
qualify refusals, immutable identity reuse, refresh and replay. Declaration does
not acquire content. The published Host has no `source.acquire` operation; the
concurrent execution-lifecycle implementation is not imported early. Resource
attachment still requires the qualified native binding/secure carrier: the
current raw `resource.attach` input is not a human-readable root-path setup API,
and Studio does not fabricate its digest/carrier in React. These are named
Application/Studio setup gaps, not missing Resource ownership.

Authority now authors `policy.case.bind`, `policy.case.replace` and
`policy.case.unbind`, with exact artifact/binding references, explicit reasons
and a captured generation. Real Host tests qualify each success and stale
refusal, unchanged policy on refusal, projection refresh and canonical replay.
Artifact import/lifecycle and Tenant-wide policy browsing remain named UI work.

**ESTABLISHED — Focused editing commands.** Editor, text-field and PTY clipboard
operations share the Workbench editing boundary. Menus retain focus/selection;
Undo/Redo follow actual history, read-only/terminal actions are gated, and delayed
clipboard results cannot enter a different file or terminal. Permission refusal
is visible. Browser regression and native Tauri editor/PTY acceptance qualify the
path; no Case or filesystem save is implied.

**ESTABLISHED — Material presentation and local Workbench quality.** Exact retained
bytes now feed image/PDF/audio/video renderers; Markdown is rendered with a lazy
trusted parser and qualified relative navigation. Markdown/SVG/CSV can preview
explicitly unsaved local drafts. CSV/TSV gains bounded rows, full-set search/sort
and accessible column sizing. Settings/layout share validated persistent values;
Edit menus restore the prior editing focus, and active tabs remain visible after
resizing. Real read-only Markdown acceptance
uses the durable Case at generation 64; heterogeneous binary acceptance uses
authored digest-qualified responses and does not claim new live binary Sources.
The backend baseline remains `0c9d934a6594c2b1e4c461bbe69c55e73cd8ccc6`;
concurrent execution-lifecycle work remains independently owned.

`CASE.PRODUCT.VERTICAL.COHERENCE.0` established
`case:studio-live-qualification` as the durable operator-owned product oracle.
`STUDIO.ENVIRONMENT.AUTHORING.0` has established the bounded Environment
interaction model: Files, Sources and Resources are distinct Surfaces, exact
material reads remain governed, and local editing stops at the current
application/authority boundary. Governed participant-origin Save remains an
explicit backend/application gap rather than a frontend workaround. The first
S1 foothold now supplies one resident application Host per `YAI_HOME`; deliberate
RuntimeInstance supervision remains the next S1 milestone.
The persistent Case is advanced only through normal YAI operations and is never
reset by Studio tests. `APPLICATION.CAPABILITY.OPERATION.PARITY.0` published at
`99c8d15d81a700a64d65fa779cbb96db6d22722f`; this bounded editor milestone
records its delta without opportunistically consuming the new operations. The
Studio sequence is:

```text
🟢 S1.PRODUCT.HOST.LIFECYCLE.0
        ↓
🟢 STUDIO.MATERIAL.IDENTITY.EDITOR.0
        ↓
🟡 STUDIO.CAPABILITY.SURFACE.CONVERGENCE.0
        ↓
🔴 S1.RUNTIME.SUPERVISION.1
```

| Lane | Owner | Current executable truth | Selected next milestone | Backend dependency | Status |
|---|---|---|---|---|---|
| Workbench | Studio S2 | One contribution-driven Workbench and universal Surface system; exact material identity fencing and the syntax-aware editor foundation are established | Capability Surface convergence, then feature-local Product Quality Passes | Typed presentation facts | 🟡 **PARTIAL** |
| Product Case vertical | Cross-surface qualification | Persistent Case with real Sources, Knowledge, policy, Workflow and generation refresh | Keep the product oracle coherent as owners grow | Normal CLI/domain operations | 🟢 **ESTABLISHED** |
| Application / X03 | `yai-application` plus existing owners | Published 61-operation catalog; Studio reads plus authored Case/Participant/Review/HumanInput actions | Continue action-level `STUDIO.CAPABILITY.SURFACE.CONVERGENCE.0` | Published typed owners; running Host version can lag checkout | 🟡 **PARTIAL** |
| Host / S1 | `application/yai-host` plus `yai-application` | Resident same-user Unix Host, singleton discovery, typed IPC, events, lifecycle CLI and Studio client | `S1.RUNTIME.SUPERVISION.1` after bounded capability convergence | Integrate the existing scheduler without duplicating its lease/execution owners | 🟡 **PARTIAL** |
| Memory / S4 | Transition/graph/Recall plus Studio | Bounded timeline and derived relations | Typed navigation, Inspector and temporal canvas | Resolvable typed relations | 🔴 **OPEN** |
| Environment / S5 | Sources/Resources plus Studio | Typed File/Source/Resource exploration, governed exact reads, renderer selection, exact material identity and syntax-aware local dirty/revert | Governed save after capability convergence | Participant-origin admitted file mutation and revision-conflict contract | 🟡 **PARTIAL** |
| Authority / S6 | Policy/Review/Grant/Workflow plus Studio | Bound policy/grant/decision reads; Review decisions and HumanInput forms with backend refusals | Policy management, workflow configuration and broader progression UX | Published typed operations; execution lifecycle remains separately owned | 🟡 **PARTIAL** |
| Compute / S7 | Provider governance plus Studio | Case-bound targets only; model/runtime/deployment facts remain distinct | Generic models/providers/targets experience | Provider discovery/configuration contracts | 🔴 **OPEN** |
| YVEX / S8 | YVEX public management plane | Generic inference posture only | Native management plane | Versioned public YVEX capabilities | ⚪ **LATER** |
| External / S10–S12 | Selected future owners | No public external/mobile contract | Later selection | S1 and explicit interface programs | ⚪ **LATER** |

### Capability delta awaiting Studio convergence

The published Application parity work adds or advances typed
operations for Case lifecycle, Participants, policy/review, Resources, Sources,
Workflow, handoff, identity/Tenant, Recall/working state and cognitive binding.
It also makes disconnect-safe execution and runtime supervision explicit
blockers. The editor milestone recorded that delta without consuming it.
Current convergence integrates only the authored actions above. Provider,
policy, Source/Resource setup, handoff and semantic/cognitive forms remain
Studio debt even when their operation is advertised. Runtime/source execution
worktree contracts are not imported before independent publication and sync.

## Surface/backend responsibility

| Studio surface | YAI owner | Application status | Missing-fact classification |
|---|---|---|---|
| Environment | Sources / Resources | Typed Sources/Resources and hierarchical files; exact retained reads through the authorized Source resolver | Participant-origin filesystem mutation, save receipt and stale-revision refusal are not qualified |
| Knowledge | M07 derived source knowledge | Bounded authorized derivation with retained backing | Rich navigation remains Studio/S4; absent derivation stays empty |
| Memory | Transition ledger, graph and Recall | Timeline bounded; Experience relations may be empty | Typed relation traversal remains S4 |
| Authority | Policy / Review / Grant | Bound facts plus Review approve/deny/defer | Policy management and complete chains remain S6; backend checks each decision |
| Work | Workflow / Execution | Definition/resolution plus participant HumanInput | Definition/binding/patch forms and execution management remain Studio/S6 debt |
| Compute | Provider governance | Case-bound targets partial | Model catalog, runtime/provider discovery and deployment management remain S7 |
| YVEX | External runtime | Generic inference target only | Native management is absent until S8 |
| Conversation | Committed Turns | Read-only | SEND remains an application mutation gap |

## Current sequence

```text
architecture formalization
        ↓
Workbench Kernel foundation
        ↓
Universal Work Surface and productization hardening
        ↓
persistent cross-surface Product Case coherence
        ↓
Environment exploration and local-authoring boundary
        ↓
resident application Host foundation
        ↓
material identity and editor hardening
        ↓
capability Surface convergence after Application parity publication
        ↓
RuntimeInstance supervision under product Host lifecycle
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

- **ESTABLISHED integration bridge:** `CASE.PRODUCT.VERTICAL.COHERENCE.0`
  maintains one persistent real Case as the CLI/application/Studio product
  oracle after the bounded S2 foundation.
- **ESTABLISHED, bounded:** `STUDIO.ENVIRONMENT.AUTHORING.0` provides distinct
  File, Source and Resource Surfaces, hierarchical qualified files, exact
  governed material reads, trusted `Open With…`, and a local dirty/revert
  editor. Persistence remains partial because YAI exposes no admitted
  participant-origin filesystem-content mutation or stale-revision contract.
- **ESTABLISHED S1 foothold:** one Rust-owned resident application Host serves
  independently launched local clients through private versioned Unix IPC;
  Studio discovers/starts/attaches and no longer embeds `LocalApplication`.
- **ESTABLISHED, bounded:** `STUDIO.MATERIAL.IDENTITY.EDITOR.0` fences exact
  material reads, rejects late cross-material responses, replaces the textarea
  foothold with a lazy syntax-aware editor and keeps governed Save explicitly
  unavailable.
- **NEXT after Application parity publication:**
  `STUDIO.CAPABILITY.SURFACE.CONVERGENCE.0`, reconciling typed capabilities
  through existing Workbench contributions.
- **THEN:** `S1.RUNTIME.SUPERVISION.1`, integrating the existing bounded
  `RuntimeInstance` lifecycle without replacing its lease, fairness or recovery
  owners.
- **NEXT feature program after S1 runtime supervision:** S4 Memory / Inspector /
  Navigation, using the established S2 seams.
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
| Workbench Kernel | One shell for live and fixture Case data; scoped Platform services, registered internal views/surfaces/panels/auxiliary views and host-capability-driven Terminal. |
| Universal Work Surface | Surface Input/Group/Registry mechanics; shared preview/pinning/navigation for perspective, text, Markdown, image, PDF, table, timeline, graph and Settings surfaces; qualified media-type resolution. |
| Workbench productization | Surface roles/capabilities and read-only posture; Command Palette, Quick Open and renderer-owned search; singleton searchable Settings with versioned local preferences; trusted structured text, image/vector, PDF, table, audio/video and unknown-material paths; lazy heavy renderers, shared Panel toolbar seams and a resizable in-tool terminal instance pane. |
| Environment exploration / authoring boundary | File hierarchy from qualified projected paths; distinct Source and Resource Surfaces; authorized exact retained material read; trusted renderer selection; local text dirty/revert lifecycle with Save disabled until YAI qualifies participant-origin mutation and revision conflict handling. |
| Resident application Host foothold | One Linux-qualified Host per `YAI_HOME`; private Unix discovery/handshake, same-user attachment, application request forwarding, event fanout, telemetry and CLI lifecycle; native Studio auto-starts/attaches and survives independently from the Host. |
| Exact material identity and syntax editor | Request/response identity is fenced across Case, object, Source, revision, path, digest, generation, media type and bytes; late preview reads cannot initialize another file; CodeMirror supplies a lazy syntax-aware file editor while governed Save remains unavailable. |

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

**Posture:** resident application Host foothold established; runtime supervision open.

- **Purpose:** provide one resident YAI Local Host per `YAI_HOME` / local
  profile for Studio, CLI and future clients.
- **Established:** `application/yai-host` owns a resident Rust process with one
  `LocalApplication` per explicit `YAI_HOME`, a private `0600` Unix socket under
  a `0700` run root, owner/process-identity checked discovery, protocol/profile
  handshake, ephemeral client attachments, bounded logs, telemetry, generation
  observation/event fanout and graceful cleanup. `yai host status/start/stop/
  restart/logs/serve` use the same lifecycle library as Tauri. Native Studio
  discovers or starts the Host, reconnects/resyncs after replacement, and does
  not stop it when a window closes. Browser and explicit fixture paths remain
  independent. The existing bounded local `RuntimeInstance` remains separate.
- **Target properties:** discover/start/attach, explicit readiness and shutdown,
  application service, supervised RuntimeInstance, scoped update service,
  attachments and real host telemetry.
- **Dependencies:** current application owners, transport/security
  qualification and the `yaid` compatibility archaeology.
- **Current gaps:** `RuntimeInstance` is not yet supervised by the Host; ordinary
  domain CLI operations have not all converged through Host transport; macOS/
  Windows IPC, OS-login autostart, remote transport, durable event replay and
  multi-client mutation parity remain unqualified.
- **Completion boundary:** independently launched clients share one secured
  same-profile host and the existing RuntimeInstance is supervised without
  moving Case semantics or scheduler ownership into transport/desktop code.
- **Non-goals:** global cross-environment authority or automatic provider/model
  startup.

## S2 — Workbench Kernel

**Posture:** bounded horizontal foundation and exact-material editor hardening established.

- **Purpose:** make desktop regions and frontend infrastructure stable hosts for
  internally authored YAI contributions.
- **Established:** one `StudioApplication` and `WorkbenchKernel` for LiveClient
  and FixtureClient; scoped command, context-key, menu, keybinding,
  configuration, navigation, theme, lifecycle and host-capability services;
  registered View Containers/Views, Surface Inputs and one Surface Group, Panel,
  Auxiliary Bar and Inspector seam. Terminal is a built-in Panel contribution
  and desktop availability is independent from the Case data source.
- **Established hardening:** the central region is a universal Work Surface.
  Trusted renderer registration, media-aware material resolution and bounded UI
  capability declarations let heterogeneous representations use the same tabs,
  preview, pinning, close, navigation and Inspector selection seams.
- **Established productization:** Content, Projection, System and Interactive
  Case presentation roles remain frontend-only; renderer capabilities govern
  preview/singleton/search/zoom/edit affordances. Command Palette, qualified
  Quick Open, current-Surface search, fixture-only Case search proof, searchable
  singleton Settings and versioned local preferences use existing registries.
  Trusted renderers cover text/Markdown/structured text, browser-safe raster and
  inert SVG image presentation, PDF, table, native audio/video and an explicit
  unknown-material fallback. PDF and graph code load outside the initial shell
  chunk. Panel contributions can place compact actions in the shared header.
- **Target properties:** Platform services, registered view containers/views,
  Surface Inputs/Groups/renderers, panels, context keys, keybindings, menus,
  configuration, lifecycle and layout services. Trusted generic data surfaces
  and future declarative Case Views compose through these seams.
- **Dependencies:** S0 ownership rules and existing UI foundation.
- **Current gaps:** only one visible Surface Group is exposed; contribution
  contracts are internal and bounded to current consumers; live Case search,
  qualified external-open paths, split groups, richer typed Inspectors,
  Calendar/Form/Board/Chart renderers and any admitted persistence for shared
  declarative Case Views remain open.
- **Completion boundary:** the selected bounded horizontal boundary is earned:
  built-in
  features register through internal seams while the Kernel owns regions and
  remains ignorant of Case feature semantics. Later feature waves extend those
  seams and apply the permanent Product Quality Pass; continued minor S2 polish
  does not block S1.
- **Non-goals:** public extension SDK, marketplace, arbitrary generated
  JavaScript or a frontend-owned canonical View store.

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
- **Dependencies:** adequate typed application projections and S2 Surface,
  Inspector, navigation and context seams.
- **Current gaps:** current inspectors and temporal views remain bounded. A
  relation endpoint without projected detail is explicitly a reference, not a
  fabricated object; full typed resolution remains open.
- **Completion boundary:** every displayed meaningful object/relation resolves
  through a typed Inspector and coordinated timeline/graph/work navigation.
- **Non-goals:** a canonical frontend graph store or causality inferred from
  chronology.

## S5 — Participants / Environment

**Posture:** partial; bounded exploration and local authoring foundation exists.

- **Purpose:** expose who participates and what material/operational world is
  attached to the Case.
- **Established:** real Participant summaries; hierarchical file presentation;
  distinct typed Source and Resource Surfaces; exact retained material reads;
  trusted renderer selection; local dirty/revert editing without authority
  bypass.
- **Target properties:** rich Participant, Source and Resource inspection,
  acquisition posture, provenance and explicit unavailable backing.
- **Dependencies:** S1 application coverage, S2 views and S4 Inspector
  traversability.
- **Current gaps:** participant-origin governed file mutation, save receipts,
  expected-revision conflict refusal, finer repository/directory/file Source
  kinds, external observation and complete Resource families are not available.
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

- **Purpose:** expose the generic inference plane without flattening model,
  runtime/provider, deployment target and Case binding into one concept.
- **Established:** bounded sanitized facts for targets already bound to a Case,
  including their provider key, adapter/runtime, model identity, locality and
  availability posture.
- **Target properties:** model catalog and capability; provider/runtime
  discovery and configuration; exact target/deployment identity, endpoint,
  locality, health and qualification; explicit Case binding and refusal through
  typed YAI application operations.
- **Dependencies:** S1 host/application coverage, provider governance and S2/S4
  presentation seams.
- **Current gaps:** no complete live provider setup or management contract.
- **Completion boundary:** generic providers can be inspected and configured
  without secrets, invented telemetry or provider-brand execution branches.
- **Non-goals:** making provider runtime state canonical Case truth.

## S8 — YVEX Control

**Posture:** horizon; unimplemented.

- **Purpose:** add first-party management of public YVEX runtime capabilities
  on a plane separate from the S7 generic model/provider/target experience.
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
