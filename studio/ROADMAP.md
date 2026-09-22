# YAI Studio capability roadmap

Studio execution control. The root [YAI roadmap](../ROADMAP.md) alone owns
X03/X04 maturity and global selection. [Product doctrine](../docs/studio.md)
owns architecture; [Studio README](README.md) owns build/run commands; the
[cumulative acceptance runbook](../docs/zero-to-current.md) owns operator checks.
Counts below are inventory, never a percentage of product completion.

| Status | Meaning |
|---|---|
| 🟢 ESTABLISHED | Bounded executable property qualified by retained tests. |
| 🟡 PARTIAL | Useful behavior exists; the named remaining boundary is open. |
| 🔴 OPEN | Selected capability is not established. |
| ⚪ LATER | Explicit horizon, not current implementation. |

<!-- maturity-counts:start -->
Generated from the capability board: 🟢 ESTABLISHED **22** · 🟡 PARTIAL **17** · 🔴 OPEN **2** · ⚪ LATER **3** · **44 properties**.
Regenerate: `python3 tools/validation/check_studio_roadmap.py --write`.
<!-- maturity-counts:end -->

## Execution control

**CURRENT:** `STUDIO.CASE.IDE.OPERATIONALIZATION.0` — bounded operational
foundation established. The qualified interactions below consume published
owners; the remaining partial properties do not become complete by publication.

**ACTIVE / NEXT:** consolidate S1 runtime supervision on the published execution
foundation. The existing scheduler remains the sole runtime owner; Studio does
not create another scheduler.
**THEN:** deepen S4 typed Memory/navigation, S5 governed authoring/intake, S6
Authority/Work and S7 generic Compute against the named Application boundaries.
**BLOCKED:** governed Save, folder intake with qualified routing, policy catalog
and combined-policy explanation, Conversation SEND and ambiguous-delivery recovery.
**HORIZON:** native YVEX management, external/mobile clients, Computer Use,
shared declarative Case Views and public extensions.

### Backend and Capability Delta Check

Last Studio review: `d049888b4f273748657a24e02245b979ccb73e02`.
This implementation started on published
`6482df05858a14c8bc138943932f72fca492af01`: `1f724c13` added the bounded provider
latency diagnostic and `6482df05` excluded Cargo output from placement checks.
During validation, `28001995e3d83d581c0b170b56cedaff1797b518` published the execution
foundation. With explicit operator authorization, only the isolated Studio commit
was rebased onto it. Independently owned work remains untouched.

The new published delta adds eleven operations: `case.run`, `case.stop`,
`execution.get`, `source.acquire`, `source.resume`, `resource.attach_process`,
`resource.request`, `knowledge.inspect`, `knowledge.search`, `knowledge.resolve`
and `knowledge.navigation`. The catalog now exposes 72 operations; partial family
blockers remain exact. The Host supervises the existing RuntimeInstance. This is
not Conversation SEND, complete interrupted-delivery recovery or native YVEX control.

The code-owned [Application catalog](../application/yai-application/src/capabilities.rs)
remains the operation identity owner. Studio's
[consumer evidence map](src/contrib/settings/capabilityPosture.ts) classifies
63 connected interactions, four deliberately alternate paths and five named
interaction debts. `tests/studio/capability-parity.mjs` compares every ID against
the executable catalog. This inventory does not replace per-action positive,
refusal/stale and lost-acknowledgement tests. A newly advertised Host operation
stays unqualified until reconciled; a running Host may predate the checkout.

Action-level coverage is mandatory: a summary projection does not qualify
search, historical reconstruction, mutation or execution for that family. Each
interaction needs a typed client, authored UI and positive/refusal evidence.
Submitted work reconnects through its durable domain identity; IPC failure never
triggers implicit resubmission. Preserve local dirty buffers during resync.

Every wave starts with Git/remote and Backend Sync, a Capability Delta Check,
and a Product Quality Pass plan. Preserve independently owned work; only
published executable contracts become Studio contracts. Every materially changed
capability must retain its existing exact catalog entry or update that owner and
regenerate its human matrix. Never infer authority from discoverability.

## Capability board

The rows are Studio-local product properties, not new backend semantic owners.
Evidence paths refer to `tests/studio/` unless otherwise qualified.

