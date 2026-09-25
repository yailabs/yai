# YAI Studio — Case IDE / Case Workbench

Authority: canonical Studio product/frontend architecture, subordinate to the
[Constitution](constitution.md) and YAI-owned application semantics.
[ROADMAP](../ROADMAP.md#product-interfaces) alone owns macro state, maturity,
engineering selection, promotion, interlock and backend prerequisites. This
specification is neither an implementation queue nor evidence of promotion.
The subordinate [Studio roadmap](../studio/ROADMAP.md) records implementation
progression and dependencies without owning maturity.

## Purpose and Product Position

The permanent development loop is published YAI/public YVEX contract → explicit
delta classification → sibling CLI/Studio integration → real Case exercise →
owner-level finding. Studio milestones begin with both YAI Capability and YVEX
Public Contract Delta Checks. Only published contracts count as current support;
running Host versions, local edits and public producer observations are separate
evidence. Studio product control remains in `studio/ROADMAP.md`.

`case:tech-infra-inference-service` is the operator's persistent internal
infrastructure-service Case on Exon/Spark, distinct from the technical oracle
`case:studio-live-qualification`. The operator retired `case:yai-enterprise-launch`;
its cancelled/closed history remains retained, and it must not be recreated as
the operational scenario. Reconstructible behavioral Cases supply controlled
counterexamples without resetting operator Cases. A model's service-readiness
opinion is candidate evidence, never deployment approval or a substitute for
validation and human acceptance.

Providers projects Tenant computational inventory independently of a Case's
bindings. The Host carries this service; it does not own provider semantics.
Compute projects the current Case's bound capability and cognitive assignments.
The YVEX platform view lists compatible deployments using qualified extension
metadata, which is not proof of producer identity or native management support.
Public management, inventory and generic inference remain distinct contracts.
For an authorized registered `yvex.http.v1` target, `provider.models` can also
project the exact model's validated public HTTP v3 catalog capacity. Studio dates
that observation and shows input/sequence token limits alongside the exposed
model. The catalog read neither reserves resources nor proves current residency,
health, request fit or a successful generation. Generic OpenAI-compatible
connections retain model-name discovery without a YVEX capacity claim.

Workbench renderers declare Material, Product or Canvas presentation archetypes
independently of their interaction roles. The rail contribution metadata keeps
core YAI, platform and eligible pinned views distinct; fixed defaults cannot be
hidden by local pinning preferences. Neither archetypes nor navigation metadata
create canonical YAI object kinds.

Studio is the official first-party Case IDE / Case Workbench inside YAI.
The Case is the primary visual and operational unit: a coherent workspace for
its sources, knowledge, experience, authority and work. A Case window/workspace
does not require an operating-system process per Case. The filesystem and
conversation are views within that workspace, not its organizing truth.

Studio is not an enterprise dashboard, ChatGPT clone, graphical command catalog,
replacement CLI, independent YVEX UI or new semantic owner. Native CLI, Studio
and future Mobile are clients over the same YAI application meaning, subject to
ordinary authentication, Participant scope, disclosure and admission.

The current [shell](../studio/README.md) has a bounded resident-Host live mode. Its
Start Center lists Cases visible to the authenticated local principal and its
Workbench consumes authorized application projections for Overview, Environment,
Knowledge, Memory, Authority, Work, Compute and committed Conversation Turns.
Fixtures remain an explicit development mode only. Local two-client event
fanout, parallel Case isolation and restart have separate bounded qualification;
they do not establish remote Host transport or universal mutation-conflict
handling. ROADMAP and the retained product tests own the exact qualified scope.

The persistent product oracle for this boundary is
`case:studio-live-qualification` in an operator-owned non-Golden `YAI_HOME`.
It is real Case state, not a fixture or disposable test seed. Its bounded
repository Resource, declared/acquired Sources, source-grounded Knowledge,
policy binding, Workflow and history are created only through normal YAI
operations. Cross-surface qualification compares their semantic identities
across CLI owners, `yai-application` and this Workbench; restarting Studio must
not alter them.

## YAI Product Topology

### CURRENT — resident local application Host foothold

`application/yai-host` now owns one resident Rust application process for an
explicit `YAI_HOME` on the qualified Linux path. The Host constructs one
`LocalApplication`, exposes typed requests/results over a private versioned
Unix-domain transport, observes visible Case generations and fans out bounded
invalidation events to attached clients. Discovery is tied to the canonical
profile identity and live process identity; its run directory is `0700` and the
socket/discovery files are `0600`.

Normal native Studio is a Host client. Its Tauri process discovers or starts
the Host, performs the protocol/profile handshake, forwards application calls
and receives update events. It does not construct `LocalApplication` or poll
YAI persistence. Closing Studio ends its attachment and transient PTYs while
the Host and durable Cases remain alive. Explicit fixture mode does not require
or start YAI, and browser mode has no local Host transport. The native CLI still
reaches several domain owners through CLI/store-coupled adapters, while
`yai host status/start/stop/restart/logs/serve` uses the shared Host lifecycle
library. The current C `yaid` process remains a separate narrow daemon; it is
not the application Host.

The same facade now exposes machine-readable `application.capabilities` metadata
and a separate current `case.capabilities` view. The former describes stable
product support, operation contracts, impact and client posture; the latter is
Case/Participant-qualified Resource requestability and never a grant. Studio
discovers the connected Host's actual catalog and invalidates it on connection
loss or instance change. Settings > Advanced distinguishes advertised operations
from explicitly integrated Studio interactions; `application_ready` does not
imply a complete UI or current execution permission. See the generated
[capability matrix](reference/application-capabilities.md).
The code-owned catalog and
[Studio consumer map](../studio/src/contrib/settings/capabilityPosture.ts)
provide the current operation inventory; the catalog-derived parity check reports
counts without treating them as behavioral coverage. Studio now authors ordinary
text SEND, exact Case resume, retained-Turn realization/composition and controlled
effect submission/reconciliation, as well as Source acquisition, Resource requests
and execution observation. Their consumer entries name the product suites and
remaining action-level limits, including multimedia composition and process
reconciliation. The map also records alternative presentations and unconnected
operations such as cognitive planning and ambient refresh assessment.
Ambiguous delivery stays indeterminate unless its domain permits reconciliation;
Application readiness is not universal recovery or complete Studio parity.

Authored live forms currently consume Case creation/cancellation/closure,
Participant role addition and authenticated self-linking, Review
approve/deny/defer, and Workflow HumanInput. Creation, Participant setup and
identity linking remain separate explicit YAI actions; opening a window does
not implicitly grant authority. Review decisions do not execute their effects.
Successful actions refresh the authorized projection. Refusals retain the form
and explain the supplied reason. If a transport acknowledgement is lost, the
action may already have committed: Studio blocks resubmission in that dialog
and asks the operator to inspect current state. There is no optimistic canonical
mutation, frontend policy evaluation or automatic retry of a mutation.

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

The application-Host portion of this topology is current executable behavior on
the qualified Linux path, including bounded supervision of the existing runtime
scheduler. Platform login autostart,
cross-platform local transports, remote clients and provider supervision remain
target properties.

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

CURRENT: the resident Rust-owned Local Host supervises the existing
RuntimeInstance alongside the application and update services. This
placement does not make RuntimeInstance the Case owner. The host target is
Rust-owned because the current application composition, runtime scheduler and
Case owners are Rust; this does not authorize a second C/Rust lifecycle owner.

OPEN: the implementation must characterize remaining `yaid` consumers and
compatibility obligations before any drain. Current `yaid` is not promoted or
renamed into the product host, and this specification does not remove it.

### Lifecycle, autostart and host telemetry

The current headless lifecycle surface is:

```text
yai host status
yai host start
yai host stop
yai host restart
yai host logs
yai host serve       # low-level foreground/service entrypoint
```

These commands manage the host process, never Case lifecycle. `Settings > YAI
Host` consumes real process/application/client telemetry and exposes qualified
stop/restart controls. Desktop settings may later offer `Start YAI automatically`
using qualified user-level
platform mechanisms such as a Linux user service, macOS LaunchAgent or Windows
user startup/service mechanism. Studio must still be able to start the host when
it is absent.

The singleton surface displays only authoritative facts currently exposed by
the Host: status, PID/process identity, uptime, version/build, `YAI_HOME`,
protocol, transport, application readiness, attached clients, event sequence
and activity. Runtime supervision is displayed from the actual Host response;
an older Host may still report `not_integrated`. Unavailable OS or scheduler
telemetry is not fabricated. Host telemetry is operational
process state, not Case state.

Provider processes have an independent lifecycle. Starting YAI does not load a
model, allocate a GPU, launch YVEX or start llama.cpp/vLLM. Cloud and local
providers remain selected capabilities with their own availability. Future YVEX
supervision belongs to its native management plane, not generic host readiness.

### OPEN implementation boundaries

The Host does not yet converge ordinary domain CLI calls on Host transport,
provide durable event replay, qualify every multi-client mutation conflict,
start at OS login or implement macOS/Windows
local transport. Reconnect is snapshot/resync based because the event sequence
is process-local and non-canonical. No LAN listener, remote serving,
mutation-complete API or provider supervision is selected by this document.

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
| Cross-platform local and future remote transport | YAI host plus qualified Interfaces projection; the current private Linux Unix transport is internal and local only |
| Incremental progress and general scoped subscriptions | YAI lifecycle facts plus transport; current Case invalidation and process-local sequence still need durable replay, broader progress, backpressure, cancellation and redaction qualification |
| Simultaneous mutation and stale-write recovery | YAI; multiple attachments and snapshot reattachment are qualified, while concurrent mutation/refusal, stale generations and idempotency remain open |
| Resource/file changes outside YAI actions | YAI observation/source boundaries; provenance, revision, confinement and explicit admission, including watcher gaps |
| Native provider management evidence | Public provider/YVEX management contracts; truthful capability/version/permission and failure exposure |
| Remote/Mobile consumption | YAI/Interfaces authentication, disclosure and transport contracts beyond local OS trust |

These gaps do not automatically select work or establish an Interlock. The
current Tauri bridge is a narrow Host client: it binds no listener, sends typed
requests over the private local socket and consumes Host invalidations. React
cannot close the remaining gaps.

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

For the resident Host, the same archaeology recovered four bounded
properties: same-machine-only endpoints, a private endpoint root and `0600`
socket posture, explicit discovery source/status, and handshake/version mismatch
as a first-class refusal. The historical listener removed only a stale socket it
owned and cleaned its socket/discovery paths on failure and stop. These belong
to the current Linux transport. Its dev-only single-client probe dispatcher,
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

### One Workbench, two first-party systems

Studio is the graphical operating surface for both YAI and YVEX. YVEX is not
reduced to a provider settings page. YAI owns Case semantics, authority and
context; YVEX owns source/package compilation and computational execution.
Studio composes their qualified interfaces without acquiring either ownership.
Providers remains Tenant inventory/governance, Compute remains Case-scoped use,
and YVEX is a platform workspace spanning the source-to-result lifecycle.
A YVEX Source, Profile or Session is not a YAI Source, Case or Participant.

The selected interface has local navigation for **Models & acquisition**,
**Preparation & artifacts**, **Runtime & machines**, **Sessions & generation**,
and **Observability**. Routine tasks use compact authored flows; exact lineage,
tensors, policies and diagnostic evidence use Inspector/advanced views. Acquisition,
compilation and loading expose owner-authored progress and failure, not estimated
completion. Model identity, representation, profile and engine generation remain
distinct. A downloaded checkpoint does not imply an executable model.

Case requirements inform selection and compatibility, not silent calibration.
Representation/quantization, calibration input and runtime policy remain explicit
YVEX-owned choices with provenance and feasibility evidence. YAI supplies its
required modalities, context and authority; Studio exposes mismatches and asks for
an explicit supported choice. It does not truncate context, tune an engine or
unload another Case's shared deployment automatically.

### Published producer reconciliation

The reviewed public cuts are YVEX `main` at
`3f4a1c182d35e5a0e163adb81008ae7a366efcc6` (local protocol 20) and `models2` at
`dabbc09dacf500684b60a3ae8675956a016436b9` (local protocol 22, HTTP compatibility
v3). This published source cut does not prove that the running deployment has
been upgraded or reloaded. Review the pinned
[operation registry](https://github.com/yailabs/yvex/blob/dabbc09dacf500684b60a3ae8675956a016436b9/config/operator/registry.json),
[command ownership](https://github.com/yailabs/yvex/blob/dabbc09dacf500684b60a3ae8675956a016436b9/docs/architecture/commands.md),
[compiler architecture](https://github.com/yailabs/yvex/blob/dabbc09dacf500684b60a3ae8675956a016436b9/docs/architecture/compilation.md),
[local protocol](https://github.com/yailabs/yvex/blob/dabbc09dacf500684b60a3ae8675956a016436b9/docs/contracts/local-protocol.md),
[telemetry contract](https://github.com/yailabs/yvex/blob/dabbc09dacf500684b60a3ae8675956a016436b9/docs/contracts/events-telemetry.md)
and [HTTP profile](https://github.com/yailabs/yvex/blob/dabbc09dacf500684b60a3ae8675956a016436b9/docs/openai-compatibility.md).

| Producer boundary | Executable scope at the reviewed cut | Studio integration posture |
|---|---|---|
| Finite offline owners / installed ABI | Search, HF/local acquisition, resume/stop, verification, preparation, artifacts, profiles, quantization and inspection | Selected product/advanced flows; no current shared YAI typed management carrier. Do not parse CLI output or link compiler ownership into React. |
| Private local protocol v22 | Host status/memory/logs/stop, engine lifecycle, leases, sessions, generation, cancellation, events and preflight | Producer functionality exists. Versioned UID-local transport is explicitly private, not an authenticated remote SDK; a qualified management boundary is required. |
| HTTP compatibility v3 | Health, loaded-model discovery, exact model/capacity identities, preflight, Chat Completions and Responses subset | Generic governed inference and exact preflight already have YAI consumers. Raw producer data availability does not imply full Studio projection. |
| Native media generation | Typed directional capabilities and admitted media-engine results/progress | Do not promise HTTP multimedia parity: the reviewed compatibility profile refuses multimodal input and image/audio endpoint families. Case output/evidence admission and a typed media route must be qualified. |
| Remote machines | Local host/runtime facts do not establish network identity or remote trust | Enrollment, discovery, bootstrap, authenticated management and revocation remain shared YAI/producer contract work. |

The earlier public delta at YVEX
[`671a5befa0c20f6e07248b0223fad4ec531c15bd`](https://github.com/yailabs/yvex/blob/671a5befa0c20f6e07248b0223fad4ec531c15bd/docs/contracts/remote-management.md)
adds `device.describe` and `host.status` through a dedicated restricted SSH
bootstrap. It requires independently approved host-key pinning, enrolled peer
keys and a forced protocol command. This does not permit ordinary remote-shell
administration or remote lifecycle/log/model controls. Shared YAI consumption
and Studio interaction remain unqualified; producer availability is recorded
separately from client coverage.

The current `dabbc09` delta adds `model load MODEL --ctx N` and advanced
`engine load PROFILE --ctx N`, backed by local protocol v22's
`load_context_capacity`. Zero retains the registered default; a positive value
requests a new text-engine generation subject to compiled model limits and
resource admission. Existing loaded aliases are not resized or silently unloaded;
media engines refuse the override. Protocol v21 clients are refused explicitly.
This is published producer capability, **partially consumable**: generic YAI
preflight can observe an admitted capacity, but Studio cannot issue native loads
until shared management is qualified. The restricted remote bootstrap still
exposes only identity/status. Operator-requested 4K qualification continues
without reloading; successful small-Case inference does not qualify the full
infrastructure Case or a larger deployment.

Compilation has coordinated computation and parameter/package lanes. Package
lineage joins executable specialization before an engine is loaded. A future
pipeline UI must preserve this structure rather than draw a fictitious universal
linear “tensor → GGUF → generation” compiler for every family.

Coverage means supported semantic actions with authored interactions and tests,
not all registry rows turned into buttons. At the reviewed cut the registry has
172 entries: 14 removed, 33 product-default, 64 product-advanced, 54 engineering,
6 API-only and 1 automation. These are producer inventory categories, not a Studio coverage
percentage or 158 independent UI requirements. REPL-local, protocol, duplicated
porcelain/plumbing and diagnostic actions require explicit dispositions. No
“90% required” or “90% integrated” claim is qualified. Regenerate the inventory
from the pinned producer registry on the next delta check; do not maintain a
second operation registry here.

### Inference plane

YVEX continues through the same qualified generic OpenAI-compatible provider
boundary as other compatible runtimes. No `if provider == YVEX` branch belongs
in generic cognitive execution. YAI owns context meaning, target qualification,
selection and admission; YVEX owns its exposed model/runtime realization.
Provider output remains candidate material. Computational continuity is neither
Case continuity nor evidence of a successful external effect.

### Management plane

The selected product requirement is a YAI-owned machine and provider-management
capability exposed through typed Application operations to both CLI and Studio.
It must discover eligible machines on the LAN, explicitly enroll a Tenant machine
asset, retain its verified identity across launches, select its execution
location, start/stop the runtime, load/unload a model and observe supported
status, memory accounting and bounded followable logs.
For example, Studio on Exon selects the registered DGX Spark; execution and
telemetry belong to Spark, not to the UI machine. An address or display name is
not a durable device identity, and discovery does not grant trust or authority.

YVEX owns its published device identity, discovery proof and runtime management
semantics. YAI owns Tenant-scoped candidate qualification, explicit registration,
the authenticated association to that identity, Case-independent device inventory,
execution-location selection and application-level refusal/evidence meaning.
CLI and Studio consume those same operations; neither owns a private discovery
or startup path. One enrolled compute machine may serve several Cases without
inheriting authority or disclosure from any of them. Registration must
distinguish a known offline device from an unknown peer or changed identity.
Starting a stopped remote runtime requires a reachable authenticated
management/bootstrap carrier on that machine; the stopped inference listener
cannot provide its own startup mechanism.
Transport, enrollment/revocation and lifecycle contracts must be qualified before
this interaction can be implemented. Do not infer them from CLI help, a local
Unix socket path, or generic OpenAI-compatible HTTP. A shell/SSH command runner
in either YAI backend or Studio does not substitute for that contract.

Additional public YVEX product capabilities may include installed
and admitted models, engines, load/unload, deployment, compilation,
memory/residency, sessions, runtime evidence, device state and authoritative
telemetry. Runtime sessions here belong to YVEX, not a new YAI Case Session.

Such controls require real versioned public management capabilities, permissions,
results and truthful failure states. They must not bypass YAI inference
qualification, read YVEX internals or make YVEX a Case owner. No management API
is invented or implemented. Normal provider qualification remains black-box
consumption as specified by AGENTS; an operator's separately authorized producer
deployment attempt does not establish YAI machine-management capabilities.

## Provider Experience

Target a common experience for generic OpenAI-compatible endpoints, vLLM,
llama.cpp, OpenAI, Anthropic and other providers. This is a UX horizon, not a
claim that all protocols or adapters exist today. Show actual capabilities,
model identity, qualification, unavailable/refusal posture and provenance.
Discovery, operator consent and suitability remain YAI application semantics.
YVEX is a strategic first-party integration with additional qualified management
capabilities; inference semantics remain generic. Studio now renders the generic
provider target/posture exposed by YAI, with sanitized endpoint identity. Compute
supports typed target registration, import of measured qualification evidence,
trust decisions, explicit Case binding, owner-authenticated exposed-model discovery,
primary-conversation suitability attestation and cognitive assignment. Qualification records supplied
evidence; it does not perform an undisclosed probe. Native YVEX management and
Tenant-wide target discovery remain unimplemented.

The UI preserves four different concepts:

| Concept | Meaning |
|---|---|
| Model | The exposed model identity/capability, such as Qwen or Llama |
| Provider / runtime | The service or runtime that realizes inference, such as an OpenAI-compatible endpoint, vLLM or llama.cpp |
| Target / deployment | The exact provider + model + endpoint + locality + qualification/health combination |
| Case binding | The governed relation that makes an exact target available to one Case |

YVEX may realize an inference target through this generic plane. Its future
first-party management surface remains a separate plane and does not change
generic cognitive execution.

## Progressive Disclosure

One product serves ordinary users, developers and operators. Start with the
Case, the current work, readable results and explicit review/action choices.
Reveal identifiers, provenance, exact refusals, execution details, raw qualified
evidence and runtime diagnostics on demand. Advanced visibility does not add
permission. Simpler presentation must not hide indeterminate delivery, stale
data, required consent or missing evidence. There are no separate semantic
products or parallel simple/advanced state models.

For product identity, a canonical reference and a primary human label are
different presentation facts. `case:studio-live-qualification` remains the
durable lookup and technical identity; ordinary navigation presents “Studio
Live Qualification” and exposes the canonical ref in Inspector/technical
detail. The bounded application fallback derives a readable Case label because
YAI does not yet own a persistent Case display-name contract. This rule does
not authorize generic prefix stripping for unrelated identifiers.

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

Exact `material.read` content, whether UTF-8 or base64, is identity/digest-checked
before any trusted renderer receives it. Live image/PDF/audio/video paths use
those retained bytes, never a filesystem shortcut. SVG is displayed only as an
inert image. Markdown uses a lazy maintained parser with raw HTML disabled;
remote images are not fetched and only links resolving to qualified inventory
become Workbench navigation. Markdown, SVG and CSV previews may show an explicitly
labelled local draft; that draft has not been admitted to the Case. CSV/TSV use a
lazy parser, an explicit first-row-as-column-names preference, bounded 50-row
pages, full-input filtering/sorting and keyboard/pointer column sizing. Parse
failures retain the Text Editor route; table rows are local representations of
their material, not newly invented canonical objects.

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

CURRENT editable state is deliberately bounded. Studio-local Settings may
change versioned local preferences. A trusted text/code Surface can hold an
exact retained text revision in a local buffer, with cursor/selection, line
numbers, undo/redo, find/replace, dirty state and explicit revert. This editing
capability does not authorize persistence. No participant-origin filesystem
content mutation is currently qualified, so Save is disabled and closing a
dirty Surface requires explicit discard. A future governed Save must pass the
typed application boundary, existing Resource/authority/admission owners and
revision conflict checks before a refreshed projection can become clean; React
never writes a source path directly.

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
Explorer/context widths and the bottom-panel size use the same local preference
owner as Settings and resizing. Invalid persisted numbers/types are ignored,
numeric edits are validated, and terminal scrollback updates its existing
renderer. Compact Settings navigation responds to available Surface width.

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

Current bounded graph presentation accounts for every qualified endpoint.
Endpoints without projected object detail remain explicit references. Full-set
search, paginated neighborhoods, keyboard selection and coordinate-correct pan,
zoom and fit are local presentation state. Visible/total counts disclose the
relations outside the current page or filter; absence from the canvas is not
absence from YAI. Knowledge collections distinguish documents, units, entities,
topics and contradictions without inferring missing categories. Inspector
exposes exact projected text and provenance references through shared local
navigation. Rich temporal and graph exploration remains a further S4 program.

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

Environment preserves three object kinds. A **File / Material** is one concrete
readable revision. A **Source** is the governed Case relationship that declares
perimeter, roles, Resource, acquisition posture and revision. A **Resource** is
an attached operational capability with its own type, policy/review posture and
disclosed operations. The Environment sidebar builds the file hierarchy only
from qualified projected `file.path` facts, while Source and Resource rows open
dedicated typed Surfaces. Opening a Source therefore never masquerades as
opening its file. File content stays central and minimal; path, canonical refs,
digest, revision, media type, Source, Resource and provenance belong to
Inspector technical detail.

Qualified media type selects the default trusted renderer. Textual formats use
the text/code Surface, browser-safe images use image presentation, PDF uses its
document renderer and audio/video use native media playback. `Open With…`
selects another admitted renderer for the same Surface identity, including
Markdown edit/preview, SVG code/preview and CSV text/table; this choice is local
interaction state and does not create a Case object. Finer Source distinctions
than the current canonical actions (`discovery`, `database_query`, `http_fetch`)
remain a backend model gap rather than a filename guess in React.

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
other content use one reusable preview tab until explicitly pinned; Source and
Resource Surfaces retain their own typed identity. Quick material navigation
does not replace an open Environment or Knowledge perspective. The
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

Every primary Case perspective evolves from passive readiness/reporting toward
a real operational Surface as typed YAI application capabilities become
available. That evolution never licenses a frontend workaround: when an owner,
projection or mutation contract is absent, Studio reports the gap and retains
read-only/local interaction posture.

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
has its own Cargo workspace/lockfile and depends on the bounded YAI Host client
crate, not a second semantic layer. Core/CLI Make
targets remain independent of Node, Tauri and Studio. The source-placement guard
admits only the added desktop Rust source/build script and excludes generated
Studio dependency/build trees from source classification.

React/TypeScript owns rendering and interaction; Tauri supplies the container,
Host client bridge and native capabilities. Rust shell code remains thin; typed
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
small presentation types. Normal mode currently requires the Tauri Host-client
adapter; an absent Host, authentication failure and unsupported projections remain explicit result
states. It never knows LMDB layout, CLI syntax or private Rust domain structs.
The application projection is intentionally smaller than `CaseState` and does
not predeclare every future operation. Its local Host transport is not a stable
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

## Workbench interaction lifetime

The application window owns local Case interaction sessions: each Case has its
own Surface group, drafts and navigation. An attachment or snapshot change does
not recreate these owners. Dirty previews pin automatically; exact material
identity, rather than a reusable tab identifier, resolves open Surfaces. Late
reads and mismatched returned bytes fail closed. Incoming authoritative content
replaces clean buffers and marks dirty buffers stale until explicit reload.

PTYs belong to the desktop window independently of Case data and panel
visibility. Closing the desktop releases its PTYs; it leaves the resident YAI
Host alive. Window-manager close and application close share the local draft
guard. Draft retention is window-local and does not imply persistence or Save.
The initial window fits the current monitor work area. Trusted editor-generated
styles reuse Tauri's response nonce; the desktop CSP remains restrictive.

Historical archaeology reconfirmed the operational attachment/continuity
distinction in `yai-dev` client-connection code at `5c1c7b9d0` (last relevant
change `e9ad7f498`) and pre-drain IPC cleanup at `dda93ee3`. Those mechanisms
provide no reusable frontend draft or tab implementation. Studio retains its
current Workbench owners; historical Session/Agent runtime ownership is rejected.

### Governed Source setup

Environment and Resource Surfaces offer declaration against already-attached
Resources. The form uses the projected operation names and, for database/HTTP
Sources, only exposed named requests. Roles and bootstrap intent are explicit.
YAI validates the actual perimeter, Participant and authority; a declaration
does not acquire bytes or publish policy. Source revocation requires a reason
and retains canonical history. Acquisition and explicit resume use the published
typed operations, expected generation, exact attempt and prior progress reference.
An interrupted attempt is not assumed to be running or automatically resubmitted.
General Resource attachment still requires a qualified native binding/carrier;
Studio does not reconstruct it from a path. Process attachment uses the published
same-user PID/identity capture with explicit bounded signal actions and review;
attachment itself sends no signal.

### Case policy binding interactions

Authority offers binding, same-lineage replacement and unbinding of an exact
published policy artifact. These actions capture the Case generation when the
operator opens the form; resync never silently rebases an open decision. YAI
checks publication, Tenant authority and generation. Stale refusal exposes an
explicit close-and-refresh path without resubmission. Artifact import, lifecycle
management and Tenant-wide policy browsing remain separate interactions; a bound
policy projection is not an artifact catalog.

### Focused editing commands

Workbench Edit commands resolve the focused editor, native text field or terminal
through a local editing service. Menus and the command palette preserve their
originating control. Undo/redo enablement follows the actual editor history;
read-only content and PTYs do not advertise unsupported edit actions. Clipboard
operations are local host/browser interactions, never YAI mutations. A pending
paste is discarded if focus, document identity, selection or local text changes,
or its terminal is hidden/closed. Refused clipboard access produces a visible
notice instead of a silent failure. Native editor and terminal key ownership is
preserved.

The Workbench can focus any registered Surface by temporarily hiding neighboring
regions. This is a local layout choice: groups, local buffers, terminal processes
and hidden region state remain alive. Restore returns the prior visibility and
geometry. A direct region command exits focus mode and reveals that region; it
does not create another window or alter Case state.

## Operational Case surfaces and governed intake

Studio's primary perspectives evolve from reporting toward operational surfaces
as typed YAI capabilities qualify. Navigation, menus, search, settings and panels
extend the same Workbench; they do not establish parallel application owners.
Back/Forward records Case-local presentation locations, including active Surface
and explicit Inspector selection. Activity remains a concise latest-20 feed.
Journal is a searchable, pausable projection of committed Transition history;
its current latest-160 bound is disclosed and is not a full ledger browser.
Executions separates committed lifecycle history from exact operational
observations. Work can submit bounded tasks, request cooperative stop of an exact
runner, and observe source attempts or Resource effects through `execution.get`.
Window-local, Case/Participant-scoped recovery references are retained before
submission; they contain no task, result, authority or canonical execution state.
After lost acknowledgement, reconnect or remount, observation never redispatches.
The latest twelve local references are not a complete execution catalog. Result
bodies and full receipt navigation remain separate projection boundaries.

A policy document is a carrier with provenance, not an authority role inferred
from its filename. Qualified Source roles route policy-only Sources and their
policy material to Authority. Mixed documentary/normative material can retain
its documentary Source and exact original while its qualified normative region
participates in the policy lifecycle. Studio must not remove documentary
provenance or silently treat ordinary JSON, Markdown, TOML or prose as authority.
The current explicit policy upload sends exact selected bytes to `policy.ingest`.
YAI's supported compiler profiles, validation blockers, typed rules, conflicts
and lifecycle results determine what is renderable. Publication and Case binding
remain separate explicit actions. Neither drag-and-drop nor an LLM can activate
policy. Recursive folder intake and a mixed-region routing preview require
qualified acquisition and route projections; Studio currently refuses directory
drops rather than scanning the filesystem.

Authority renders typed rule effects, subjects/roles, operations, constraints,
reasons and provenance where returned by the owner. Current Case bindings are
separate from transient imported-artifact results. The Application does not yet
supply a general policy catalog/read operation, combined EffectivePolicy
explanation or policy simulation. Reconstructing those in React would create a
second authority evaluator. A future explanatory model may narrate qualified
rules and their interactions, but its output remains candidate explanation,
never permission or an executable policy decision.

Recall exposes documentary evidence, historical experience, exact cut, selection
reasons and closure limitations. Working State exposes current mandatory control,
explicit evidence groups, budgets, omissions and W4 paging. Expanding a group is
an explicit operation. Decision Frontier preparation requires actual alternatives
from the exact requalified Working State; request preparation neither scores nor
executes them. No retrieval score becomes confidence, and a captured result does
not attest current authority after the Case changes. Ambient refresh belongs to
an actual consumer lineage; an inspection Surface cannot invent that lineage.

Work composes Workflow, Handoff and current projected operational history.
Workflow authoring currently supports bounded human checkpoints and explicit
patch adoption; YAI retains topology and progression semantics. Handoff uses
exact same-Tenant Case identities and transfers neither authority nor Resources.
Absent inbox/effective-topology/full-result projections remain visible boundaries.
Resource requests use qualified bound configuration digests and authored variants:
filesystem read/search, discovery, named read-only database query, HTTP fetch,
process runner and MCP catalog. No arbitrary SQL, shell command or URL is supplied
by these forms. Receipt outcome describes admitted external execution; it does not
imply a process exited successfully. Current authority is rechecked on observation.

Knowledge offers owner-backed inspect, lexical search, exact unit resolution and
documentary navigation separately from filtering the already projected collection.
Results are fenced by Case/generation/source identity; revocation or a changed
snapshot invalidates them. Retrieval relevance never becomes confidence or truth.

Compute distinguishes model identity, provider adapter/runtime, exact deployment,
qualification, trust, health observations and Case binding. Registration does not
qualify a target. Importing measured probe evidence records an existing observation;
it does not run a network probe. Binding does not bypass execution-time authority.
YVEX compatibility is a generic OpenAI-compatible target with optional extension
telemetry; it is not proof of producer identity or native management. Operational
provider/YVEX controls belong in Compute; Settings contains local preferences and
navigation to that management Surface.

Graphs occupy the Work Surface, with shared pan/zoom, drag, search, filters,
selection, neighborhoods and relation inspection. Knowledge uses associative
layout; Experience uses temporal layout. Workflow retains directed progression.
Layout and clustering never create semantic relations. Unresolved exact refs
remain explicit. Historical archaeology inspected `yai-dev` at `5c1c7b9d0`,
including the non-executable graph view stub and the adjacent lineage graph
projection boundary: exact relation identity and explanation are retained in the
current projection/Inspector seam; historical UI/runtime ownership is not copied.

### Conversation execution in Studio

The native composer calls `conversation.send` through the resident Host. It
retains the exact submission envelope before dispatch. Committed Turns and
`execution.get` results are rendered independently; no optimistic Turn or
invented assistant reply enters the transcript. Lost acknowledgements are
observed first and can retry only the same envelope. Stale generation refusal
preserves the local draft. A projected `execution_request_ref` lets another
Studio process recover the recorded response without dispatching it again.
Result observation rechecks current authority. Draft/envelope retention is
window-session storage, partitioned by Case and Participant, not Case truth.
Text is currently bounded to 64 KiB; streaming and attachment authoring remain
unimplemented. The immutable Conversation projection remains a read surface;
SEND availability comes from the current operation catalog and YAI admission.
The current Participant's `model_context` view admission is projected by
`case.summary`. Studio requires an explicit positive projection before enabling
Send; an absent or older Host projection remains unknown and blocks sending.
Working State offers the existing owner-checked admission action. It never
admits a view automatically, and the Application rechecks current authority at
submission. Moving to Working State preserves the unsent Case/Participant draft.
The composer has a compact `+` tools menu for authored Case navigation and a
per-SEND Standard/Fast Search memory preference. It does not register or select
a System Model as the primary conversation provider. The exact preference is
retained in the submitted envelope for lost-acknowledgement retry, and the
Application result reports the effective path. Today the public non-generative
producer is unavailable, so a Fast request visibly degrades to qualified
standard Recall/W; no score, recommendation or model execution is implied.

Compute separates measured wire qualification from operator-attested semantic
suitability and the primary cognitive binding. `provider.models` authorizes the
Tenant owner before bounded generic catalog access, and reuses the CLI discovery
algorithm. Catalog metadata neither registers a target nor qualifies it. YVEX
compat.v3 HTTP has no Source/Artifact acquisition or engine-load management API;
the producer's local management plane requires a separate qualified connection.
Studio displays that boundary explicitly instead of issuing remote shell commands.

### Independent contextual tools

Conversation, Inspector and Activity are mounted Workbench contributions. Each
can remain docked or become a movable, resizable card inside the same WebView;
they do not create native windows or Host clients. Closing a card hides it;
the compact contextual launcher reopens it. Conversation retains the same
composer and execution observation during layout changes. Inspector can pin an
exact object while other selection changes; an explicit relation link inside
the card replaces that pinned target. Technical metadata remains collapsible.

Card arrangement, file buffers, tabs and navigation are partitioned by Case and
Participant in the window-local session. Returning to a Case restores its
arrangement; this is not a shared or persisted Case View. Cards are constrained
to the viewport on resize. Their move/resize handles accept arrow keys, close
returns focus to the Work Surface, and Focus Work Surface temporarily hides all
contextual tools without disposing them. Reset Layout restores the docked layout.


### Operational Overview and Telemetry

Overview composes current qualified Case facts, Workflow prompts, Source links and
recent committed history. Its separately requested model explanation uses the
existing `conversation.send` and `execution.get` operations. It persists the exact
submission envelope before dispatch, observes on reopen and never automatically
regenerates or redispatches. Like Conversation, generation requires the current
Participant's explicitly admitted `model_context` view; an absent projection
blocks it and links to Working State for the owner-checked admission. Reading an
already retained explanation remains separately authorized. Candidate text is not
authority. Its Markdown supports
readable structure; links can inspect only refs disclosed in the current Case
projection, with no remote images or external navigation. A changed Case cut is
explicitly distinguished from the retained explanation.

Telemetry is a Product Surface consuming existing Host telemetry and Case
projections. Host PID and client process facts are observations; Resource inventory
and model endpoints are configuration. They are not database health, endpoint
reachability or a complete OS process list. Those require a qualified backend
observation contract. Existing provider health observations are shown with their
recorded time, circuit posture and failure count; they do not imply a new probe. No Studio-side process scan or network probe is introduced.
The existing Application capability entries remain exact: this is another authored
consumer of Conversation/execution operations, not a new semantic capability.


Providers/YVEX product workspace uses a compact selectable deployment inventory
and local Runtime/Evidence/Platform navigation rather than the document-page
composition. Runtime presents the authorized recorded health, time, circuit and
failure class. Evidence scopes the existing governance actions to the selected
immutable target. Inspector owns exact technical metadata. Both inventory and
provider Inspector perform bounded ten-second reads while visible; they do not
probe the producer or initiate inference. Platform identifies the unconsumed
native management boundary without manufacturing compiler, residency or log
telemetry. Existing `provider.inspect` and `provider.governance` catalog ownership
and operation identities remain exact; this presentation change adds no operation
or backend maturity claim. CLI and Studio still consume the same provider owners.


Provider registration presents catalog discovery and an explicit model selector.
The contribution fences asynchronous catalog responses by connection edit/request
identity and invalidates model selection on endpoint/locality/credential changes.
An explicit manual identity alternative preserves generic providers without model
catalog support. Existing `provider.models` and `provider.register` contracts and
`provider.inspect`/`provider.governance` capability identities remain exact; this
change adds no backend operation. Compute and Conversation interaction suites
exercise the real Host boundary, including delayed-catalog rejection.


Archived execution context projects Working State selection decisions alongside
wire capacity observations: pinned items, selected optional items, omissions and
owner-authored reasons. Studio does not classify authority or estimate tokens from
semantic units. This retains the existing execution.get disclosure/generation
fence and introduces no new capability; absent archived decisions remain unknown.


Registered deployment catalog checks use `provider.models` with Tenant and exact
`target_ref`. Application resolves the retained endpoint/locality/credential
reference after owner authorization; Studio never reconstructs a connection from
its redacted inventory. The original explicit-connection input remains supported
for registration. Ambiguous mixed inputs refuse. Responses bind the target and
observation time. Catalog presence is metadata, not engine residency, qualification,
capacity admission or a provider-health mutation. `yai provider models TARGET
--tenant TENANT` uses the same typed dispatcher. Workspace checks are explicit,
read-only, and keyed by exact Tenant/target in bounded window-local state.
Workspace and status bar share that observation across navigation; Host loss or
capability refresh invalidates it. Late reads cannot replace newer checks or
cross Host/Tenant/target identity. Catalog timestamps are explicit; this is not
continuous monitoring, residency evidence or a new capability.

Native Host auto-start uses the same Rust product-process composition as
`yai host serve`, including the existing RuntimeInstance supervisor. The desktop
links the narrow `yai::serve_application_host` entry; it does not execute or
parse CLI commands. An existing CLI-started Host is insufficient proof of this
path: native qualification starts without a Host and checks both the desktop
executable identity and `supervised_running` before work is accepted. The
`platform.local_host` and `platform.runtime_host` capability identities and
existing typed operations remain exact; no new scheduler or authority is added.

Conversation and Overview explanation polling follows only Application `admitted`
or `running` execution observations without a recorded result. `unresolved` is
not evidence of active work: the view offers an explicit status check and never
redispatches it. Terminal responses stop polling within that observation cycle;
Case updates and manual checks reapply current disclosure. The existing
`execution.get` read contract remains exact. `conversation-observation.mjs`
qualifies these presentation behaviors separately from governed Host execution.

Telemetry uses a compact Product Surface with one selected observation section.
Its toolbar tabs and secondary sidebar share an existing window-local context
key; keyboard arrows/Home/End use the same selection. Navigation does not alter
the browser hash or create Case history. Host, Runtime, Shells, Clients, Resources,
Endpoints and Executions retain their distinct observation scopes. Exact execution
receipts mount only when that section is selected; selection adds no health probe.
