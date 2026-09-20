# YAI Studio — Case IDE / Case Workbench

Authority: canonical Studio product/frontend architecture, subordinate to the
[Constitution](constitution.md) and YAI-owned application semantics.
[ROADMAP](../ROADMAP.md#product-interfaces) alone owns macro state, maturity,
engineering selection, promotion, interlock and backend prerequisites. This
specification is neither an implementation queue nor evidence of promotion.
The subordinate [Studio roadmap](../studio/ROADMAP.md) records implementation
progression and dependencies without owning maturity.

## Purpose and Product Position

Studio is the official first-party Case IDE / Case Workbench inside YAI.
The Case is the primary visual and operational unit: a coherent workspace for
its sources, knowledge, experience, authority and work. A Case window/workspace
does not require an operating-system process per Case. The filesystem and
conversation are views within that workspace, not its organizing truth.

Studio is not an enterprise dashboard, ChatGPT clone, graphical command catalog,
replacement CLI, independent YVEX UI or new semantic owner. Native CLI, Studio
and future Mobile are clients over the same YAI application meaning, subject to
ordinary authentication, Participant scope, disclosure and admission.

The current [shell](../studio/README.md) has a bounded single-host live mode. Its
Start Center lists Cases visible to the authenticated local principal and its
Workbench consumes authorized application projections for Overview, Environment,
Knowledge, Memory, Authority, Work, Compute and committed Conversation Turns.
Fixtures remain an explicit development mode only. This vertical does not
establish remote, general multi-client or complete application-API qualification;
ROADMAP owns its exact maturity.

## YAI Product Topology

### CURRENT — bounded in-process desktop vertical

Today the Tauri process constructs `application/yai-application` in-process.
That bounded application facade authenticates the local operating-system
principal and supplies Case list/open/summary projections plus generation
invalidation. Studio does not start a resident YAI application service. Closing
the desktop process ends this adapter and its transient PTYs, while the durable
Case remains unchanged. The native CLI still reaches several owners through
CLI/store-coupled adapters. The current C `yaid` process is a separate narrow
daemon for status/info/shutdown and compatibility behavior; it is not the
complete application host.

### TARGET — one YAI product, many client surfaces

YAI is one product with a resident local application host and multiple client
surfaces:

```text
YAI PRODUCT

├── YAI Local Host
│   ├── Application Service
│   ├── RuntimeInstance / scheduling
│   ├── Update / Event Service
│   ├── Client Attachment Service
│   └── Host / system lifecycle and telemetry
│
└── Client Surfaces
    ├── YAI Studio
    ├── CLI
    ├── future structured external clients
    └── future remote / Mobile clients
```

The adopted target is normally one resident YAI Local Host for each explicit
`YAI_HOME` / local profile. It serves many Cases and many clients. Studio is the
primary first-party desktop client; it is not a frontend that owns or embeds a
second backend. Headless CLI/API clients attach to the same application meaning.
Launching Studio should discover an existing host or start it, wait for declared
readiness, and attach. Users must not need to start a daemon or configure a
socket manually for the ordinary desktop path.

```text
close Studio
    != stop YAI Local Host
    != close a Case
    != cancel background work
```

This is selected product architecture, not current executable behavior.

### Target process cardinality

| Entity | Target cardinality |
|---|---:|
| YAI Local Host | normally 1 per `YAI_HOME` |
| RuntimeInstance | normally 1 per Local Host |
| Cases | N |
| Studio windows | N |
| CLI clients | N |
| external clients | N |
| client attachments | N |
| Threads | N |
| Work / executions | N |
| provider runtimes | independently managed |

Multiple Studio windows attach to one host. Window A and Window C may observe
the same Case while Window B observes another; CLI and external clients may be
attached concurrently. Each window owns only tabs, layout, local navigation,
graph positions, scroll, selected material and transient drafts. YAI continues
to own Cases, Participants, history, knowledge, authority, Workflow, Resources,
provider posture and executions.

Different explicit environments such as `~/.yai`, `~/company/.yai` and
`~/research/.yai` may each have an isolated Local Host. No global cross-profile
authority owner is introduced. The consumer default is one environment;
advanced/operator use may select more than one explicitly.

### RuntimeInstance and `yaid`

CURRENT: `cmd/yai/src/runtime_instance.rs` is a bounded, tenant-fair,
single-host multi-Case scheduler with its own serve/status/shutdown lifecycle.
It advances admitted work but owns neither Case history nor Case semantics.
`cmd/yaid` remains a limited C daemon with narrow status/info/shutdown,
fixture-loop and compatibility journal/projection behavior.

TARGET: the resident Rust-owned Local Host progressively supervises normally
one RuntimeInstance alongside the application and update services. This
placement does not make RuntimeInstance the Case owner. The host target is
Rust-owned because the current application composition, runtime scheduler and
Case owners are Rust; this does not authorize a second C/Rust lifecycle owner.

OPEN: the implementation must characterize remaining `yaid` consumers and
compatibility obligations before any drain. Current `yaid` is not promoted or
renamed into the product host, and this specification does not remove it.

### Lifecycle, autostart and host telemetry targets

The implementation program may finalize names, but the intended headless
lifecycle surface is:

```text
yai host status
yai host start
yai host stop
yai host restart
yai host logs
yai host serve       # low-level foreground/service entrypoint
```

These commands manage the host process, never Case lifecycle. Desktop settings
will eventually offer `Start YAI automatically` using qualified user-level
platform mechanisms such as a Linux user service, macOS LaunchAgent or Windows
user startup/service mechanism. Studio must still be able to start the host when
it is absent.

The future singleton `Settings > YAI Host` surface displays only authoritative
facts: status, PID, uptime, version/build, `YAI_HOME`, protocol, transport,
authenticated principal, RuntimeInstance workers/work/queues, visible Cases,
attached clients, RSS/CPU/threads, heartbeat, last error and restart count where
the host actually exposes them. Host telemetry is operational process state, not
Case state, and unavailable values remain unavailable.

Provider processes have an independent lifecycle. Starting YAI does not load a
model, allocate a GPU, launch YVEX or start llama.cpp/vLLM. Cloud and local
providers remain selected capabilities with their own availability. Future YVEX
supervision belongs to its native management plane, not generic host readiness.

### OPEN implementation boundaries

The resident host still requires a versioned local transport, discovery,
authentication, readiness, reconnect/resync, update ordering, attachment
lifecycle, telemetry, packaging/autostart, CLI convergence and multi-client
qualification. None is implied by the present in-process bridge. No LAN listener,
remote serving, mutation-complete API or provider supervision is selected by
this document alone.

## Architectural Invariants

```text
Case truth                     -> YAI
Application API                -> YAI-owned typed boundary
Interface/transport projection -> existing Interfaces contracts where qualified
Studio                         -> representation and interaction
CLI                            -> terminal representation and interaction
YVEX                           -> inference/runtime truth
```

Committed Transition history is canonical; CaseState is its rebuildable current
materialization. Original immutable content retains its existing owner. Derived
views remain derived. The [state contract](reference/state-transitions.md),
[semantics](reference/semantics.md), [context contract](reference/context.md),
[Recall](recall.md) and [execution boundaries](reference/boundaries.md) define
meaning; this document does not redefine their schemas or algorithms.

Studio must never spawn `./yai` as its canonical API, parse human CLI output,
read LMDB/private Transition or graph layouts, or reconstruct Policy,
EffectivePolicy, Recall, Workflow, historical/as-of truth, source validity,
D/H/S epistemic posture, authority, provider state or Case semantics. A displayed
success, graph edge, model answer or selected tab grants nothing. Actions and
external effects still require the current YAI typed admission chain.

React owns rendering/interaction. The native shell owns only needed desktop
integration, not another business layer. Interfaces may own its compiled
interface representation and qualified projections, never independent operation
meaning competing with YAI. No duplicate registry, DTO model, persistence or
canonical `Session` is introduced for a frontend.

## Current Application Boundary and Backend Gaps

[Executable architecture](architecture.md#current-applicationclient-seams-and-limit)
is authoritative for current seams. `application/yai-application` now composes a
small set of typed, authorized read projections from existing engine owners:
runtime readiness, Case list/open/summary and generation invalidation. It owns no
persistence or semantic state. `case.open` requires the current authenticated
principal's admitted Participant link and returns only an ephemeral attachment.
The native CLI still has store-coupled adapters and `ConversationController`
remains inside its crate, so X03 is not a complete stable shared/public API.

The existing `yaid` Unix socket handles status/info/shutdown and compatibility
fixture/journal/projection operations. It does not serve those controller
operations or a Studio event stream. Studio must not repurpose it by assuming
that the existence of a socket proves application coverage.

| Gap exposed by Studio | Responsible boundary and required qualification |
|---|---|
| Broader typed queries/actions/results/refusals shared with the native CLI | YAI application layer; preserve owner checks and one operation meaning |
| Standalone local listener and general transport qualification | YAI host plus qualified Interfaces projection; discovery, authentication, negotiation, disposal and unavailable posture beyond the bounded in-process bridge |
| Incremental progress and general scoped subscriptions | YAI lifecycle facts plus transport; current generation invalidation needs broader ordering, missed-update, backpressure, cancellation and redaction qualification |
| Simultaneous attachments and reattachment | YAI; current Principal/Participant/Thread resolution, concurrent mutation/refusal, stale generations, idempotency and recovery |
| Resource/file changes outside YAI actions | YAI observation/source boundaries; provenance, revision, confinement and explicit admission, including watcher gaps |
| Native provider management evidence | Public provider/YVEX management contracts; truthful capability/version/permission and failure exposure |
| Remote/Mobile consumption | YAI/Interfaces authentication, disclosure and transport contracts beyond local OS trust |

These gaps do not automatically select work or establish an Interlock. The
current Tauri bridge is an in-process local adapter: it authenticates through
YAI for every request, binds no network socket and exposes one versioned call
surface plus generation invalidations. React cannot close the remaining gaps.

## Interfaces and Historical Reconciliation

The inspected local Interfaces origin is `git@github.com:yailabs/interfaces.git`,
at [bae6cdf7](https://github.com/yailabs/interfaces/tree/bae6cdf7cf17f3e6a58c0323852c7c0efeb26147).
Its [transport strategy](https://github.com/yailabs/interfaces/blob/bae6cdf7cf17f3e6a58c0323852c7c0efeb26147/docs/transports/studio-transport-strategy.md),
[Rust/Tauri boundary](https://github.com/yailabs/interfaces/blob/bae6cdf7cf17f3e6a58c0323852c7c0efeb26147/docs/sdk/rust-tauri-bridge-contract.md),
TypeScript client sources, schemas and conformance fixtures exist. The inspected
SDK exports also include Session, Agent and Orchestrator surfaces. The old
readiness matrix refers to RT.04 and a previous runtime. None is automatically
compatible with current YAI's Case/Transition ownership. The old planning SHA
339fca4371e1c7aced2d91c6595ecdc3cac0ea87 cited in ROADMAP is absent from this
checkout; no remote history identity or migration is inferred from that absence.

Preserve the useful direction: local IPC/RPC for native requests, an associated
local event stream for updates, loopback HTTP only as a qualified development
fallback, and future secured remote/LAN projections. Preserve explicit
unavailable/version/refusal states and event gaps. These are candidate transport
contracts requiring a current YAI producer and conformance, not dependencies or
new routes selected by this skeleton. No Interfaces process is needed to build
or run core, CLI or this bootstrap. A generated CLI remains a possible reference
consumer; the handwritten CLI remains first-party product.

Archaeology also inspected `yai-dev` at
[5c1c7b9d0](https://github.com/yailabs/yai-dev/commit/5c1c7b9d099eea9f2947146cd821d6501c4a6ddf), adjacent
client-attachment schemas, session continuity code and the IPC probe tests,
then the pre-drain source at
[dda93ee3](https://github.com/yailabs/yai-dev/tree/dda93ee3ac6c9a8471822db8e516a2d46ad12c54).
The historical `src/runtime/boundary/transport/local_ipc_rpc_listener.c`
implemented an owner-checked Unix endpoint, restricted socket creation, discovery
cleanup and one-client probe acceptance; Windows explicitly deferred. Adjacent
`api/ipc_probe_dispatch.c` distinguished supported reads, blocked sends,
deferred operations and unsupported operations. The executable
`tools/checks/runtime/check-local-ipc-rpc-runtime-e2e.py` built and ran a probe
checking handshake/response identities, refusals and socket/discovery cleanup;
it explicitly excluded production readiness. This is stronger evidence than a
transport document, but not full Case client or multi-client qualification.

The substrate drain commit
[2a4018147](https://github.com/yailabs/yai-dev/commit/2a4018147219044dfe1fad2268759b1f2a585945)
removed/reorganized those runtime ownership trees; surviving session drain code
still contains old file-backed continuity and session caches, and its knowledge
session drain test is primarily compilation/layout proof. It does not establish
a new canonical Session requirement in current YAI. The adjacent `studio`
checkout is a historical documentation bootstrap using a retired core name; no
application implementation was found there or copied into this repository.

Recovered properties are separation of client attachment from Case identity,
truthful unavailable/refusal posture, and transport cleanup as a future
qualification criterion. Their target owners are the YAI application host and
its transport adapter. Reuse is semantic only: no historical runtime plane,
session store, registry, C/Rust duplicate owner or SDK ontology is imported.
Current controller restart tests already derive Threads from committed Turns;
that stronger executable continuity contract is preserved unchanged.

For the resident-host target, the same archaeology recovers four more bounded
properties: same-machine-only endpoints, a private endpoint root and `0600`
socket posture, explicit discovery source/status, and handshake/version mismatch
as a first-class refusal. The historical listener removed only a stale socket it
owned and cleaned its socket/discovery paths on failure and stop. These belong
to future transport qualification. Its dev-only single-client probe dispatcher,
old operation vocabulary and C runtime ownership are rejected because they do
not cover current Case/application semantics or multi-client lifecycle.

## Multi-client Continuity

Target: the same Case can be observed or used concurrently by Studio,
`yai open CASE`, several terminals, future Mobile and remote clients. Each
attachment resolves its own authenticated access and appropriate Participant
and Thread. Shared Case identity never implies shared credentials, scope or
unrestricted disclosure. Closing a window, connection or terminal does not
close the Case or erase committed work.

| Concept | Continuity and lifecycle |
|---|---|
| Case | Durable canonical continuity owned by YAI |
| Participant | Case identity/role and admitted views, distinct from a client connection |
| Thread | Currently a field of committed Turns; inventory derives from history, not a new store |
| Execution/work | YAI-owned attempts, budgets, effects, review and recovery; frontend detachment does not manufacture cancellation/completion |
| Frontend attachment | Ephemeral interaction/observation context, no historical authority |
| Transient draft | Unsubmitted local text/selection; no Turn until explicit application submission |
| PTY/process | Native process/FD lifecycle; distinct from Case and execution authority |
| Local UI state | Window, navigation and rendering preferences; never Case truth |

Current application `case.open` resolves the locally authenticated principal's
linked Participant and last committed thread, where one exists. The attachment
is not persisted and closing Studio changes no Case lifecycle. Current runtime
admission provides single-host mutual exclusion for active Case advancement.
Generation invalidation proves one bounded observer path; it does not qualify
simultaneous mutations, concurrent SEND, background execution after detachment
or distributed operation. Those behaviors need explicit conflict, revocation,
reconnect and recovery tests at the application boundary.

## Open in Terminal

Preferred target: start a new terminal frontend attached to the same durable
Case, with authorized Participant/Thread selection, and reconstruct its view
through YAI. This need not transfer an existing REPL or PTY process. Existing
`yai open CASE` and in-Case thread selection are footholds, not a qualified
desktop handoff command or promise of exact launch flags for every selection.

The Case survives the surface. Unsaved drafts, scrollback, terminal environment
and cursor are not implicitly shared; any future explicit transfer of local
draft text remains unsubmitted input. Desktop launch mechanics must validate
arguments and avoid shell interpolation. Relaunch cannot duplicate a submitted
Turn, repeat an indeterminate effect or inherit an unauthorized Participant.

## Integrated Terminal

The target is a real terminal, not a text area mimicking shell output:

```text
Studio renderer
    |
terminal frontend
    |
native PTY host
    |
shell / yai / yvex / tools
```

| Surface | Meaning |
|---|---|
| Human interactive shell | Ordinary user-controlled terminal process and environment; its actions are not automatically governed YAI executions |
| YAI REPL | Existing CLI frontend over YAI, with REPLAI terminal mechanics |
| YVEX CLI | Operator use of the external runtime's public CLI; no Case semantic authority or implied YAI API |
| Structured YAI execution output | Facts from governed YAI execution/result/receipt contracts, linked to exact work and admission |
| Future Computer Use | Governed target observation/actions, separately qualified from terminal interaction |

Structured process execution takes an admitted command/profile and returns
qualified observations/receipts. It is not simulated typing into a shell. PTY
bytes cannot replace an execution receipt, and terminal output must not be parsed
into Case authority. REPLAI remains the current CLI mechanics owner. Studio's
desktop shell now uses `@xterm/xterm` as its terminal renderer and a narrow
`portable-pty` Tauri host. The bridge accepts only terminal lifecycle, input and
resize operations; it does not expose a generic process command. It starts the
user's ordinary local shell in a deliberate local home directory, supports
multiple transient terminals and cleans them up with the desktop window.
Browser mode reports that the desktop host is required and never emulates a PTY.

PTY bytes remain presentation mechanics. Studio does not parse them, inject Case
labels as commands or use them as application results. A user may invoke `yai`
inside the shell, but all Case facts rendered elsewhere still come only through
LiveClient's typed YAI boundary. The current shell is generic and local; it is
not Case-attached Open in Terminal and has no persistent terminal sessions.

## Dynamic Case Projection

Target: Studio reacts to actual system facts, scoped to the viewing Participant.
The interactive view is not a second semantic compiler; YAI's existing technical
Projection/ContextFrame types retain their own definitions. The application
boundary must expose adequate views and update facts for the UI.

| Change class | Required source and display posture |
|---|---|
| YAI-originated mutations | Accepted result and current generation; refresh qualified owner views |
| Execution progress | Actual lifecycle facts; do not synthesize token streaming or a completion percentage |
| Reviews | Current typed review/Decision posture with eligibility rechecked by YAI |
| Effects | Distinguish prepared, observed, finalized and indeterminate work |
| Files/artifacts changed | Exact admitted revision/backing or produced artifact identity |
| External filesystem changes | Qualified observations, provenance and freshness; an OS notification alone is not semantic admission |
| Participant activity | Disclosed committed activity or explicitly qualified ephemeral presence; no invented presence badges |
| Provider/runtime state | Public authoritative evidence when available, otherwise unknown/stale/unavailable |

Subscription loss, missing events and stale views must be visible. The current
local bridge emits Case identity, generation, sequence, cursor and affected view
families after a real authorized generation change. A typed authorized heartbeat
compares the attached Case generation and closes event-delivery gaps; LiveClient
invalidates, refetches `case.summary` and performs a full resync after stale
generation. It does not expose private Transitions or claim a general event
stream. External file observation, progress streaming and full dynamic
projection remain unimplemented.

## Computer Use

Computer Use is a future YAI-governed capability observed through Studio. The
renderer does not own a computer agent or gain authority from user focus.
The surface will show computer targets, observations, actions, screenshots/frame
state where qualified, review/authority and execution receipts. Targets and
input actions need their own YAI admission/resource boundary. No detailed
implementation, automation driver or screenshot transport is selected here.

## YVEX Integration

### Inference plane

YVEX continues through the same qualified generic OpenAI-compatible provider
boundary as other compatible runtimes. No `if provider == YVEX` branch belongs
in generic cognitive execution. YAI owns context meaning, target qualification,
selection and admission; YVEX owns its exposed model/runtime realization.
Provider output remains candidate material. Computational continuity is neither
Case continuity nor evidence of a successful external effect.

### Management plane

Studio may later expose additional public YVEX product capabilities: installed
and admitted models, engines, load/unload, deployment, compilation,
memory/residency, sessions, runtime evidence, device state and authoritative
telemetry. Runtime sessions here belong to YVEX, not a new YAI Case Session.

Such controls require real versioned public management capabilities, permissions,
results and truthful failure states. They must not bypass YAI inference
qualification, read YVEX internals or make YVEX a Case owner. No management API
is invented or implemented. During this wave YVEX is not inspected/administered;
provider qualification remains black-box consumption as specified by AGENTS.

## Provider Experience

Target a common experience for generic OpenAI-compatible endpoints, vLLM,
llama.cpp, OpenAI, Anthropic and other providers. This is a UX horizon, not a
claim that all protocols or adapters exist today. Show actual capabilities,
model identity, qualification, unavailable/refusal posture and provenance.
Discovery, operator consent and suitability remain YAI application semantics.
YVEX is a strategic first-party integration with additional qualified management
capabilities; inference semantics remain generic. Studio now renders the generic
provider target/posture already exposed by YAI, with sanitized endpoint identity.
Live provider configuration and YVEX management remain unimplemented.

## Progressive Disclosure

One product serves ordinary users, developers and operators. Start with the
Case, the current work, readable results and explicit review/action choices.
Reveal identifiers, provenance, exact refusals, execution details, raw qualified
evidence and runtime diagnostics on demand. Advanced visibility does not add
permission. Simpler presentation must not hide indeterminate delivery, stale
data, required consent or missing evidence. There are no separate semantic
products or parallel simple/advanced state models.

## Studio Workbench Architecture

### Selected structure

React, TypeScript, Vite and Tauri remain the renderer and desktop-host
technologies. The adopted product architecture above them is:

```text
YAI Studio
├── Platform
├── Workbench Kernel
├── YAI Built-in Contributions
└── Desktop Host
```

The Platform and Kernel own frontend mechanics and local UI state. Built-in
contributions present typed YAI application facts. The Desktop Host owns the
window and narrow native integrations such as PTY lifecycle. None owns Case
meaning.

### Platform

CURRENT: the bounded Platform implementation provides scoped commands, context
keys, keybindings, menus, versioned local Workbench preferences, navigation,
semantic theme selection, lifecycle/disposables and explicit host capabilities.
Registrations belong to a Workbench instance and are disposed on teardown; no
global extension registry survives tests or hot reload. Command Palette and
Quick Open consume those existing registries rather than duplicate command or
navigation ownership. A broader context-expression language remains OPEN.
A command such as `studio.go.back` or `studio.terminal.new` is a frontend action
identity, not a YAI application operation. A command may invoke an admitted YAI
operation through the application boundary, but its UI identity grants no
authority.

Current typed predicates gate commands, menus and keybindings with keys including
the data source, native host, Case attachment, active view/panel, selection,
layout visibility and terminal availability/focus. Broader keys such as exact
typed review selection grow only with real consumers. Context keys remain
ephemeral presentation facts; they do not become Case policy.

### Workbench Kernel

CURRENT: one Kernel owns stable regions and layout services:

```text
Workbench Kernel
├── Activity Bar and registered View Containers
├── Sidebar and Views
├── Surface Groups, Surface Inputs and Tabs
├── Bottom Panel
├── Auxiliary / Context Bar
├── Inspector Host
├── Status / desktop chrome
└── layout services
```

The Kernel does not know the semantics of Memory, Knowledge, Authority, Work or
providers. It renders registered internal contributions. One visible Surface
Group currently owns inputs, active selection, preview reuse, pinning, close and
previous/next behavior; the model does not preclude later groups, while split
interaction remains OPEN. A Surface Input identifies presentation/navigation
state, while a registered trusted renderer owns its representation. The Kernel
therefore hosts files, documents, artifacts, graphs, timelines, Settings,
provider detail and future Computer surfaces without an ever-growing feature
switch. Panel and auxiliary views use the same rule.

### Universal Work Surface

The central Workbench region is a **Work Surface**, not a code editor. An editor
is one possible renderer. No Workbench primitive assumes that its primary input
is a text file, source file or document.

```text
Case object / Artifact / Projection / Tool
                  │
                  ▼
             Surface Input
                  │
                  ▼
            Surface Registry
                  │
                  ▼
          trusted Surface Renderer
                  │
                  ▼
             Surface Group
                  │
                  ▼
              Work Surface
```

CURRENT: presentation roles distinguish Content, Projection, System and
Interactive Case Surfaces without introducing Case ontology. Renderer-declared
capabilities such as previewable, pinnable, singleton, editable, dirty-aware,
searchable, zoomable, navigable and selectable control Workbench behavior only;
they never grant YAI authority. Tabs express preview/pinned, genuine dirty,
read-only and unavailable posture with restrained affordances rather than
pretending every Surface is an editable file.

Perspective, Markdown, text, structured text, browser-safe image/vector, PDF,
structured table, audio/video, timeline, graph and Settings renderers use the
same Surface Group and tab mechanics. Qualified MIME/media types select material
renderers; random React components do not infer representation from filename
extensions or raw bytes. PDF supports lazy multi-page canvas rendering, zoom,
fit-width, page navigation and text search. Tables support filtering, sorting,
row selection and horizontal/column resizing. Unknown media opens a first-class
metadata/unavailable Surface and is never coerced into binary text. External
open remains unavailable until YAI exposes a qualified local path or permitted
temporary representation.

TARGET: trusted reusable data surfaces include Calendar, Form, Board, Chart,
Gallery and Detail/Object in addition to the current bounded renderer families.
Databases, provider tools and Computer surfaces can add trusted renderer types
without a Kernel change. A Surface is a representation, not the underlying object: a PDF
artifact can open in a PDF Surface, appointment facts in a Calendar Surface and
qualified relations in a Graph Surface while YAI retains truth and authority.

Editable surface gestures follow one admission path:

```text
Surface gesture
      ↓
typed application action
      ↓
YAI authority / validation
      ↓
Case mutation and/or governed effect
      ↓
result and refreshed projection
```

React state alone never commits a business mutation.

CURRENT editable state is deliberately narrow: Studio-local Settings may change
versioned local preferences. Case materials remain read-only because no general
save/admission operation is qualified. A future editable Case Surface holds
local dirty state until an explicit typed application action succeeds, then
refreshes from the resulting projection; it never silently writes a source from
React.

### Search and Settings infrastructure

Search is one Workbench interaction grammar with distinct owners. Command
Palette searches enabled commands and executes through `CommandService`; Quick
Open searches open/recent Surface inputs plus presentation references already
exposed to Studio; current-Surface search delegates to the active renderer; and
Settings search delegates to the Settings registry. A Case-search provider seam
exists, but the current live YAI application boundary exposes no qualified
semantic/content search query, so live mode reports that exact unavailability.
Fixture mode supplies deterministic Case-search results only for Workbench
qualification. Studio never scans the filesystem or LMDB to fill that gap.

Settings is one singleton System Surface. Built-in contributions register
entries under General, Appearance, Workbench, Terminal, YAI Host, Providers,
YVEX, Security and Advanced. Scope is explicit: local Studio, Case-backed or
Host. Only genuine local preferences are editable today, persisted as a small
versioned browser/WebView local-storage representation. Case or Host settings
require their respective typed operations and otherwise remain explanatory or
unavailable. No settings database or Case truth is created.

### Declarative Case Views — target

A future Case may define a trusted view through supported Surface primitives.
For example, call transcripts could become admitted source/events, then
classified appointment candidates, authority-confirmed appointment facts and a
Calendar Surface that operators use directly. A conceptual `Appointments` view
might name a calendar surface, a qualified appointment projection and mappings
for start, end, label and operator. This selects a product property, not a
schema, syntax, persistence owner or currently implemented Calendar system.

There are two distinct extension classes. **Built-in Workbench Contributions**
are statically authored YaiLabs code for deep capabilities such as Memory,
Knowledge, Authority, Terminal, YVEX and Computer Use; they may register new
trusted renderers. **Declarative Case Views** compose renderers already trusted
by Studio and do not execute arbitrary code. Natural-language composition
normally proposes a declarative view for YAI validation/admission rather than
executing model-generated React or JavaScript inside Studio. A future shared,
persistent Case View needs a qualified YAI-owned persistence and admission
contract; no frontend View store is selected here.

Tab selection, calendar zoom, graph positions, scroll, column widths and
temporary filters remain frontend-local. Desktop composes Surfaces inside its
Activity Bar, Sidebar, Work Surface, Auxiliary Bar and Panel. A future mobile
client may present the same Calendar, Participant or Workflow surface semantics
full-screen in a mobile composition; the desktop Workbench layout itself is not
portable product truth. Surface selection reports meaningful typed selection to
the shared Inspector/navigation seam rather than creating private navigation
stacks.

### Internal built-in contributions

Overview, Participants, Environment, Knowledge, Memory, Authority, Work,
Compute, Conversation, Terminal, Settings and future YVEX management are
YaiLabs-authored built-in contributions. A contribution may register commands,
context keys, menu placements, view containers/views, Surface Inputs and
renderers, inspectors, settings and justified status items. It does not modify Workbench regions
directly.

Studio is contribution-driven internally, not an externally extensible plugin
platform. No third-party SDK, public plugin API, extension host, marketplace or
compatibility guarantee is selected. The internal model still matters because
it isolates features from layout, centralizes commands/menus/keyboard behavior,
keeps generated code and coding-agent work on stable seams, and prevents ad hoc
controls from fragmenting the product.

Menu locations are themselves Workbench surfaces: application, surface context,
tree context, graph node/edge context, Inspector context and terminal context.
The application menu consumes the scoped command and menu services now. The
other locations are accepted internal locations but remain unpopulated until a
real contribution needs them.

### Design foundation ownership

The Workbench owns fonts, semantic typography, spacing, row heights, icon
optical size, focus states, tooltips, tabs, borders, radius and semantic colors.
Contributions use shared tokens and primitives rather than defining local visual
systems. This is a product constraint, not a theme preference.

Every Studio implementation milestone performs a proportionate Product Quality
Pass on the visible regions it materially changes: ownership, hierarchy,
typography, spacing, icon alignment, semantic color, interactive states,
surface separation, keyboard/accessibility, empty/error/unavailable posture,
resizing, duplicated patterns and visual regression. Compilation alone does not
close a visible Studio wave. The pass does not authorize unrelated redesign or
override executable truth and root ROADMAP maturity.

### Inspector, navigation and traversability

Inspector is a navigation surface, not a generic property dump. Typed
inspectors for Participants, Sources, Resources, Artifacts, Claims, Events,
Episodes, Reviews, Decisions, Workflows, Executions and Providers progressively
expose readable identity, summary, relations, provenance, authority, technical
detail and admitted actions. Technical IDs remain available but do not dominate
the first-level experience.

The product invariant is: **no dead objects; no dead edges.** Every meaningful
object shown in Studio must be selectable, inspectable and resolvable, and open
where that action has meaning. Every meaningful relation must explain its kind,
expose provenance/backing and navigate to both endpoints. Inspector is the
bridge; it need not be the final destination. Local Back/Forward preserves these
transitions without modifying Case history.

Settings is one singleton Surface with internal navigation and history. Its
current sections are General, Appearance, Workbench, Terminal, YAI Host,
Providers, YVEX, Security and Advanced. Opening or searching a section does not
create another Settings tab.

### Graph and temporal infrastructure

Knowledge, Experience/Memory, Authority and Workflow graphs share canvas
mechanics such as pan/zoom, selection, filters, navigation and Inspector
integration. They do not necessarily share vocabulary, clustering, layout or
edge semantics. Each graph is a derived application projection, never canonical
truth or an independent frontend store.

Memory's target temporal view is a horizontal canvas able to present lanes,
branches, overlapping activity, generation markers, event selection and links
to graph/Inspector. Chronology never implies causality. Timeline, Graph,
Inspector and Work Surface are coordinated ways to traverse the same Case:

```text
Timeline ↔ Graph ↔ Inspector ↔ Work Surface
```

Transitions between them require a typed relation; visual proximity alone does
not create one.

## Case Workbench Surfaces

Every row describes the product target; only the bounded fields named in the
current application projection are live. The Start Center enters a Case and the
Workbench is organized by Overview, Environment, Knowledge, Memory, Authority,
Work and Compute. These are presentation perspectives over one Case, not frontend
domain owners. Environment presents admitted Sources, their acquired revision
file inventory and operational Resources; it never scans the filesystem from
React. Knowledge shows
qualified derivation or explicit absence; Memory presents committed history and
derived graph relations; Authority presents current policy/review/grant facts;
Work presents current workflow definition/resolution; Compute presents generic
provider posture. Unsupported families use explicit empty/unavailable states.

Overview is the Case lens, not a generic home dashboard. It keeps durable Case
identity and attachment facts central, then presents Environment, Knowledge,
Memory, Authority, Work and Compute as navigable dimensions of that same Case.
The sidebar repeats this composition as a compact outline and Inspector can show
the complete bounded Case posture. This is how the product communicates entry
into a Case without turning labels or KPI tiles into substitute semantics.

In explicit fixture mode, Case composition is a single local work-surface tab: Identity, Sources,
Participants, Authority, Resources and Compute remain visible together rather
than implying a server-owned wizard lifecycle. After entering a Case, Studio
does not use its brand chrome as a route back to the Start Center. The desktop
application menu owns opening or composing another Case, while the current Case
continues to be the Workbench context. Back navigation may leave a local surface
for that Case, but must not reinterpret the product root as the previous Case
attachment.

The right-hand Context Panel can represent Conversation, Inspector and Activity
without making conversation the product root. Inspector identifies the selected
fact type, properties and qualified Case relations; materials may be explicitly
opened from it. Primary perspectives retain distinct tabs. Files, documents and
sources use one reusable preview tab until explicitly pinned, so quick material
navigation does not replace an open Environment or Knowledge perspective. The
central work surface remains a generic local tab host. The fixture graph is a
lightweight rendered projection, not an inference engine or causal model. A
source may also be a Resource, but the views preserve those distinct semantic
roles.

| Surface | Representation and interaction | Existing owner or prerequisite |
|---|---|---|
| Case Explorer | Case workspace, Participants, Resources, Workflow, artifacts, status | Existing YAI Case owners and scoped queries |
| Source Explorer | Source Map: local/mounted sources, repositories, DB, APIs, MCP, roles, revisions/backing, coverage | Source/resource acquisition contracts; general Source Map still a target |
| Resources | Operational capabilities, attachments and current scope | YAI Resources/Binding/admission |
| Knowledge | Documents, entities, topics, claims, contradictions, wiki/navigation | Bounded source-grounded derivation; broader interpretation/coverage future |
| Experience / Timeline | Events, Episodes, Decisions, effects and historical state | Qualified history/as-of and experience readers |
| Graph | Distinct D/H/S, resources, relations and navigation | Derived qualified access, never inferred causal authority |
| Recall | Requests/traces, documentary/experience segments, sources, reasons and missingness | Qualified D/H/S Recall |
| Policy / Authority | Policy, EffectivePolicy, DecisionBasis, scope and eligibility | Existing governance/admission owners |
| Workflow | Tasks, bindings, progression and handoff | YAI Workflow lifecycle |
| Executions | Work, invocation, progress, attempts, stops and results | YAI execution owners; public progress gap |
| Reviews | Requests, evidence, current decision and available actions | Typed YAI review/admission |
| Artifacts | Exact content identity, revisions, availability and explicit publication | YAI content/source/effect owners |
| Evidence | Provenance, source closure, retained backing and missingness | Owner-qualified evidence roles; no universal Evidence database |
| Cognitive State | S/W, future E identity/compatibility/capability | YAI semantic compilation; YVEX computational E |
| Providers | Common discovery/connection/qualification experience | Generic YAI provider boundary |
| YVEX | Additional native product management capabilities | Future public YVEX management contract |
| Work Surface | Trusted representations of Case objects, artifacts, projections and tools | Explicit save/admission through existing or future YAI contracts; no silent source rewriting |
| Computer surface | Targets, frames, observations/actions and receipts | Future governed Computer Use capability |
| Bottom Panel | Logs, evidence, tests, execution/output tools | Presentation of facts from responsible owners |
| Terminal | Real shell/REPL/tool processes | Qualified desktop-local xterm + portable-pty mechanics; Case-attached handoff remains future work |

## Local UI State vs Case State

| State | May remain frontend-local | Must originate from YAI or the named authoritative source |
|---|---|---|
| Active tab, viewed file, layout, panels, scroll, graph positions | Yes | Selection does not change authority or scope |
| Selected terminal, window size and keyboard preferences | Yes | Native host owns real process/FD facts |
| Unsubmitted text, local draft, surface selection | Yes | Explicit SEND/save uses YAI admission; no implicit Turn |
| Existing persisted Advanced application draft | Only a local edit buffer/view | Existing Case-namespaced draft owner; draft is still not committed history |
| Selected Case/Participant/Thread | Local attachment selection/reference | YAI resolves identity, access and committed Thread facts |
| Case lifecycle, historical/current truth | No | Committed history and qualified CaseState/readers |
| Policy, Recall, Workflow, reviews and effects | No | Existing YAI owners; cached views do not authorize actions |
| Source/file/artifact revisions and provenance | Local view cache only | Qualified source/content/resource observation/admission |
| Execution and participant activity | View cache only | Actual disclosed YAI lifecycle/activity facts |
| Provider connection/qualification | View cache only | YAI provider contracts |
| Physical model load, residency, runtime telemetry | View cache only | Authoritative public runtime evidence, not inferred from a successful call |
| Fixture data | Explicit offline development input | Never presented as live runtime truth or sent as an admitted action |

## Desktop Technology Direction

Selected initial direction: **React + TypeScript + Vite + Tauri 2**, under
`studio/`. No concrete incompatibility was found with the current independent
engine workspace and CLI Cargo package. No existing root Node workspace or
frontend CI was found to inherit; npm is local to Studio, with its own lockfile. The native shell
has its own Cargo workspace/lockfile and depends on the bounded YAI application
crate, not a second semantic layer. Core/CLI Make
targets remain independent of Node, Tauri and Studio. The source-placement guard
admits only the added desktop Rust source/build script and excludes generated
Studio dependency/build trees from source classification.

React/TypeScript owns rendering and interaction; Tauri supplies the container
and the in-process request/update adapter. Rust shell code remains thin; typed
application composition lives below it and existing YAI owners remain canonical.
The desktop window uses one Studio-owned title row with native window controls;
this removes redundant OS/application/Case bars without moving product behavior
into the Tauri shell.
No Next.js, Electron, editor engine, graph library or native plugin is added.
The deliberately selected terminal renderer is the narrow xterm surface described
above. CSS tokens, reusable controls and one inline icon system form
the permanent UI foundation without an external UI kit. The dark desktop grammar
uses tonal surface layers, spacing and typography for most separation, reserving
visible dividers for structural splits. Controls and grouped surfaces use modest
radius while the application frame and terminal-like regions stay comparatively
hard. Tauri requires a PNG at compile time; the shell uses one
transparent RGBA pixel, not a product icon design. Future modules grow when
there is code to own; the conceptual
`case`, `components`, `surfaces`, `state`, `terminal`, `yvex` placement is not a
requirement to create empty noun directories.

The desktop workbench adopts the structural seams needed by an extensible IDE:
UI commands are registered independently from menu placement and keyboard
bindings; Activity Bar items select view containers; the primary sidebar,
Surface tabs, auxiliary context panel, bottom panel and desktop chrome have
stable responsibilities; and visual roles use shared semantic tokens. This is
frontend-local contribution plumbing, not a YAI operation registry and not a
plugin host. Future YaiLabs-authored built-in contributions use these seams
without receiving Case authority or depending on component-private styling.

The bootstrap follows the official [Vite guide](https://vite.dev/guide/) and
[Tauri Vite integration](https://v2.tauri.app/start/frontend/vite/). Desktop
prerequisites are isolated from backend prerequisites. Installer packaging,
signing and platform distribution are not qualified by an executable build.

## Live vs Fixture Mode

```text
CaseDataSource (frontend presentation seam, not a YAI API)
├── LiveDataSource    -> LiveClient application operations/update invalidation
└── FixtureDataSource -> FixtureClient authored development input
             │
             ▼
      one CasePresentation
             │
             ▼
 StudioApplication + WorkbenchKernel

HostServices (independent axis)
├── desktop-native -> PTY/window controls available
└── web            -> explicit capability absence
```

`LiveClient` maps the bounded `yai.studio.application.v1` result envelope into
small presentation types. Normal mode currently requires the Tauri-local
adapter; an absent adapter, authentication failure and unsupported projections remain explicit result
states. It never knows LMDB layout, CLI syntax or private Rust domain structs.
The application projection is intentionally smaller than `CaseState` and does
not predeclare every future operation. Its in-process transport is not a stable
public SDK or remote service qualification.

FixtureClient reads authored examples under `tests/fixtures/studio/`. A bounded
adapter maps those authored values into the same frontend Case presentation
contract consumed by built-in contributions. It performs
no I/O, timer-driven execution or semantic reconstruction and can be selected
only when `VITE_STUDIO_MODE=fixture` is set before the build/dev process.

Fixtures support visual development and deterministic screenshot evidence now;
site/README/docs reuse and visual regression remain consumers of the same data.
They must be plausible, recorded or sanitized with explicit origin, contract
version and missingness. No fixture is presented as live telemetry or used as a
silent fallback. In the explicit fixture build,
`fixture=ordinary|developer|execution` selects exact authored data. Fixture data
posture remains visible. The bare fixture URL opens the authored Start Center
and the composition action opens the complete offline composer. Static
running/review/failure labels are not live telemetry. On Tauri desktop the
normal Terminal contribution owns a real PTY even when Case data is
fixture-authored; in a browser the same contribution reports that the desktop
host is required. No review/send controls simulate authority.

Layout sizes, collapse state, perspective, Context Panel mode, graph/timeline
selection and work-surface tabs are React state. URL query state selects only
authored Start/Case inputs for deterministic rendering.
Closing/reopening a panel retains its size and current tab/draft within the
selected fixture.
Switching scenarios resets material tabs and the draft, while retaining layout;
reloading restores deterministic scenario defaults. This is frontend-local
interaction, not Case attachment or multi-client continuity. The same rendering
components consume each scenario without scenario-specific layout branches.

## Qualification

Qualify each future surface at its named contract/version and YAI SHA. Require
the same application meaning and refusal behavior as the native CLI; no private
store access or output parsing. Demonstrate scoped visibility, identity,
provenance, stale/unavailable states, authorized mutations and indeterminate
recovery where applicable. Use deterministic fixtures for rendering, plus real
YAI application consumption for product evidence; neither substitutes for the
other. Multi-client tests must demonstrate attach/detach/restart, conflicting
writes, revocation and event resync without losing or duplicating Case work.

Terminal qualification additionally needs real PTY input/output, resize,
interrupt/EOF, cleanup and child-process lifecycle. Open in Terminal must retain
the Case/authorized Participant/Thread without depending on a surviving window.
Provider management and Computer Use each require independently qualified
public capability and authority contracts. Usability/accessibility and visual
regression are separate frontend proof; they cannot establish application semantics.

[Validation procedures](test-cases.md#studio-bootstrap-isolation) distinguish
backend publication, frontend build and native shell build. The cumulative
[Golden runbook](zero-to-current.md) remains the product acceptance procedure;
human acceptance requires the operator's result at the relevant SHA. A compiling
shell does not establish live product qualification. Fixture interaction and
visual evidence qualify only their explicitly offline scope. All promotion stays
in ROADMAP.