| ID | Status | Capability / executable truth | Next boundary / dependency | Evidence |
|---|---|---|---|---|
| SW01 | 🟢 ESTABLISHED | One contribution-driven Workbench for live and fixture data | Feature-local quality passes | kernel.test.cjs; workbench.mjs |
| SW02 | 🟢 ESTABLISHED | Universal Surface registry, roles, preview/pin/close and singleton Settings | Multiple visible groups later | kernel.test.cjs; workbench.mjs |
| SW03 | 🟢 ESTABLISHED | Back/Forward across perspectives, objects, Surface activation and Settings; new navigation invalidates Forward | Preserve Case-local lifetime | navigation-journal.mjs |
| SW04 | 🟢 ESTABLISHED | Command Palette, projected-object Quick Open, renderer-owned Find, top command center | Semantic Case Search needs an owner | workbench.mjs; navigation-journal.mjs |
| SW05 | 🟢 ESTABLISHED | Reversible Surface focus, region resizing, window-local draft/tab/navigation retention | Shared/persisted Case Views later | focus-layout.mjs; reliability.mjs |
| SF01 | 🟢 ESTABLISHED | Exact material identity/bytes fencing through resident Host | No direct filesystem read | material-lifecycle.mjs |
| SF02 | 🟢 ESTABLISHED | Lazy CodeMirror, syntax modes, find/replace, dirty/revert, Open With | Language servers not selected | editing.mjs; workbench.mjs |
| SF03 | 🟢 ESTABLISHED | Markdown, inert SVG/images, PDF, table, audio/video and unknown fallback | Rich editing not implied | media-surfaces.mjs; desktop-csp.mjs |
| SF04 | 🔴 OPEN | Governed Save and stale-revision write refusal | Participant-origin content mutation/admission/receipt contract | Save remains disabled |
| SE01 | 🟢 ESTABLISHED | Distinct File/Source/Resource identity; projected path tree and type-specific exploration | No frontend filesystem scan | environment-actions.mjs; workbench.mjs |
| SE02 | 🟡 PARTIAL | Source declare/acquire/resume/revoke and explicit policy-source publication with exact attempt recovery | Interrupted in-flight attempts can remain unresolved; no automatic redispatch | environment-actions.mjs; source-policy-actions.mjs; execution-actions.mjs |
| SE03 | 🟡 PARTIAL | Resource kind/bounds/requestability, process attachment and seven authored request variants with receipt observation | General native binding setup, content admission and catalog-qualified MCP calls remain interaction debt | source-policy-actions.mjs; effect-actions.mjs |
| SE04 | 🔴 OPEN | Governed mixed-folder intake and routing preview | Authorized recursive intake/material acquisition plus typed route projection; no extension classifier | Policy-only file upload is not folder intake |
| SK01 | 🟢 ESTABLISHED | Bounded Knowledge collections plus owner inspect/search/resolve/documentary navigation | Lexical relevance is not truth; exact backing rechecked | knowledge-navigation.mjs; execution-actions.mjs |
| SK02 | 🟡 PARTIAL | Full-canvas associative graph, pan/zoom/drag, filters, neighborhoods, edges and unresolved refs | Backend projection paging and denser graph strategy remain | knowledge-navigation.mjs |
| SM01 | 🟡 PARTIAL | Case Timeline and temporal Experience Graph from qualified relations | Typed pagination and richer temporal navigation | knowledge-navigation.mjs; operational-live.mjs |
| SM02 | 🟢 ESTABLISHED | Authored Recall task/cut/required refs; evidence, selection reasons, closure and limitations | Relevance is not confidence | memory-actions.mjs |
| SM03 | 🟢 ESTABLISHED | Working State compile/refresh, explicit W4 paging, current control and stale posture | No implicit paging or authority promotion | memory-actions.mjs |
| SM04 | 🟡 PARTIAL | Decision Frontier and exact request preparation from qualified W | Ambient assessment needs real consumer lineage; producer execution remains separate | memory-actions.mjs |
| SM05 | ⚪ LATER | Optional technical Memory backing inspection | No invented DuckDB or parallel store product | Current owners remain Transition/derived views |
| SA01 | 🟢 ESTABLISHED | Explicit policy document ingest, typed IR, validate/publish/retire/revoke and Case bind/replace/unbind | Compilers own interpretation; publication is not binding | policy-intake.mjs; policy-actions.mjs |
| SA02 | 🟢 ESTABLISHED | Review approve/deny/defer and projected Grants/last Decision | Backend enforces current authority | application-actions.mjs |
| SA03 | 🟡 PARTIAL | Policy Sources routed to Authority; documentary portions retain provenance | Policy catalog/read, mixed-region route projection, effective combined-policy explanation and simulation open | source-policy-actions.mjs |
| SO01 | 🟡 PARTIAL | Work center exposes Workflow, Handoff, bounded run/stop, exact execution observation and history | No complete execution catalog, Handoff inbox or general result-body browser | work-actions.mjs; effect-actions.mjs |
| SO02 | 🟡 PARTIAL | Define/bind human checkpoint workflows, HumanInput, checkpoint patch propose/adopt | Other node authors, effective patched topology/prompt projection open | work-actions.mjs |
| SO03 | 🟡 PARTIAL | Explicit Handoff offer/accept/decline/result/reconcile | Authored exact refs work; inbox/read projection and richer result navigation open | work-actions.mjs |
| SO04 | 🟢 ESTABLISHED | Journal from committed history, follow/pause/search/type/component, Inspector/Timeline | Latest 160 disclosure; not a complete ledger browser | navigation-journal.mjs; operational-live.mjs |
| SC01 | 🟡 PARTIAL | Committed Conversation read and local composer draft | Published reconnect-safe SEND/execution owner required | attachment-lifecycle.mjs |
| SP01 | 🟢 ESTABLISHED | Authored generic target register, measured evidence import, trust and explicit Case binding | Full model/runtime catalog not implied | compute-actions.mjs |
| SP02 | 🟡 PARTIAL | Models, provider adapter, deployment, qualification, trust and health kept distinct | Tenant target discovery and typed probe operation absent | compute-actions.mjs |
| SP03 | 🟡 PARTIAL | Cognitive preparation distinguished from provider configuration | Semantic evidence authoring/discovery and exact requirement preparation remain UI/Application composition debt | Explicit Compute posture |
| SY01 | 🟡 PARTIAL | Generic OpenAI-compatible target with optional yvex.http.v1 telemetry posture in Compute | Real DeepSeek deployment unavailable in current external lane | Controlled provider test is not external model evidence |
| SY02 | ⚪ LATER | Native YVEX Source/Artifact/Profile/Engine/Session management | Versioned YVEX public management plane | No private producer coupling |
| SH01 | 🟢 ESTABLISHED | Resident same-user Unix Host, singleton discovery, events, auto-attach, telemetry and lifecycle | Host survives Studio; normal live mode never embeds Application | application/yai-host tests; existing native acceptance |
| SH02 | 🟡 PARTIAL | Host supervises the existing RuntimeInstance; exact run/stop and reconnect observation | S1 lifecycle depth and ambiguous-delivery recovery remain open | effect-actions.mjs; application-execution-lifecycle characterization |
| SI01 | 🟡 PARTIAL | Common typed Inspector for projected objects, graph edges and Journal events | Handoff inbox, execution result bodies and model catalog details require read projections | knowledge-navigation.mjs; navigation-journal.mjs |
| SI02 | 🟢 ESTABLISHED | Identity/Manage Activity footer, real Principal/Tenant/Participant, local identity bootstrap | Other-Principal enrollment and membership picker remain open | identity-actions.mjs; navigation-journal.mjs |
| SI03 | 🟢 ESTABLISHED | Singleton searchable preferences; Provider/YVEX management points to Compute | Settings is not another management owner | workbench.mjs; navigation-journal.mjs |
| SD01 | 🟢 ESTABLISHED | Native PTY, compact shared panel toolbar and conditional resizable shell list | PTY remains desktop capability, not Case authority | native desktop acceptance; terminal Rust tests |
| SD02 | 🟡 PARTIAL | Linux native window, drag/resize, status bar and strict production CSP | Other desktop platforms require native qualification | desktop-csp.mjs; reliability.mjs |
| SQ01 | 🟢 ESTABLISHED | Persistent real qualification world, explicit convergent enrichment and two-client Host event resync | Extend by normal product operations; never reset operator Case | operational-live.mjs; operational_world.py |
| SQ02 | 🟢 ESTABLISHED | Shared geometry/tokens, lazy heavy renderers, four-size visual matrix and per-action regressions | Permanent proportional Product Quality Pass | browser suites and production build |
| SQ03 | 🟡 PARTIAL | Automated local proof and Golden local | Human Golden and external producer evidence remain independent | HUMAN_GOLDEN_CASE = PENDING_OPERATOR |
| SQ04 | ⚪ LATER | Remote/mobile, public plugins, Computer Use and shared declarative Case Views | Explicit transport, authority and product programs | No accidental local TCP API |

