# YAI Studio — Case IDE / Case Workbench

Authority: canonical Studio product/frontend architecture, subordinate to the
[Constitution](constitution.md) and YAI-owned application semantics.
[ROADMAP](../ROADMAP.md#product-interfaces) alone owns macro state, maturity,
engineering selection, promotion, interlock and backend prerequisites. This
specification is neither an implementation queue nor evidence of promotion.

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

The current [bootstrap](../studio/README.md) contains only a minimal React page
and Tauri window. All workbench surfaces below are targets. Documentation and
compilation do not qualify the product or change its roadmap maturity.

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

## Existing Application Boundary and Backend Gaps

[Executable architecture](architecture.md#current-applicationclient-seams-and-limit)
is authoritative for current seams. `ConversationController` has typed actions
and results inside the CLI crate, with `pub(super)` visibility. Its result events
are buffered facts, not a subscription service. Some inspections return generic
JSON; other CLI adapters still take argument vectors, open stores and capture
printed output. Public engine types still require authorized store/content setup.
These are useful existing mechanisms, not an exported frontend-independent API.

The existing `yaid` Unix socket handles status/info/shutdown and compatibility
fixture/journal/projection operations. It does not serve those controller
operations or a Studio event stream. Studio must not repurpose it by assuming
that the existence of a socket proves application coverage.

| Gap exposed by Studio | Responsible boundary and required qualification |
|---|---|
| Stable typed queries/actions/results/refusals outside CLI adaptation | YAI application layer; preserve owner checks and one operation meaning |
| Local application hosting/listener and versioned transport mapping | YAI host plus qualified Interfaces projection; discovery, authentication, negotiation, disposal and unavailable posture |
| Incremental progress and scoped subscriptions | YAI lifecycle facts plus transport; ordering, identity, missed updates, resync, backpressure, cancellation and redaction |
| Simultaneous attachments and reattachment | YAI; current Principal/Participant/Thread resolution, concurrent mutation/refusal, stale generations, idempotency and recovery |
| Resource/file changes outside YAI actions | YAI observation/source boundaries; provenance, revision, confinement and explicit admission, including watcher gaps |
| Native provider management evidence | Public provider/YVEX management contracts; truthful capability/version/permission and failure exposure |
| Remote/Mobile consumption | YAI/Interfaces authentication, disclosure and transport contracts beyond local OS trust |

These gaps do not automatically select work or establish an Interlock. X03 and
its dependency/promotion decisions remain in ROADMAP. React cannot close them.

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

Current `open` resolves a locally authenticated Participant and its last
committed thread; a new empty thread stays controller-local until SEND. Current
runtime admission provides single-host mutual exclusion for active Case
advancement. Neither this nor LMDB write serialization qualifies simultaneous
Studio/CLI subscriptions, concurrent SEND semantics, background execution after
detachment or distributed operation. Those behaviors need explicit tests of
conflicts, revocation, reconnect and recovery at the application boundary.

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
into Case authority. REPLAI remains the current CLI mechanics owner; whether any
of its mechanics fit a future pane requires separate integration qualification.
No emulator, PTY host, shell plugin or process launcher is implemented here.

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

Subscription loss, missing events and stale views must be visible. Reconnect
uses qualified snapshots/resync; a frontend must not replay guessed mutations
or independently reduce private Transitions. External file observation and full
dynamic projection remain unimplemented in Studio.

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
capabilities; inference semantics remain generic. No provider UI exists yet.

## Progressive Disclosure

One product serves ordinary users, developers and operators. Start with the
Case, the current work, readable results and explicit review/action choices.
Reveal identifiers, provenance, exact refusals, execution details, raw qualified
evidence and runtime diagnostics on demand. Advanced visibility does not add
permission. Simpler presentation must not hide indeterminate delivery, stale
data, required consent or missing evidence. There are no separate semantic
products or parallel simple/advanced state models.

## Case Workbench Surfaces

Every row is a product target, not an implemented panel. The previous roadmap
surface design is migrated here, with existing owner names retained rather than
new frontend stores. A source explorer organizes the world related to a Case;
Knowledge organizes what qualified sources state. Neither owns the other.

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
| Editor / work surface | Composition, Case/source artifacts and editable derived views | Explicit save/admission through existing or future YAI contracts; no silent source rewriting |
| Computer surface | Targets, frames, observations/actions and receipts | Future governed Computer Use capability |
| Bottom Panel | Logs, evidence, tests, execution/output tools | Presentation of facts from responsible owners |
| Terminal | Real shell/REPL/tool processes | Future native PTY host and terminal frontend |

## Local UI State vs Case State

| State | May remain frontend-local | Must originate from YAI or the named authoritative source |
|---|---|---|
| Active tab, viewed file, layout, panels, scroll, graph positions | Yes | Selection does not change authority or scope |
| Selected terminal, window size and keyboard preferences | Yes | Native host owns real process/FD facts |
| Unsubmitted text, local draft, editor selection | Yes | Explicit SEND/save uses YAI admission; no implicit Turn |
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
has its own Cargo workspace/lockfile and no engine dependency. Core/CLI Make
targets remain independent of Node, Tauri and Studio. The source-placement guard
admits only the added desktop Rust source/build script and excludes generated
Studio dependency/build trees from source classification.

React/TypeScript owns rendering and interaction; Tauri supplies the container
and later strictly necessary native integration. Rust shell code must remain a
thin qualified client/adapter, never a second business layer. No Next.js,
Electron, editor, terminal emulator, graph library, design system or native
plugin is added. Tauri requires a PNG at compile time; the shell uses one
transparent RGBA pixel, not a product icon design. Future modules grow when
there is code to own; the conceptual
`case`, `components`, `surfaces`, `state`, `terminal`, `yvex` placement is not a
requirement to create empty noun directories.

The bootstrap follows the official [Vite guide](https://vite.dev/guide/) and
[Tauri Vite integration](https://v2.tauri.app/start/frontend/vite/). Desktop
prerequisites are isolated from backend prerequisites. Installer packaging,
signing and platform distribution are not qualified by an executable build.

## Live vs Fixture Mode

```text
StudioClient (future qualified application consumer seam)
├── LiveClient
└── FixtureClient
          -> the same React components
```

`src/clients/` reserves placement, without declaring an empty interface or
inventing method names/DTOs. Once YAI qualifies its boundary, LiveClient adapts
that single contract and preserves typed identity, scope, results/refusals and
events. FixtureClient supplies deterministic instances of the same contract;
it does not simulate a second Case engine or grant production actions.

Future fixtures support visual development, screenshots, site/README/docs and
visual regression. They must be plausible, recorded or sanitized with explicit
origin, contract version and missingness. Label fixture mode visibly; never
claim its values are real telemetry or silently fall back from live errors to
fixtures. Neither client, mode selector nor fixture dataset exists in the
bootstrap. There is no fake success path.

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
regression follow real surfaces, not this scaffolding page.

[Validation procedures](test-cases.md#studio-bootstrap-isolation) distinguish
backend publication, frontend build and native shell build. The cumulative
[Golden runbook](zero-to-current.md) remains the product acceptance procedure;
human acceptance requires the operator's result at the relevant SHA. A compiling
skeleton qualifies build isolation only. All promotion stays in ROADMAP.