## Surface / owner / parity matrix

| Surface | Canonical or derived YAI owner | Application / CLI | Studio posture and exact remaining boundary |
|---|---|---|---|
| Environment | Sources, Resources, retained revision resolver | Typed read/declare/acquire/resume/publish/revoke and governed request; CLI shares owners | Exact attempts/receipts, seven request variants, process attach; general secure binding and content-admission authors remain open |
| Knowledge | M07 source-grounded derivation | Authorized summary/inspect/search/resolve/navigation; CLI inspect/build/graph | Owner-backed lexical queries, collections and associative graph; no frontend canonical index |
| Memory | Transition, Recall, Semantic Working State | Recall/compile/page/refresh/frontier/request typed; CLI owner surfaces | Authored forms and results; ambient consumer lineage missing in inspection |
| Authority | Policy compiler/IR, lifecycle, EffectivePolicy, Review/Grant | Typed lifecycle/binding/reviews; CLI catalogs/routes | Typed candidate rules and active bindings; catalog read and combined-policy explanation need projection |
| Work | Workflow, Handoff, execution owners | Typed workflow/handoff/run/stop/execution observation; CLI shares owners | Authored forms and exact window-retained receipt references; Handoff inbox and complete execution discovery missing |
| Compute | Provider governance and cognitive binding owners | Register/qualify/trust/bind typed; CLI probe/semantic qualification | Explicit target setup; probe, semantic evidence and catalog composition remain partial |
| Conversation | Committed Turns and execution controller | Read summary; reconnect-safe SEND not in baseline catalog | Read-only committed history; local draft never becomes a fake Turn |
| Journal / Activity | Committed Transition history | Authorized latest-160 timeline | Journal filter/follow, latest-20 Activity; missing typed pagination |
| Host / Settings | Rust Host / Studio-local preference owner | Real Host IPC telemetry/control; CLI lifecycle | Local preferences separate from Case; actual scheduler supervision reported by Host |

Each missing UI fact must be classified as **backend capability gap**,
**Application boundary gap**, **Studio interaction debt**, **authority/contract
refusal**, or **deferred product program**. Raw Rust input serializability alone
does not prove a usable operation. No UI parses CLI output, reads LMDB, evaluates
policy, dispatches provider HTTP or invents canonical objects.

## Release progression and program ownership

S0 foundation, S2 universal Workbench/productization and the bounded S3 desktop
foothold are established. Product Case coherence and Environment authoring are
established within their read/local-edit boundaries. S1 resident Application
Host and bounded existing-scheduler supervision are established; **S1 as a whole
remains PARTIAL** for the catalogued lifecycle/recovery boundaries. `yaid` is separate compatibility machinery, not this
Host. The current operationalization/refoundation advances S4/S5/S6/S7 without
promoting them wholesale. S8 native YVEX, S9 broader desktop/platform work and
S10–S12 external/Computer/remote-mobile remain separate selections.

## Persistent qualification world

`case:studio-live-qualification` remains operator-owned. The operational wave
advances generation 64 → 103 through normal CLI product operations, then
explicit controlled HTTP observations reach 108, 113 and 118. Two independently
attached LiveClients observe the real resident Host events and resync at
108 → 113, then 113 → 118 after the published execution-Host upgrade. Journal
pause holds the previous cut; resume follows generation 118. This browser bridge
evidence is not a claim of two native processes. Native release WebKitGTK/PTY
acceptance separately reopens the same generation without mutating the Case.
The original six Sources, repository Resource and completed Workflow survive.
The retained world has seven Resources, nine Sources and two policy bindings:
repository/policy discovery, filesystem, SQLite database, HTTP, process runner
and read-only MCP. HTTP/MCP qualification peers are temporary, explicitly started;
retained observations do not imply current health.

`tests/qualification/studio-product-vertical/operational_world.py` supports
inspect/advance/serve with explicit profile and input root. A second advance
reuses Resource/Source identities, revisions and request identities; it does not
duplicate effects. Automated Workflow/Handoff/Review/provider tests use separate
temporary real Cases. The operator Case has no fabricated provider or Turn.
DeepSeek qualification is an independent deployment lane; another model is not
silently substituted. The operator-authorized V4 Flash load attempt was refused
by the producer (`model is not launchable`): its present MXFP4 representation
reports a malformed runtime binding. This is a deployment limitation, not a
successful inference test or a Studio native-management capability. Golden and
continuity canary state are never test setup.

## Permanent product quality and acceptance

Every Studio milestone includes a **Product Quality Pass** for materially touched
regions: ownership, hierarchy, typography, spacing, icons, semantic color,
focus/hover/selection, borders, keyboard, empty/error/stale states, responsive
geometry, duplicated controls and proportionate visual regression. Compile/test
success alone is insufficient when visible regressions remain. This authorizes
no unrelated redesign and creates no subjective root maturity promotion.

Qualify 1600×960, 1440×900, 1280×800 and 1000×650 where practical. Heavy editor,
Markdown, terminal, PDF, graph and operational Surfaces remain lazy. Report build
chunks, not benchmark claims. Every primary perspective evolves from passive
reporting toward operational interaction **as typed YAI capabilities qualify**.
Policy carriers remain provenance; only qualified typed rules enter authority.
An LLM may explain qualified material later, but never activate policy or supply
permission. Canonical refs remain available behind human presentation labels.

The cumulative runbook names what the operator should inspect after every wave.
Human acceptance stays `PENDING_OPERATOR` until an explicit result at the relevant
SHA. Root X03/X04 remains unchanged by operation counts or this Studio board.
