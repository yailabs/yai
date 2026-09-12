# Current executable architecture

Authority: implementation truth. This edition covers the published W20
foundation, the closed I01 multipart-conversation interlock, its
provider-independent interaction-host closure, the I02 cognitive-binding
planning interlock, the I03 typed provider-realization boundary, and the I04
bounded cognitive-composition boundary, followed by I05 governed cognitive
target arbitration and I06 conversation-host cognitive routing. R5 subsequently
removed the obsolete vendored editor without changing cognitive semantics or
the then-qualified REPLAI pin. Golden subsequently converged governed resource
access and the Case workbench; guided setup, public model discovery, single
connection and separate model/system presentation extend that product consumer.
The later qualified REPLAI repin is preserved. Historical checkpoints and exact
captured evidence remain available through the immutable Git links below;
current executable proof stays with the tests. Historical dossiers are not a
second architecture tree in the working repository.

This document includes current contradictions. It does not claim that the
[Constitution](constitution.md) is implemented. Target changes and sequencing
belong only in the [Roadmap](../ROADMAP.md).

The [semantic cognitive-state target](semantic-state-execution-target.md) is
deliberately separate. The [source refoundation](https://github.com/yailabs/yai/blob/c187648e9d9909d4d9b6711f131fa726d48cd585/refoundation/validation/semantic-state-refoundation/REPORT.md)
implements bounded replay-qualified S, scoped W and derived semantic deltas with
full-recompilation fallback. General task sufficiency, a public model-state frame,
optimized incremental compilation and semantic demand paging remain open.
State Read and State Update are OPEN target capabilities, not current execution
contracts. Bounded locality evidence is not universal Case-age independence or
arbitrary target-native state lowering. Context-only preparation is the current compatibility path, not the
target definition of memory. "State Fabric" names an architectural composition,
not a source owner. Live maturity and selection belong only to the Roadmap.

## Executable summary

YAI has two unequal product processes:

```text
operator
  |
  +-- yai (Rust command/process boundary)
  |     +-- semantic-state composition / bounded W compilation / context lowering
  |     +-- Case-canonical ordered multipart Turns and immutable content ownership
  |     +-- provenance-bound operational-memory derivation/retrieval
  |     +-- derived semantic ResidencyPlan and bounded Case execution loop
  |     +-- controlled filesystem effect transition family
  |     +-- Case-native typed human review on the controlled carrier
  |     +-- durable single-Case runtime admission metadata
  |     +-- deterministic governance intake and immutable policy artifacts
  |     +-- local POSIX-authenticated Principal and Tenant security isolation
  |     +-- Tenant-scoped provider targets, qualification, trust and shared health
  |     +-- Case-canonical provider selection and delivery-safe attempt lineage
  |     +-- Case-canonical cognitive bindings and execution-free lane planning
  |     +-- exact typed provider realization and provenance-bound auxiliary output
  |     +-- bounded direct-or-prerequisite cognitive execution composition
  |     +-- canonical Transition/CaseState authority in LMDB
  |     +-- legacy journal inspect/import/replay compatibility
  |     +-- LMDB graph materialization/query
  |     +-- DuckDB analytical derivation
  |     +-- yai-engine Rust library
  |     +-- selected yaid Unix-socket requests
  |
  +-- yaid (C daemon)
        +-- Unix-socket status/info/shutdown
        +-- prepared minimum/filesystem fixture loops
        +-- C JSONL records/journal and projections used by those loops
        +-- restartable hot-state snapshot
```

[`cmd/yai/src/main.rs`](../cmd/yai/src/main.rs) is a small process bootstrap.
The compiled command registry, centralized parser, help and output boundary are
owned by [`cmd/yai/src/cli/`](../cmd/yai/src/cli/); stable command adapters are
isolated in
[`cmd/yai/src/command_adapters.rs`](../cmd/yai/src/command_adapters.rs). The
domain implementation remains grouped by demonstrated boundary in
[`provider.rs`](../cmd/yai/src/provider.rs),
[`case_runtime.rs`](../cmd/yai/src/case_runtime.rs),
[`policy.rs`](../cmd/yai/src/policy.rs),
[`security.rs`](../cmd/yai/src/security.rs),
[`conversation_controller.rs`](../cmd/yai/src/conversation_controller.rs),
[`conversation_cli.rs`](../cmd/yai/src/conversation_cli.rs),
[`cognitive_cli.rs`](../cmd/yai/src/cognitive_cli.rs),
[`cognitive_execution.rs`](../cmd/yai/src/cognitive_execution.rs),
[`memory_cli.rs`](../cmd/yai/src/memory_cli.rs),
[`review.rs`](../cmd/yai/src/review.rs),
[`controlled_effect.rs`](../cmd/yai/src/controlled_effect.rs),
[`filesystem.rs`](../cmd/yai/src/filesystem.rs),
[`replay.rs`](../cmd/yai/src/replay.rs),
[`graph_runtime.rs`](../cmd/yai/src/graph_runtime.rs), and
[`analytics.rs`](../cmd/yai/src/analytics.rs). These modules do not imply
future subsystems; they isolate existing behavior for the next semantic
refoundation.

Provider administration adapters are isolated in
[`provider_governance_cli.rs`](../cmd/yai/src/provider_governance_cli.rs), while
typed owner contracts and deterministic selection live in
[`provider_governance.rs`](../engine/yai-engine/src/provider_governance.rs).
See the current [provider governance contract](provider-governance.md).
Provider-independent cognitive capability, suitability, binding, lane and
plan contracts live in
[`cognitive.rs`](../engine/yai-engine/src/cognitive.rs). I03 keeps those plans
pure, then revalidates a fresh plan against exact mechanical adapter evidence
before the existing governed invocation/result seam performs one dispatch.
I04 composes at most one explicit prerequisite into a fresh primary
realization. Requests and source closures are content-addressed values. I06
canonically adopts a request as conversation execution intent; Advanced I04
requests can still be ephemeral. Source closure and execution plans remain
derived. No layer owns deployment/runtime state.

I05 extends the same Case cognitive binding, not ProviderSelection, with an
optional ordered policy (at most eight exact target/evidence candidates).
Existing v1 bindings remain pinned. The v2 policy's `target_id` and associated
digest/evidence identify its first preference; `target_policy` explicitly lists
ordered alternatives. They do not name an already selected execution target.
Only a fresh `CognitiveExecutionPlan v2` does that. Planning records every
candidate's exclusions and an integrity-bound governance/evidence snapshot.
Known source shapes are qualified before choice; unknown shape remains deferred
until I03. First eligible preference wins, without quality scores, locality
guesses, or health rankings. Unavailable/open-circuit targets are excluded;
health never rewrites semantic evidence or policy.

I03 rechecks the arbitration snapshot before exact selection and again at
invocation admission. A changed snapshot requires a new plan, not transport
substitution. Arbitrated lanes bind both the canonical policy and exact chosen
target/evidence. I04 uses shape-aware planning for both direct and prerequisite
paths and preserves its bounded source-closure/recovery algorithm. Any unresolved
prior delivery for the same semantic source requirement blocks cross-target
redispatch. No arbitration database, provider session, runtime owner or automatic
retry controller is introduced.

I06 makes ConversationController the application consumer of that stack.
Normal SEND binds PrimaryConversation because the action is conversational;
media never implies SpeechToText or ImageUnderstanding. An optional explicit
prerequisite names exact ordered source parts. The immutable intent is committed
with the Turn in one LMDB transaction, after complete content-object publication.
An older/plumbing Turn adopts its first intent at explicit host execution.
Retry cannot replace it, duplicate SEND or hide an unresolved invocation under a
new source closure. Compatible recorded results/derivations are reused through
I03/I04; changed governance triggers fresh planning or refusal. The controller's
former direct provider-order/failover loop is removed. REPLAI supplies editing
events only and remains pinned unchanged. Cancellation checks gate future
stages; no transport abort is claimed. No new owner/database or CaseState field
is introduced. See [I06](https://github.com/yailabs/yai/blob/c187648e9d9909d4d9b6711f131fa726d48cd585/refoundation/foundation-recovery/interlock-06/REPORT.md).

The normal CLI links [`yai-engine`](../engine/yai-engine/src/lib.rs) directly
as Rust. There is no product C→Rust call edge or installed Rust C ABI. The
former marker FFI crate and smoke bridge were removed.

The production C archive contains 16 sources. `yaid` adds only its entrypoint,
IPC, and core loop. Component-only C mechanics build into a separate
characterization archive and are linked only by their tests. Static archive
membership is no longer used to make those components look product-reachable.

## Current application/client seams and limit

`./yai` is YAI's handwritten native product CLI. Command metadata, parsing,
help, guided interaction and output projection live in YAI, not in an external
interface compiler. The current seams are useful but do not constitute a
complete, stable frontend-independent Application API:

- [`cli/product.rs`](../cmd/yai/src/cli/product.rs) maps parsed invocations to
  owner queries or command adapters. Some results are typed CLI views or native
  JSON; other paths still pass argument vectors and capture handler stdout.
  `execute_structured_legacy` can turn printed fields into presentation output.
  That compatibility behavior is not an application contract for other clients.
- [`ConversationController`](../cmd/yai/src/conversation_controller.rs) exposes
  typed actions, submission/execution results and application-event facts within
  the CLI crate. Its `pub(super)` surface is not an exported client API; events
  returned in a result are not a public progress/event stream. Some inspections
  return `serde_json::Value`, and the controller opens the authorized store.
- Source acquisition in [`controlled_effect/source.rs`](../cmd/yai/src/controlled_effect/source.rs)
  composes existing resource, policy and history owners, but its command seam
  still takes CLI arguments and returns JSON. Policy, graph and Workflow
  handlers likewise mix command adaptation and orchestration. The shared typed
  [`review action`](../cmd/yai/src/review.rs) reuses current authority checks,
  yet still takes an LMDB store and prints outcomes. These are hardening gaps,
  not independent semantic authorities or reasons to duplicate their behavior.
- The [`yai-engine` crate exports](../engine/yai-engine/src/lib.rs) include
  domain contracts and persistence modules. Its authorized historical,
  experience and Recall readers return qualified typed values through
  [`LmdbRecordStore`](../engine/yai-engine/src/store/lmdb.rs). A consumer still
  needs store/content setup; public Rust visibility alone does not establish a
  persistence-independent application facade or supported remote SDK.

The [application/client target](../ROADMAP.md#application-and-client-boundary--adopted-target)
requires native CLI and future native Studio to consume one typed YAI application
boundary. Current CLI/store coupling is recorded above, not refactored by that
decision. No stable public Application API, exported interface package,
interfaces integration, generated official SDK or Studio implementation is
claimed. Current builds have no interfaces dependency. The roadmap owns future
export/client qualification; frontends must not acquire domain authority by
presenting these values.

## Demonstrated product verticals

“Complete” means that the bounded path has an entrypoint, consequence or
durable residue, operator output, and executable regression proof. It does not
mean constitutional, general, or production-ready.

| Vertical | Current path and demonstrated consequence | First architectural gap |
|---|---|---|
| Case conversation content | mutable non-canonical draft → bounded text/media imports → explicit original/derived provenance → SEND → `ConversationTurnCommitted` → immutable content-addressed bytes plus canonical ordered references; I03 can publish bounded provider-derived text as a separate canonical relation without mutating the Turn | non-text generated output and automatic unbounded composition remain later work |
| Conversation interaction host | host-normalized ordered parts + explicit semantic prerequisites → atomic Turn/intent SEND → shared I05/I02/I03 direct realization or I04 composition; failure and retry preserve one Turn/intent without ResourceAttachment, Workflow, Policy, Effect, or Case-runtime admission | the Advanced `yai prompt` frontend consumes native REPLAI; generic terminal mechanics stay external |
| Cognitive capability planning | explicit requirement → pinned or ordered Case/Participant cognitive binding → exact semantic, governance and known mechanical eligibility → first eligible candidate with inspectable exclusions → exact native/derived/unresolved plan and lane; planning remains execution-free | learned/economic routing remains absent; I06 connects the conversation host |
| Typed provider realization | fresh I02 plan + exact current binding/evidence/envelope + ProviderQualification v5 wire-shape evidence + canonical Turn parts → exact-target governed selection → ordered OpenAI-compatible typed request → ProviderInvocation/ProviderResult; derived routes preserve immutable text/source/result provenance; native function calls and JSON-object output have separate mechanical probes | production STT/vision adapters, public YVEX typed-media compatibility and streaming remain later work |
| Cognitive execution composition | explicit primary goal + ordered canonical source selection → content-addressed composition request → proven direct primary bypass or one exact auxiliary I02/I03 realization → canonical derived content → deterministic original/derived source closure → fresh exact primary I02/I03 realization; I05 arbitrates before each exact plan, and compatible prerequisites resume without redispatch | only depth-two speech/image-to-text prerequisites are admitted; recursive graphs remain absent; I06 consumes this bounded composition in the host |
| Case-bound provider prompt | replay-qualified SemanticState + current EffectivePolicy → bounded scoped W → Projection/ContextFrame v10 → exact provider render → Invocation/Result lineage; existing residency report describes W selection, not a second compiler | context compatibility only, not persistent model state; optional public target preflight belongs to execution admission, not W; interactive conversation uses cognitive realization; --once/piped legacy diagnostics remain separate |
| Governed provider routing | immutable Tenant ProviderTarget → synthetic evidence-bound qualification → Tenant-Owner approval → shared fresh health/circuit → exact Case provider binding → mechanical requirement/filtering → canonical ProviderSelection and attempt outcome; local fixtures prove qualified capability differences, deterministic exclusions, pre-dispatch safe failover and indeterminate-delivery refusal | H18 adds HTTPS/credential/circuit hardening; real provider capacity and full external Golden qualification remain separate evidence requirements |
| Agentless Case runtime | authenticated Tenant owner starts a disposable bounded runner which reloads CaseState → reconciles effects/review → gates on normative readiness and temporal validity → repairs memory → invokes provider → normalizes/admits/effects → repeats from canonical reality; one admitted runner per Case is executable | this bounded runner is not a universal capability loop; the separate single-host multi-Case scheduler is implemented, not a distributed lease/consensus system |
| Controlled external effect | Tenant-scoped attachment + Ready/Valid EffectivePolicy → exact Operation → DecisionBasis/Decision/finite ExecutionGrant → durable fenced PREPARE → filesystem replacement, confined process or admitted MCP tool call → Observation/Receipt → FINALIZE/INDETERMINATE | process confinement is bounded Linux x86_64; arbitrary shell, general database mutation and state-changing HTTP are not admitted |
| Human-reviewed filesystem effect | policy-driven `REQUIRE_REVIEW` → v2 request → per-command POSIX Principal authentication → Tenant membership → explicit Principal/Participant link → Case-role eligibility → ReviewAction v2 → effective Decision → same Grant/carrier path | local POSIX identity only; no SSO, remote signer or membership removal lifecycle |
| Governance intake and admission | authenticated Tenant owner + JSON/explicit Markdown or text-PDF policy block → Tenant-owned immutable artifact/lifecycle → exact Tenant-safe Case binding → EffectivePolicy → operation-specific DecisionBasis and derived cognitive rules; P@1/P@2 remain distinct and validity/revoke contract future authority | unresolved prose never publishes; local ownership is enforced; external organization identity, credential security, retention and distributed revoke remain future work |
| Journal compatibility | inspect/dry-run/import `yai.store.record.v0` or `yai.record.v1`, preserving unknowns opaquely in an isolated target; old replay still materializes legacy record indexes | general semantic promotion is deliberately absent; the old record plane remains compatibility data, not authority |
| Graph / experience access | typed canonical transitions → replay-qualified historical evidence under current disclosure → exact-source experience relations and bounded directed paths; existing decoded legacy records → rebuildable RuntimeGraph remains a separate compatibility access path | general causal discovery and all-owner traversal remain absent; bounded D/H/S Recall is described below; legacy-only cases still depend on compatibility translation |
| Analytical facts | LMDB operational records → DuckDB extraction → reports | four declared families have no extractor; schema/orchestration remains embedded in the command crate |

`yaid` startup, status/info/shutdown, restart, fixture loops, and hot snapshot
reconstruction remain covered separately. The former direct `carrier
fs-write` product command is removed. Its read-only counterpart remains an
observation compatibility command. The C daemon filesystem loop now emits
explicit descriptor/no-effect fixture residue and no longer performs or claims
a product filesystem mutation.

Partial and component paths must not be promoted into product claims:

- `yai process observe` probes a PID but persists no Observation; CLI process
  signaling does not dispatch.
- the C process carrier observes, signals, and re-observes a test-owned child;
  it is a characterized platform boundary, not a product vertical.
- the C filesystem carrier preserves pre-state, real write, post-state hash,
  and receipt mechanics under component tests only.
- `yaid` filesystem fixtures create an input fixture and descriptor/no-effect
  records; they are case/journal test setup, not product effect evidence.

The provider planning registry, CaseHandle/CapabilityLease inspection views,
generic carrier registries, synthetic dispatch families, C graph/index/memory
mirrors, `net/`, and its unconsumed `proto/` fixtures are absent. None had a
product caller or unique property not already captured by the surviving
verticals/tests.

## Case workbench and attached capabilities

`./yai open [CASE]` is the short Product frontend over ConversationController and
the qualified REPLAI editor. Guided `init`/`open` compose existing security and
Case contracts with explicit confirmations, not ambient global context. The
bounded participant bootstrap commits existing typed transitions atomically;
it does not grant policy, resources or provider trust. Existing explicit
`case workbench` and one-shot commands remain supported. `/connect`, `/review`,
`/attach`, `/policy publish`, `/admit` and `/workflow bind` ask for missing input;
bare `/retry` resolves a canonical current-thread Turn. The frontend calls typed
application actions without subprocess dispatch or new terminal mechanics.
Guided connection first discovers a bounded public catalog through the existing
provider/controller seam, without target registration or Case data disclosure.
A singleton supplies the exact model; multiple entries require selection. No
catalog result implies semantic or mechanical suitability or physical load state.
Literal locality is inferred by the existing address rules; ambiguous names and
authentication challenges produce conditional questions. One generation-scoped
confirmation admits probes, trust and an operator attestation whose provenance
reference is generated. Existing primaries require explicit replacement there.
Catalog disappearance or stale Case approval refuses without inference/binding.
No catalog registry, model alias, runtime owner or provider-brand branch appears.
`/connect` is the single guided action; no connection-profile selector exists in
the frontend or controller. Its existing synthetic qualification probes text,
native functions and JSON independently. Text is required to connect; unsupported
optional shapes and their failures remain inspectable. Exact execution, not a
connection label, enforces each required shape. Partial mechanical evidence
never authorizes a missing shape. Buffered inference has a separate
bounded response deadline (default 300s; `YAI_PROVIDER_RESPONSE_TIMEOUT_SECS`,
1–3600), while connection, metadata GET and ordinary resource budgets remain 30s. Expiry after
submission is delivery-indeterminate, never automatic retry. Probe progress and
bounded error categories are visible; no model error prose or secret is admitted
as qualification evidence. All-text input is lowered as distinct ordered string
messages, including repeated equal parts; media retains ordered content arrays.
The same adapter lowering is exercised by the version-2 synthetic shape suite.
Canonical Turn/closure IDs and provenance are unchanged; wire messages are not
a second conversation history. This is not a claim of external media support.
Normal conversation presentation separates buffered model replies from YAI
notices and qualification progress. It emits no automatic execution JSON/IDs;
explicit `/details` renders the last action's unchanged result after current
Case access checks. This disposable display snapshot is not history, a new
semantic owner or a provider response. Canonical inspection remains unchanged.
Ordinary conversation remains I06. Explicit `/work` commits finite work intent before
I02–I05 arbitration/realization; Workflow CaseWork nodes call that same host.
No Agent, connector/tool registry or independent execution history is introduced.

The derived Case capability view is scoped by generation, Participant, current
resource envelope and READY EffectivePolicy. Native provider function requests
are candidates: exact normalization creates an Operation, not permission.
Arbitrary assistant prose cannot become a hidden tool call. Human review and
current authority remain mandatory when required. Delegation never transfers
the human Principal's review authority to the model.

Resource effects reuse Decision/Grant and fenced PREPARE. Filesystem writes
retain their existing atomic carrier. A separately qualified process runner
executes only descriptor-pinned root-owned ELF, exact argv/cwd/environment and
bounded output/time under Linux x86_64 Landlock/seccomp confinement. MCP tools
are conservatively effects; indeterminate delivery holds the fence and is not
blind-retried. Database mutation is explicitly denied in the reference deck;
arbitrary SQL mutations, unrestricted shells and state-changing HTTP are absent.

Read observations remain scoped external material. Filesystem access is
descriptor-relative beneath admitted roots; SQLite queries use a bounded
quiescent rollback-mode image with a read-only authorizer and exact named query;
HTTP fetch uses exact endpoint/IP/path bounds without redirects. MCP implements
the bounded 2026-07-28 stateless Streamable HTTP subset: per-request capability
metadata, server/discover, tools/list, resources/list, resources/read and
tools/call, bounded pagination/JSON/SSE and exact catalog/schema revalidation.
It does not implement legacy session fallback, sampling, Apps, Tasks or prompts.
Remote catalog annotations never grant YAI authority.

Discovery produces integrity-bound candidates only. Explicit policy admission
imports complete immutable bytes into ConversationContentStore and records
CaseContentAdmitted identity/provenance/disclosure. Candidate drift refuses.
The original Turn is unchanged and material does not become semantic truth.
Model capability feedback and restart reuse are reconstructed from canonical
ProviderResult/Operation/Observation/effect/admission relations, not a second log.

Workbench inspection includes exact operations before review, policy, resources,
effects, Workflow, history, derived memory/graph and replay. `/rebuild` rebuilds
disposable views while verifying unchanged canonical history. Scoped Handoff
actions reuse existing source/target admission and never copy authority. The
[reference evidence](https://github.com/yailabs/yai/blob/c187648e9d9909d4d9b6711f131fa726d48cd585/refoundation/validation/golden-case-lifecycle/REPORT.md)
records that checkpoint's qualification; [Roadmap](../ROADMAP.md) owns the live
qualification posture. The [cumulative runbook](zero-to-current.md)
owns human acceptance. Loopback lifecycle proof is not live YVEX qualification.

## Current state and schema authority

### Canonical LMDB authority

Rust owns one canonical semantic write path in
[`transition.rs`](../engine/yai-engine/src/transition.rs) and
[`lmdb.rs`](../engine/yai-engine/src/store/lmdb.rs). The current serialized
contracts are `yai.transition.v18` and
`yai.case_state.v15`; Transition readers retain v1-v17 and reject unknown
future versions. Version 3 added provider identity,
semantic-frame/render lineage and typed
interaction turns. Version 4 adds Operation-bound ReviewRequest,
integrity-bound ReviewAction, effective Decision refs and resource review
posture. Version 5 adds exact Case PolicyBinding bind/replace/unbind payloads
and their compact current CaseState materialization. It does not make derived
EffectivePolicy or context canonical. Version 6 adds policy-bound authority,
version 7 temporal invalidation/cancellation/closure, and version 8 adds
immutable Case Tenant ownership, authenticated transition provenance and
Principal-to-Participant links. Versions 9 and 10 add shared ResourceControl
and exact Case-bound Workflow progression; version 11 adds amendments,
Subflow and Handoff; version 12 adds Case provider binding, selection and
attempt outcomes while keeping derived views non-canonical. Version 13 adds a
Case-canonical `ConversationTurnCommitted` payload whose ordered typed content
references are independent of provider invocation/result success. Version 14
adds explicit Case-canonical cognitive binding and
unbind history. CaseState v13 materializes only the current primary/auxiliary
binding slots; semantic suitability remains in the existing Tenant provider
governance owner, while execution plans and lanes remain derived. Version 15
adds a canonical post-SEND `ConversationDerivedContentRecorded` relation. Its
bytes remain owned by the existing immutable ConversationContentStore and it
does not add a CaseState field. ProviderQualification v4 adds exact bounded
wire-shape evidence from synthetic adapter requests while retaining v1-v3
readers. I04 adds `yai.cognitive_composition_request.v1` and
`yai.cognitive_source_closure.v1` as rebuildable content-addressed values. They
add no Transition payload, CaseState field, database or owner; exact closure
and derived-content identities are attached as causal refs to the ordinary
ProviderSelection history. Version 16 admits cognitive binding v2 ordered policy;
CaseState v14 materializes that bounded policy in the existing binding slots.
Version 17 adds `ConversationExecutionIntentRecorded`, reusing the exact I04
composition-request v1 body. Host SEND atomically appends Turn and intent;
CaseState gains no field. The Turn-view CLI projection is v2 and includes the
optional adopted intent. Stored Turn v1 and content contracts remain unchanged.
Pinned binding v1, historical plans v1 and their source histories remain readable;
historical plans must be freshly planned before new execution. I06 itself did
not change Projection v7, ContextFrame v7, RetrievalSet v3 or ProviderQualification v4.
Golden convergence advances Projection/ContextFrame to v8 for scoped resource
observations, admitted immutable material and a typed capability-output contract;
ProviderQualification v5 separately qualifies native calls and JSON output.
RetrievalSet v3 and RenderedInput v7 remain unchanged. Transition v18 and
CaseState v15 carry bounded resource envelopes, observation/effect and immutable
material-admission meaning. Source bootstrap subsequently advances them to
Transition v19 / CaseState v16 for source declarations and acquisition progress;
previous histories remain readable. Composition request v2 separates Turn author from
an explicitly delegated cognitive Participant; v3 binds finite work budgets and
optional exact Workflow execution identity. Existing request versions do not
silently acquire work budgets. WorkflowDefinition v3 admits a finite CaseWork
node and exact terminal predicate; v1/v2 definitions and store markers remain
readable without rewriting immutable definitions.
One bounded
LMDB write transaction:

1. validates typed payload closure and global Transition identity;
2. compares the expected per-Case generation;
3. appends the immutable Transition by identity and zero-padded Case sequence;
4. reduces it into CaseState;
5. commits ledger and materialization together.

The ledger databases are `transitions_by_id` and
`case_transition_sequence`; `case_state` is the rebuildable materialization.
Per-Case sequence, not timestamp, determines reducer order. Duplicate
Transition IDs and stale generations fail deterministically. Reopening after a
commit exposes both history and state; an injected failure before commit
exposes neither. Rebuild verifies:

```text
materialized CaseState == replay(ordered canonical Transitions)
```

The reducer deliberately covers only current live fields: Case lifecycle and
generation, immutable Tenant domain, Principal links, participant
bindings/admitted views, one current provider/model attachment or governed
provider binding, historical provider selections/attempt outcomes, latest
provider invocation/result/interpretation lineage, typed
Operation-bound review state, typed resource attachments, latest Operation/Decision, Grant
lifecycle, and compact prepared/finalized/indeterminate effect refs. Full
content, Observations, and Receipts remain in immutable Transitions rather than
turning CaseState into an object bag.

### Immutable conversation content

[`conversation.rs`](../engine/yai-engine/src/conversation.rs) owns original
application bytes that cannot be reconstructed from the ledger. The Linux
store is `$YAI_HOME/conversation-content-v1`: private owner-controlled
directories, descriptor-relative `openat2` access, bounded pre-read sizes,
content/digest verification, fsynced files and directories, and atomic
publication of complete immutable object directories. Binary payloads never
enter Transition or CaseState. The canonical Turn instead carries bounded
object metadata, exact ordered part identity and provenance. A missing or
corrupt owned object makes inspection/execution fail closed; it cannot inject
Case truth.

Drafts are mutable application state and are Case-namespaced. Previewing a
draft computes stable identities without publishing objects. SEND first
publishes/verifies all immutable objects and then commits the Turn transition;
provider execution is a later causal action. A crash can leave an unreferenced
complete object, never a canonical Turn pointing at a partially published one.
Conversation content is not a ResourceAttachment and import is an explicit
local-operator action, not a model-usable filesystem capability.

[`conversation_controller.rs`](../cmd/yai/src/conversation_controller.rs)
owns the application-side interaction algorithm, not conversation truth. It
accepts ordered typed parts and controller actions without parsing terminal
text, can commit a Turn independently of execution, and invokes an already
committed typed Turn with its canonical execution intent through the shared
I05/I02 planner and I03/I04 realization/composition boundary. The same bounded
Projection, ContextFrame, ProviderSelection, ProviderInvocation and ProviderResult
owners remain in use. This conversation path has no implicit ResourceAttachment,
Workflow, Effect, Policy, or operational-runtime budget. Retry cites the same
canonical Turn rather than creating another one.

Thread identity remains a field of a committed `ConversationTurn`, not a new
Thread object or store. A controller-local new thread becomes durable only on
its first committed Turn; thread listing is derived from Turn history and an
unused empty identity may disappear on restart. The controller exposes
buffered completion/failure facts and conservative pre-dispatch cancellation.
It adds no provider streaming capability and does not claim interruption of an
already dispatched buffered request.

[The native REPLAI frontend](replai-terminal.md) supplies `yai prompt` with
terminal editing, history, paste, resize/redraw and scoped terminal ownership.
It is the sole native interactive editor; [R5](https://github.com/yailabs/yai/blob/c187648e9d9909d4d9b6711f131fa726d48cd585/refoundation/integration/replai-r5/REPORT.md)
removed the obsolete vendored linenoise, not the qualified REPLAI dependency.
YAI translates events into controller actions; transient edits never publish
content or commit a Turn. Submission commits ordered text before execution.
`case enter` retains its inspection/shell setup role. Noninteractive `--once`
and piped input retain the existing invocation paths. No additional Product
`yai chat`, natural path parser or graphical frontend is introduced here.

Machine-local absolute filesystem roots are stored in a separately versioned
`local_resource_bindings` LMDB database. They survive restart because the
carrier needs them, but they are neither portable Case identity nor historical
authority. Canonical Operations target a logical attachment plus normalized
relative path.

### Legacy compatibility

The C/Rust journal remains readable as `yai.store.record.v0`; LMDB legacy
record envelopes remain `yai.record.v1`. Rust still defines 35 legacy kinds and
C 32. [`compatibility.rs`](../engine/yai-engine/src/compatibility.rs) is the
only summary-token decoder. It classifies input as losslessly structurally
promoted, promoted with compatibility metadata, preserved opaque, or rejected
malformed. Unknown future kinds/schemas are retained without invented meaning.

`yai journal compatibility-inspect` and `compatibility-import` support inspect,
dry-run, and import into an explicitly isolated LMDB target. Compatibility
payloads never append canonical Transitions or CaseState. Old `journal replay`
and legacy record indexes survive for operator/data compatibility; they are no
longer historical authority. Provider compatibility commands still emit old
JSONL/record shapes where required. New review actions write only typed
canonical Transitions and never dual-write `control/review.jsonl`.

### Derived/cache stores

- LMDB graph relations are rebuilt from typed Transitions plus the explicit
  legacy compatibility decoder; graph failure cannot affect canonical commit.
- RuntimeGraph is per-command and ephemeral.
- DuckDB facts are rebuildable analytics; historical extraction is routed
  through compatibility fields.
- `operational_memory_by_id` and `operational_memory_case_index` store
  `yai.operational_memory.v1` entries plus a generation/derivation manifest.
  They are updated after canonical commit, may fail independently, and can be
  cleared and deterministically rebuilt from ordered Transitions. They are not
  part of canonical transaction success.
- bounded typed Projection, ContextFrame and rendered-input metadata are stored
  in the separate `semantic_context_artifacts` LMDB database for inspection;
  this database is droppable and is never read by replay or CaseState reduction.
  Full rendered provider input, token sequences, and continuation values are
  not persisted.
- `yai.residency_plan.v1` is stored in the same droppable artifact database for
  inspection. It is a pure derived selection decision, never CaseState or a
  precondition for replay.
- Rust `ProjectionSummary`, `MemorySummary`, query and reconcile summaries
  remain legacy compatibility views and are not provider input. The former
  `/memory propose` producer is retired and cannot append `MemoryCandidate`.
- the C hot-state JSON snapshot is a daemon restart cache and is not updated by
  independent Rust mutations.

No canonical reducer, graph relation, provider/review decision, memory
category, or fact field parses arbitrary `summary` text. Remaining summary
grammar exists only inside the named legacy compatibility boundary and in
presentation assertions.

## Current control and effect behavior

[`effect.rs`](../engine/yai-engine/src/effect.rs) owns the only
product-reachable `filesystem.write` contract and carrier mechanics.
[`controlled_effect.rs`](../cmd/yai/src/controlled_effect.rs) owns its bounded
product orchestration. The implemented path is:

```text
typed CaseState/resource attachment
→ real OpenAI-compatible invocation
→ non-authoritative ProviderResult
→ strict yai.operation_proposal.filesystem_write.v1 decoder
→ yai.operation.v1
→ Ready yai.effective_policy.v2
→ yai.decision_basis.v1 applicability/authority/evidence evaluation
→ yai.decision.v2 (ALLOW|DENY|REQUIRE_REVIEW)
→ optional integrity-bound yai.review_request.v2 participant action
→ policy-bound yai.execution_grant.v2
→ typed pre-observation
→ EffectPrepared Transition
→ rust.filesystem.atomic_replace.v1
→ typed post-observation + yai.effect_receipt.v1
→ EffectFinalized or EffectIndeterminate/EffectReconciled Transition
→ rebuilt typed consequence view
→ second real provider invocation
```

The proposal decoder rejects unknown fields/schema/operation, malformed or
natural-language-only output, wrong attachments, empty/oversized content,
absolute paths, dot components, and traversal. Normalization creates stable
Operation identity and content digest. Provider material cannot construct a
Decision, Grant, Receipt, or canonical resource identity. New live admission is
closed-world: no explicit applicable ALLOW is DENY. A policy ALLOW cannot widen
the attachment's normalized prefix or payload bound. The proposer must be
Case-bound and satisfy every applicable policy role requirement.

The immutable Grant binds Operation/Decision/DecisionBasis and EffectivePolicy
digests, exact Case binding/artifact refs, Case and participant, logical
attachment, target/content, generation, idempotency, review evidence when
required, and typed execution obligations. The carrier
accepts no unprepared Grant: CaseState must show that the exact Grant was
consumed by the current `EffectPrepared` transition. Any intervening Case
transition makes an issued Grant stale before PREPARE. Repeated invocation of
one prepared effect observes intended post-state and returns `already_applied`
without another mutation.

Before mutation YAI records file absent/file/type plus SHA-256 digest and size
where applicable. The carrier canonicalizes the existing target parent, rejects
symlink escape, creates a same-directory temporary file, writes and `fsync`s
it, atomically renames it, and `fsync`s the parent directory. The implemented
durability claim ends at those successful local filesystem calls; hardware,
remote filesystem, or whole-system durability is not claimed. Applied status
requires a post-observed digest equal to the intended digest.

A crash after PREPARE leaves a discoverable prepared effect. `yai effect
reconcile --case ...` enumerates unresolved CaseState refs after restart and
compares the real target with the persisted expected pre-state and intended
post-digest. It concludes effect observed, no effect observed, conflict, or
still indeterminate. `--retry` is allowed only for an unchanged current
PREPARED state; ambiguous or conflicting state is never guessed away. Tests
inject crashes after Grant, after PREPARE, after visible rename, and after
receipt construction but before FINALIZE. The visible-effect crashes finalize
after restart without a duplicate write.

[`review.rs`](../cmd/yai/src/review.rs) owns only the Case-native operator
boundary. Policy may require review and Case-bound reviewer roles; the legacy
resource policy-owner field is compatibility metadata, not authority.
`REQUIRE_REVIEW` issues no Grant, commits a typed request for the
already-normalized Operation, and stops the runtime. APPROVE/DENY/DEFER actions
are identity- and digest-bound to that review, Case, Operation, reviewer and
expected generation. APPROVE and DENY derive a new effective Decision;
approval itself performs no effect. Resume revalidates the exact
EffectivePolicy identity/digest plus Case/resource bindings and executes the
original Operation through the existing controlled
carrier. `CompatibilityReview`, `PendingOperator`, `ReviewResolved`, and
`Quarantined` remain reader/reducer vocabulary for v1-v3 data only; no active
writer or product command produces them.

The direct `carrier fs-write` command and Rust primitive were removed after
characterization. [`filesystem.rs`](../cmd/yai/src/filesystem.rs) now owns only
read-only compatibility observation. `yaid run-filesystem-loop` no longer
writes `output.txt` or claims an executed receipt; it produces explicit
descriptor/no-effect fixture records used by older tests.

Surviving C control, filesystem/process carrier, receipt, and observation
components contain typed or platform properties protected by component tests.
They are built separately and are not normal product call paths.

## Current governance intake behavior

[`governance.rs`](../engine/yai-engine/src/governance.rs) owns one source
compiler, not a governance plane. Bounded JSON under
`yai.policy_source_input.v4`, Markdown with an explicit policy JSON block, and
the supported text-PDF policy-sheet profile feed the same compiler. JSON keeps
`yai.policy_source_artifact.v4`; document sources use v5 to retain exact original
bytes, media type, extractor identity and honest extracted-block coordinates.
The SHA-256 identity covers the original document, not merely extracted text.
No model or free-form policy interpreter participates. Text outside the explicit
structured block remains unresolved and blocks qualification; unsupported PDFs
refuse without OCR. See the [format contract](reference/governance.md).

The compiler emits `yai.parsed_policy.v2` facts for four current families:
operation restriction, review requirement, evidence obligation and scoped
proposer/reviewer authority requirement.
Every fact retains its source artifact and JSON location. Normalization emits
`yai.policy_ir.v2`, deterministically deduplicates equivalent semantics,
preserves unknown rule kinds as unresolved, and records contradictory outcomes
as typed conflicts. Unknown syntax/schema or malformed known rules fail;
unresolved/conflicted candidates remain inspectable but cannot validate.

`yai.policy_artifact.v5` embeds the parsed and normalized provenance chain,
including bounded declared `source_system` and `source_uri` origin metadata,
and
uses source version plus content/IR digests for immutable identity. The same
LMDB environment contains four logically separate canonical governance
databases: immutable sources, immutable artifacts, lifecycle events by ID and
their append order. A fifth rebuildable index accelerates current-policy lookup
but is not authority. This is an independent governance history, not a synthetic
Case ledger and not derived state. Lifecycle is reconstructed from integrity-
bound events:

```text
candidate → validated → published → superseded | retired
```

The shared LMDB map is configurable at open and defaults to 256 MiB (formerly
16 MiB). The supported H8 catalog contract is 256 retained sources of up to
256 KiB each plus their artifacts/events in the shared environment; capacity
exhaustion is explicit and cannot partially commit a governance transaction.

The artifact bytes never change. New authority-bearing policy lineage is
exactly `tenant_id + policy_key`; legacy v1-v4 artifacts preserve their
historical `owner_ref + policy_key` lineage but cannot be adopted as new Tenant
authority. A declared version identifies at most one immutable content inside
that lineage. Publishing another validated version in the same lineage appends
`superseded` for the previous publication and
`published` for the new artifact. `runtime_consumable` is true only for a
qualified artifact whose derived lifecycle is currently `published`; it means
eligible for future Case binding, not effective or authoritative now.

[`policy.rs`](../cmd/yai/src/policy.rs) provides ingest/inspect/validate/
publish/retire/revoke/list. Reads are pure and Tenant-filtered. Mutating
commands authenticate the current POSIX Principal and require Tenant Owner
membership; lifecycle v3 records that Principal. Source `owner_ref` remains
declared provenance and cannot select the authority domain. Full
bounded source bytes are retained to make compilation reproducible; source
artifact absence or corruption leaves the separately stored artifact's
digest/parsed/IR and declared origin inspectable, but byte-level recompilation
then cannot be claimed. There is no product source-deletion lifecycle. Global
source retention/privacy policy remains open.

No standalone policy authoring operation appends a Case Transition, invokes a provider or
carrier, creates a Decision/Grant, or modifies filesystem resources.

The [unified source frontier](case-source-bootstrap.md) composes this authoring
owner with existing Resource admission, immutable content and Case history.
`CaseSourceDeclared` / `CaseSourceProgressed` persist exact role/scoped source
relations and bounded progress; CaseState v16 replays them. There is no new
canonical store or policy engine. Initial setup can read only exact declared
policy files before the Case has ever bound policy; publication/binding remain
explicit. Ordinary file/directory, SQLite observation and HTTP representation
acquisition follow current EffectivePolicy. Denied sources are not acquired.
Original policy bytes serve both policy and knowledge roles without duplication;
acquisition alone produces no documentary claims. The separately invoked bounded
[knowledge derivation](source-grounded-knowledge.md) now consumes these revisions.
Inventory/resume/revision/revoke and current-policy source reads are normal CLI
actions; LMDB remains 37/40. Acquisition does not itself derive knowledge or
invoke Recall/W. The separate bounded Recall integration is described below.

These are bounded executable source relations, not generic source-environment
coverage. Resource attachment/discovery and filesystem locators coexist with
retained immutable content/provenance, policy intake/binding and derived
graph/index/memory/Recall. The source-bootstrap profiles above retain exact
original/observation backing; they do not implement the target's general choice
of in-place access, permitted snapshots and disposable derived caches or a global
retention/privacy lifecycle. General semantic source organization, the Case Source
Map and YAI Studio remain unimplemented
targets in the [Roadmap](../ROADMAP.md#case-source-bootstrap-and-source-grounded-knowledge--adopted-target).
Bounded unified bootstrap is implemented; its generalization is not.

Source-grounded **domain knowledge D_t** now has a bounded deterministic foothold.
`LmdbRecordStore::case_knowledge_authorized` qualifies current source/Participant,
policy, per-content access and exact retained backing in one read snapshot.
`memory_hierarchy::knowledge` derives typed source/structure/entity/topic/value
units; `graph::knowledge` derives provenance-bound documentary relations; the
existing W19/H19 BM25 owner indexes qualified text. No derivation, index, graph or
wiki build appends a Transition or creates a persistent M07 cache/database.

`yai.source_knowledge.v1` retains source-stated, deterministic-structure and
recorded-observation postures separately. Explicit entity/property disagreements
do not select current truth. SQLite schema descriptions require an exact admitted
schema-query profile; arbitrary query output is not silently a schema producer.
The PDF text extractor is shared with governance, but no policy interpretation,
publication or binding is performed by knowledge derivation. Current source or
catalog revocation removes material before graph/search; missing backing is
explicit, never replaced by a live file. Existing snapshot-only source profiles
are not broadened into general source-in-place retention.

The native `case knowledge build|inspect|search|graph|wiki|resolve` surface consumes
typed engine results below presentation; future clients need not parse CLI text.
Navigation is generated/read-only. Cross-Case policy-original/profile identity
reuse does not transfer Case visibility or qualify general shared extraction
caching. This boundary leaves Transition v19, CaseState v16, LMDB **37/40**, S/W
unchanged. The subsequent [Recall v2 integration](recall.md) consumes this
qualified D alongside H/S; automatic R → W remains unimplemented. See the
[independent derivation contract and oracle](source-grounded-knowledge.md).

A host-visible mounted path may conceptually fit the existing filesystem resource
model, subject to its confinement/identity/availability contract. No retained
product evidence qualifies NAS/SMB/NFS integration here. Protocols, credentials,
mount/discovery lifecycle and cross-machine recovery are separate host/external
responsibilities, not capabilities conferred by a Case attachment.

Wave 9 adds the distinct Case-native boundary in
[`case_policy.rs`](../engine/yai-engine/src/case_policy.rs). One atomic catalog+
Case transaction verifies an exact currently published artifact and appends a
typed v2 bind or replacement Transition; unbind is also canonical. The
artifact Tenant must exactly equal the immutable Case Tenant. CaseState keeps
only compact exact binding refs. A published newer version never moves an
existing Case automatically.

`yai.effective_policy.v3` is derived from current bindings plus the exact
immutable artifacts under `yai.policy_materializer.v3`. Its identity retains
the Case Tenant. Inputs are sorted;
DENY dominates ALLOW, required review dominates optional review, evidence
obligations and role requirements compose additively, and provenance from every
contribution survives. Its cache
is droppable; readiness (`unconfigured`, `ready`, `blocked`) is recomputed from
canonical bindings and available integrity-valid artifacts. Catalog drift is
reported independently from temporal validity/revoke. Binding/materialization alone emits
no Decision, ReviewRequest, Grant, effect or provider invocation. The separate
admission owner consumes Ready policy only after a normalized Operation exists.
See the canonical [governance
reference](reference/governance.md).

[`admission.rs`](../engine/yai-engine/src/admission.rs) owns the one current
operational policy boundary. It intersects a normalized Operation with the hard
resource envelope, Ready EffectivePolicy, Case-bound Participant roles and
typed evidence. `yai.decision_basis.v3` preserves the immutable Case Tenant,
exact EffectivePolicy,
binding/artifact/rule provenance, mechanical posture, temporal validity,
authority evaluation, obligations and final reason. A committed
`yai.decision.v3` binds that basis; only final ALLOW can produce finite
`yai.execution_grant.v3` under the same current policy identity. Missing
ALLOW, missing proposer authority, forged provider
lineage or stale policy basis fails closed. `source_provenance` is satisfied
only by canonical ProviderInvocation/ProviderResult lineage; `audit_reason`
requires an actual human ReviewAction reason; pre/post observation obligations
are carried into the Grant and never weaken carrier baseline safety.

The v2 ReviewRequest carries policy basis/effective-policy digests and role
eligibility. Approval cannot override DENY and cannot survive a changed Case
policy basis. New ReviewAction v2 records the per-command authenticated
Principal and exact linked reviewer Participant. Tenant ownership does not
create a Case role; policy eligibility still consumes only current Case roles.

### Local identity and Tenant isolation

[`security.rs`](../engine/yai-engine/src/security.rs) is the sole identity and
isolation semantic owner. `AuthenticatedPrincipal` is invocation-scoped and
sealed; on POSIX it observes real/effective UID and GID from the kernel and
uses the effective UID binding for authority. `$USER`, `HOME`, caller strings
and eUID 0 do not grant YAI authority. Stable `yai.security_principal.v1` and
immutable `yai.tenant.v1` records plus append-only `yai.security_event.v1`
Owner/Member history live in five dedicated databases in the existing LMDB
environment. Organization is immutable Tenant metadata only.

Every new Case opens through a v8 TenantCaseOpened transition and can never
change Tenant. A human Case action resolves authenticated Principal → selected
Tenant membership → one active Principal/Participant link → existing
Participant roles. The link carries no role. New scoped administrative writes
and reads are verified at the store boundary; knowledge of IDs is insufficient.
Legacy v1-v7 Cases and v1-v4 policy artifacts remain readable and replayable as
`legacy_unscoped`, but cannot produce new live authority without a future
explicit migration contract.

For the local filesystem carrier, canonical root equality or ancestor overlap
across different Tenants is rejected at attachment. This is a conservative
alias barrier, not a fencing engine or hostile mount-namespace defense. The
same Principal may belong to multiple Tenants, but every operation resolves
one Tenant context and never combines catalogs or Case-derived reads.

## Current provider and context behavior

[`semantic_state.rs`](../engine/yai-engine/src/semantic_state.rs) composes a
read-only `SemanticState` from CaseState and its exact ordered history, requiring
`CaseState == replay(history)`. S is a qualified composition of existing owners,
not a new canonical owner, mutable memory object or database. Its source identity
binds representation version, full canonical history, current materialization and,
for cognitive execution, current catalog-derived normative status. That status is
supplied by the same EffectivePolicy materializer as deterministic admission,
including validity and revocation outside Case generation changes. Observation
clock samples alone do not change semantic identity; changed validity does.
Immutable content remains owned separately; selected entries preserve exact
object references, inline text when owned by that content contract, and provenance.

One `CompilationRequest` selects `SemanticWorkingState` W under exact Participant,
admitted view, purpose, intent/output-contract identity, required source references,
resource relevance and item/semantic-unit budgets. Qualification precedes relevance.
Own Turns and exact currently authorized I06 executor delegations are distinct;
delegation does not expose the author's unrelated Turns. Current policy bindings,
control state and unresolved work retain their posture. A mandatory
`EffectiveAuthority` entry carries exact EffectivePolicy identity, readiness,
validity and normalized rules relevant to visible resource kinds, including DENY,
review, roles and evidence obligations. This is a derived cognitive view, not a
second policy evaluator or permission to execute. Current governance,
expiry/revoke, Grants and dispatch checks remain in their existing owners.
Raw imported prose is not injected as policy instructions. The bounded recent
`DecisionEvidence` family carries the executing Participant's source-closed
DecisionBasis, rule refs, resource, canonical sequence/recorded time, review and
result references. Its optional six-decision window is not general Recall or
as-of reconstruction; historical permission is never current authority.

W is deterministic, provider-independent, disposable and source/request/version
bound. The compiler uses the shared semantic budget kernel in
[`residency.rs`](../engine/yai-engine/src/residency.rs) once, upstream of rendering.
Exact requirements cannot be omitted; missing and undisclosed references produce
the same refusal. Bounded recent history and current-intent/resource-matched derived
material are optional. Selected/budget-omitted candidates have per-entry reasons;
locality omissions have aggregate counts, without exporting hidden IDs. Semantic
units are the existing serialized-character estimate, not authoritative tokens.

Normal conversation, Golden work, Workflow and bounded Case execution lower W
through [`context.rs`](../engine/yai-engine/src/context.rs) into Projection v10 and
ContextFrame v10, then the existing exact provider adapter. W's identity is reachable
from `projection.bounds.working_state_id` through existing invocation lineage.
`context inspect --id <working-state-id>` recompiles the Case snapshot against
current catalog truth and checks equality. A changed catalog can invalidate that
derived artifact even at the same Case generation; this is not historical policy
time travel. Retained DecisionBasis remains canonical historical backing.
Lowering rejects a stale/tampered W by source/request/version and full recompilation.
ResidencyPlan is a compatibility report of that selection, not a second selector.
The older `compile_projection` facade remains for bounded engine inspection tests;
it shares source extraction, and no normal provider path calls it.

SemanticDelta v1 describes source/destination generations, request and entry
additions/replacements/removals with digests. It is not a mutation command. Delta
application verifies both qualified sources, exact forward history and the old W,
then explicitly reports `FullRecompilation`; all supported classes currently use
this correctness fallback. Full/delta equality is qualified, incremental speed is
not. No cache is needed for correctness. S/W/compiler/delta identity contracts are
v2 for S/W/compiler and v1 for delta. The semantic refoundation itself left
Transition v18, CaseState v15, owner counts and LMDB 37/40 unchanged; source
bootstrap later advances only the Case source lifecycle to v19/v16.
Policy-only changes without forward Case generation
require full recompilation, not a fabricated forward semantic delta.
At the invocation commit, the product revalidates W and its exact Participant,
output contract and Projection in the same LMDB write transaction as canonical
invocation admission. A derived cache entry cannot attest its own currentness.
The future public W → YVEX experiential-state consumer is not implemented.

The target doctrine additionally distinguishes qualified historical experience H,
current S and task-conditioned Recall. The current `SemanticState` composition
above includes history for replay/source qualification. The separate bounded
historical reader below is not a Recall Compiler or general event-time query.
Neither W20's bounded
hierarchy nor the existing finite runtime proves prompt-independent E refresh
or persistent internal model deliberation. Those remain target/research properties.

[`memory.rs`](../engine/yai-engine/src/memory.rs) first derives a versioned
operational-memory materialization from typed invocation/result, normalization,
Decision and effect-chain Transitions. Each entry has a deterministic identity,
Case/generation, semantic kind, epistemic posture, bounded typed value,
Transition/Observation/Receipt/causal provenance, participant visibility and
active/superseded lifecycle. Provider claims stay explicitly
`provider_originated_claim`; only finalized/reconciled observations produce an
observed resource-effect memory. PREPARED/INDETERMINATE residue stays unresolved.

Retrieval is a pure `qualify → filter → rank/select` algorithm. It filters Case,
current generation, admitted participant/view, lifecycle, typed kind and direct
resource/causal constraints before deterministic ranking by posture, purpose,
direct match and recency. It defaults to eight entries and reports selection,
omissions, rejections and machine-readable reasons. Wave 19 retains that v1
path and adds content-addressed representation documents, a derived BM25 index,
normalized profile-bound vectors, exact-cosine reference scan and v2 hybrid RRF
after the same hard qualification barrier. Missing/stale/corrupt indexes or a
failed local encoder degrade to qualified operational retrieval and then the
canonical fallback; no graph requirement or provider-specific ontology enters
core.

[`memory_index.rs`](../engine/yai-engine/src/memory_index.rs) owns these pure
derived algorithms, manifests, checksums and the disposable filesystem layout
under `$YAI_HOME/store/derived-memory/v2`. It does not own memory truth. Builds
are serialized per Case/profile, validated before atomic publication and
content-idempotent under concurrent processes. Encoder profiles reuse W18
ProviderGovernance and require an exact loopback `text_embedding`
qualification. Exact scan is bounded to 50,000 documents; HNSW/ANN remains an
explicit deferred accelerator after measured exact-query characterization.

[`memory_hierarchy.rs`](../engine/yai-engine/src/memory_hierarchy.rs) adds W20's
deterministic structural Episodes and typed evidence-bound semantic assertions.
Epistemic class (MechanicallyGrounded, EvidenceBoundInference,
ProviderOriginatedClaim, ControlHistory) is separate from lifecycle; repetition
does not promote a provider claim into an observed fact. Bounded support graphs,
contradictions and declared mechanical supersession retain exact provenance.
Generation-based retention changes retrieval posture, not canonical history.
The hierarchy rebuilds from typed Transitions and exact recorded consolidation
ProviderResults under the bound normalizer, without re-inference. It owns no
independent memory truth. This does not make every new conversation object
automatically memory material.

W20 representation v2 and RetrievalSet v3 admit operational, episodic and
semantic families under the combined H19 bound. H19 selected-source revalidation
still resolves current qualified sources for explicit indexed search; indexed text is not
trusted authority. W and its Projection/ContextFrame retain family, epistemic class,
lifecycle and support. See [W20](https://github.com/yailabs/yai/blob/c187648e9d9909d4d9b6711f131fa726d48cd585/refoundation/foundation-recovery/wave-20/WAVE-20-REPORT.md)
and [H19](https://github.com/yailabs/yai/blob/c187648e9d9909d4d9b6711f131fa726d48cd585/refoundation/foundation-recovery/hardening-19/HARDENING-19-REPORT.md)
for the bounded proof and historical external-acceptance limitations. These
are access/derivation algorithms, not a universal Memory database or implemented
semantic paging service. Normal working-state compilation reconstructs operational,
episodic and active supported assertions from canonical sources, independent of
index availability. It no longer refreshes a vector index or dispatches an encoder
implicitly. Explicit qualified search/index build/rebuild remains available. Derived
support must remain visible; removed/inactive support cannot launder an assertion.

The compiler fails before rendering if the participant lacks the exact
`model/model_context` admission (or the established exact governed selection proof).
It includes the participant's own binding,
current provider/model binding, logical resources, latest Decision, all
unresolved effects, the four most recent finalized effects, bounded recent
typed legacy interactions, ordered multipart conversation Turns, provider
claims, and typed provenance-bearing retrieved memory. A conversation entry
preserves part order, modality, digest and original/derived/human-edit posture;
it does not turn a transcript into an Observation or semantic fact. Candidate
selection is intentionally broader than provider input.
Residency pins mandatory current/unresolved/observed truth first, then retains
or reintroduces ranked optional entries under item and semantic-unit limits;
every omission has an inspectable reason. Provider claims carry an explicit
non-authoritative posture;
finalized resource consequences cite Transition, Observation and EffectReceipt
refs; indeterminate effects remain unresolved. Runtime selection defaults to
24 items and 4,096 semantic units, never dumps the complete ledger, reports
omitted material, and rejects a budget smaller than mandatory current state.

ContextFrame has separate identity because one Projection supports different
tasks and typed output contracts. It carries provider-independent instructions,
selected semantic entries and the Wave-3 filesystem proposal contract. It owns
no CaseState, prompt transcript, token IDs, or runtime cache. The
OpenAI-compatible render function in
[`context.rs`](../engine/yai-engine/src/context.rs) combines a frame with the
minimal provider/model profile and creates a distinct render identity/digest;
[`provider.rs`](../cmd/yai/src/provider.rs) owns the HTTP transport.

Invocation and ProviderResult payloads explicitly reference
provider ID, model ID, Projection, ContextFrame, Case generation, render ID/
digest and output-contract ID. ProviderResult content remains non-authoritative.
A historical typed `InteractionTurnRecorded` transition preserves the old
completed text task/result lineage. New `ConversationTurnCommitted` transitions
record submitted user content before any invocation; a later
`ProviderInvocationStarted` cites the Turn and does not redefine it. The old
JSONL `InteractionTurn` remains compatibility output. New invocations
no longer write or consume `ParticipantViewFrame`; that RecordKind survives
only as historical input/counting compatibility. The free-form Case-entry
preview is explicitly labeled compatibility output and never reaches a
provider.

An optional `ProviderContinuationReference` is accepted only as an opaque,
provider-bound, runtime-bound transport optimization. Its value is never put in
CaseState, canonical Transition history, Projection/Frame identity, or the
derived artifact store. Historical `invalidated_and_retried` dispositions
remain readable, but a generic `invalid_continuation` HTTP response is not
proof that cognition did not execute and therefore cannot trigger an automatic
retry. A later explicit invocation rebuilds context from canonical Case state
without reusing the incompatible reference.

Product tests prove that Provider A can propose a real Wave-3 filesystem write,
Provider B can replace its binding after FINALIZE and observe both the current
typed resource consequence and its selected derived memory with Transition,
Observation and Receipt provenance, and a model ID can change under one provider
ID without changing Case identity. A separate fixture loses continuation state,
proves that the ambiguous response does not retry, then restarts on a new
endpoint, rebuilds a new Projection/Frame from Case state, and preserves typed
interaction/result
continuity. Loss of the derived context-artifact database likewise leaves
ledger and CaseState unchanged. OpenAI-compatible usage fields are captured
when supplied; token counts and latency are invocation telemetry rather than
operational authority. Unavailable usage remains unknown.

The current transport includes TLS in
[`provider_transport.rs`](../cmd/yai/src/provider_transport.rs), and W19/H19
include qualified embeddings. Streaming/transport-abort semantics, authoritative
token estimation, a native YVEX/KV protocol, learned ranking/compression and a
ContextDelta consumer remain absent. Host cancellation gates future dispatch;
it does not promise abort of an already delivered request. The context-residency
lab remains research evidence and does not prove KV reuse.

TEST.TOPOLOGY.0 keeps proof class, evidence posture and provider mode orthogonal.
[tests/README.md](../tests/README.md) and [the validation guide](test-cases.md)
own gate selection. Local HTTP/TLS/PTY qualification does not establish external
YVEX interoperability or model quality. This architecture alignment reruns
documentation guards only; historical runtime passes are cited, not recreated
as new execution evidence.

## Historical semantic reconstruction

`yai case as-of CASE GENERATION_OR_TRANSITION` reads a bounded
[`HistoricalSemanticView v1`](../engine/yai-engine/src/semantic_state/historical.rs)
within the existing semantic composition owner. It is neither W nor an execution
input. The store qualifies current Tenant/Principal/Participant and source history
in one read transaction, verifies current CaseState against full replay, then
replays the exact prefix. Generations and Transition IDs are exact equivalent
coordinates; zero/future/foreign IDs and wall-clock queries refuse.

Prefix materialization, evidence known by then, historical bound-policy meaning,
current normative posture and a derived current comparison remain separate.
Supported families are explicitly listed: lifecycle/identity, exact policies,
scoped resources/content/observations, own operations/DecisionBasis/reviews/Grants,
filesystem effects, own Turns and provider claims. Original payload availability
is resolved without reading mutable resources as replacements. Workflow/Handoff
detail, process-effect detail, W20 historical derivation and general temporal
Recall are outside this initial profile, not silently asserted complete.

Current Principal linkage controls operator inspection; explicit cognitive views
also require current admission. No owner override impersonates an unlinked model.
Historical resources must match current envelopes and disclosure. Another
Participant's history is not granted by common Tenant, path or provider identity.
No broader audit authority is introduced. Current writers do not offer resource
detach/rebind or Participant-view revoke; this reader does not invent them.

Historical policy composition uses exact immutable artifacts matched to the
prefix bindings and their publication anchors. Case and governance-catalog
sequences are independent: an arbitrary prefix does not establish a complete
catalog lifecycle cut. The response marks that missingness, while each recorded
DecisionBasis preserves its exact evaluated authority time, rules and validity.
Today's revoke is reported as **current** posture, not silently projected into
yesterday. Sampled current clocks are excluded from reproducible meaning, not
presented as artificial historical times. Observation and Transition recording
times retain their distinct existing producers; generic occurrence time is absent.
A later observation referring to an older event does not enter the earlier prefix.

Identity binds representation, normalized coordinate, full canonical history,
current disclosure, current normative meaning and resolved backing availability.
Item/byte budgets refuse rather than truncate; these are inspection limits, not
provider token capacity. Comparison labels unchanged state, still-recorded
non-authoritative evidence, superseded lineage, removal, change and items not yet
known at the historical coordinate. It is not a mutation or SemanticDelta.
No caches or inference are required. Reconstruction cost includes full history
verification and prefix replay; no Case-age-independent CPU claim is made.
See the [historical qualification report](https://github.com/yailabs/yai/blob/c187648e9d9909d4d9b6711f131fa726d48cd585/refoundation/validation/historical-semantic-reconstruction/REPORT.md).
That historical boundary left Transition v18 / CaseState v15, S/W/compiler and
context contracts, canonical owners and LMDB **37/40** unchanged.

## Qualified experience relations

`yai case experience CASE current|GENERATION|TRANSITION` derives
`yai.experience_relations.v1` in the existing graph/access owner. The same
transactional historical reader qualifies current identity/disclosure, canonical
replay and exact backing; the graph constructor consumes only that qualified
immutable result. It neither reads a cached edge as truth nor creates another
history reader, event ledger, memory store or LMDB database. Full-history
qualification still costs work proportional to Case history.

The profile projects experience-bearing recorded payloads, not every Transition.
An event exposes its exact Transition and owned object identities, recording
generation/time, supported observation time, epistemic posture and source-closure
flag. Missing occurrence time stays absent. Arbitrary timestamps or references
inside provider/resource result JSON are not temporal/causal authority.

Relations retain a kind, epistemic strength, both backing Transitions and typed
payload field, plus the generation at which the relationship became knowable:

| Meaning | Exact current sources |
|---|---|
| Recording precedence | Visible Transition sequence; neither physical event order nor global adjacency |
| Structural lineage | ProviderResult → Operation, Operation → Decision, Decision → review/Grant, review → action/re-evaluation, Grant → PREPARE, observation → admitted content, Result → interpretation |
| Normative/evidence support | Recorded DecisionBasis binding and obligation/review references, not today's policy projected backwards |
| Observed consequence | Exact PREPARE → recorded receipt/observation and Decision → admitted resource observation; failed/no-effect outcomes are not called successful mutations |
| Lifecycle | Explicit policy replacement/unbinding and review/Grant invalidation; not recency, contradiction or causal discovery |

`Transition.causal_refs` enforces required lineage references for typed writers;
extra references have no independent causal meaning. This reader uses the typed
contracts, never a free-text causal edge. Provider claims remain claims even when
their **origin relation** is structurally established. No inferred/model-proposed
physical causal relation is promoted. General assertion contradiction, Workflow
and Handoff traversal remain outside this initial scoped profile; W20's existing
qualified contradiction/mechanical-supersession rules are unchanged.

Optional `--from EXACT_REF --to EXACT_REF --hops N` selects a deterministic
directed path. Recording-order edges are excluded from path search unless
`--recording-order` is explicit. Equal-length paths use semantic kind ordering,
preferring exact lifecycle links over auxiliary support, not hash-order accident.
`no_qualified_path_within_profile_and_hops` does not distinguish absent evidence,
undisclosed intermediates or paths outside the selected profile/hop bound.
Unknown and hidden anchors share one refusal. This is exact traversal, not Recall.
An ambiguous object identity owned by multiple visible events refuses the view;
this profile does not guess which occurrence an unversioned reference means.

Only disclosed nodes and qualified edges affect path explanations, slice
identities or exported counts. No raw payload, unfiltered history hash or private
scope hash is exported. A current-disclosure digest binds the requesting
Participant, its links/views and visible resources, not other Participants'
private state. Changing that scope reidentifies the view; inspecting an old cut
never restores old permissions. No view is accepted as execution authority.

W20 MemoryEpisode records/derivation remain unchanged. Inspection shows disposable
**scoped Episode slices** grouped by typed operation/result/Turn identity, not
recursive user JSON fields. Their identities are deliberately distinct from W20
records. Origin/support relations can cross slices without merging Episodes;
the committed-origin oracle also verifies the two actual W20 Episode identities
remain distinct. Slices are bounded access metadata, not a new Episode owner or
multi-timescale segmentation system.

Required unavailable backing marks the event `source_closed=false` and suppresses
dependent edges; intact recorded existence remains inspectable. Original-source
loss with an intact exact PolicyArtifact does not erase artifact semantics.
The historical surface exposes the finer source-availability details. No mutable
resource replaces missing original content. Generic resource PREPARE and
indeterminate payloads now join that reader's existing scoped terminal observation
family; historical JSON shape/version remains v1, with additive family coverage.

Source qualification and output budgets are separate. The product uses the
existing bounded historical profile (4096 items, 16 MiB); default graph output is
256 events, 1024 relations, 1 MiB, and paths allow 16 hops. Excess output refuses,
not silent truncation. These are inspection bounds, not production scale or
model-token limits. Each read rebuilds from sources; no provider or persisted
compilation cache is required. See the
[temporal experience evidence](https://github.com/yailabs/yai/blob/c187648e9d9909d4d9b6711f131fa726d48cd585/refoundation/validation/temporal-causal-experience-0/REPORT.md).

## Query-conditioned Recall

`yai case recall CASE QUERY` returns `yai.recall_trace.v2`, derived in
[`memory_hierarchy::recall`](../engine/yai-engine/src/memory_hierarchy/recall.rs).
It is independently inspectable, not automatically inserted into W or provider
input. Current semantic working-state compilation and the runtime are unchanged.
The public store method reuses the historical reader's single qualified snapshot
and its original canonical history; no second replay engine, Recall database or
persisted compilation cache is introduced.

[The integrated contract](recall.md) adds documentary candidates to the same
resolver, with current/as-of source qualification, shared historical read basis,
family-local BM25, mandatory anchors and atomic aggregate-budget admission.
Documentary segments preserve exact source/profile/coordinates and epistemic
posture beside experience events, never an invented common causal timeline.
Source-owned acquisition evidence cannot bypass source revocation through H.
Typed v1 requests retain the following H/S contract without documentary fields.

`RecallRequest v1` binds Case, expected current generation, current linked
Participant, query, exact generation/Transition cut, required references and
candidate/event/relation/segment/depth/unit/byte bounds. Product defaults to the
current cut. Equivalent exact coordinates normalize to one generation; query
normalization reuses W19's deterministic query document, with oversized input
refused before normalization rather than silently truncated. Scope is the existing
Principal-linked operator-inspection contract, not new audit/model authority.

Candidate discovery and semantic resolution are different stages:

1. The historical reader qualifies current disclosure and exact sources before
   any public rank, candidate count, reference or explanation is produced.
2. Exact event/object/resource refs and wholly disclosed W20 Episode IDs resolve
   directly. Unknown and hidden required anchors share one refusal. An anchor
   whose recorded object exists but original backing is unavailable remains an
   explicitly incomplete trace, never substitution by a fuzzy match.
3. Existing BM25 indexes freshly derived typed event labels and mechanical W20
   assertion documents. No stored index text is trusted. An optional engine
   `RecallVectorInput` consumes pre-encoded same-profile query/vector evidence;
   each visible embedding is checked against its newly rebuilt v2 assertion.
   Unknown/hidden documents contribute nothing, including corpus counts or hashes.
   Malformed visible vectors, profile/query drift or source-integrity drift refuse.
   The CLI baseline is exact/lexical: no implicit encoder, LLM or vector fallback.
4. Required sources are selected first. Each optional candidate and its necessary
   support/conflict context is an atomic group; event-budget overflow omits that
   group without evicting required anchors. Mandatory context overflow refuses.
   Current-at-cut policy lineage, W20 mechanical supersession and unresolved
   contradiction cannot disappear because of a rank or expansion cutoff.
5. Qualified graph relations expand the selected evidence. Recording proximity
   is not an expansion rule. Support is followed towards backing, without
   fanning out from a common policy ancestor to every unrelated operation.
   Each candidate group starts from its own discovery seeds; repeated candidates
   do not reset the depth of already expanded nodes. Source closure/conflict
   resolution runs again after expansion and closed groups merge under the bound.
6. Output is organized in recording order, with scoped typed Episode slices and
   independent segments. Selection association is not a new graph edge; a trace
   may be disconnected. Relations retain their existing kind/posture/backing.

The trace carries event/object identities, recording/observation coordinates,
labels, inclusion reasons, exact W20 assertion values/classes/lifecycles,
contradiction sets, typed relations, source availability and budget omissions.
Missing occurrence time stays missing. A late observation never enters an older
cut. Policy replacement labels the earlier binding historical without changing
its recorded DecisionBasis; a bound-at-cut policy is explicitly not permission
for current execution. W20's single-current mechanical resource digest can
supersede an older digest; conflicting provider claims remain claims and do not
resolve by recency or majority. No general model narrative is generated.

Request/trace SHA-256 identities bind version, normalized request, current scoped
disclosure, qualified disclosed source material, selected semantics, availability,
bounds and optional vector evidence. Private whole-history hashes and hidden
vector IDs are excluded. Timing measurements are outside semantic identity.
Re-running the same request reconstructs the same trace after restart or loss of
graph/memory/Episode/index derivations. A new current generation, changed own
scope, backing loss, query/budget/version change reidentifies or refuses it.

Default output limits are 16 candidates, 64 events, 128 relations, 16 segments,
4 expansion hops, 16,384 semantic units and 1 MiB. Maximum accepted request bounds
are 256 candidates/events, 1,024 relations, 64 segments, 16 hops, 262,144 units and
1 MiB. Units are serialized characters divided by four (rounded up), **not model
tokens**. The separate historical preparation profile remains 4,096 items/16 MiB;
excess source/output complexity refuses. Bytes/units/relations/segment limits are
final all-or-refuse checks, not permission to trim mandatory context.

The initial profile composes the historical/experience families and mechanical
W20 assertions. It does not qualify arbitrary natural-language task understanding,
general consolidation-derived inference, complete Workflow/Handoff Recall,
universal source coverage, wall-clock as-of, learned navigation or Recall-aware W.
Lexical hits are discoverability, not a completeness guarantee. Pre-encoded vector
fixtures prove mechanics, not real encoder suitability or memory usefulness.
The 81/20,081-Transition oracle selects 8 events in 5 segments, with 64 visible
distractors; the extra 20,000 records are unrelated control history, not 20,000
visible semantic objects. Host reconstruction remains history-dependent.
See [Recall evidence](https://github.com/yailabs/yai/blob/c187648e9d9909d4d9b6711f131fa726d48cd585/refoundation/validation/recall-trace-0/REPORT.md).
That Recall boundary left Transition v18 / CaseState v15 and all
S/W/Projection/ContextFrame schemas, semantic/operational owners and LMDB
**37/40** unchanged. V2 evolves only derived Recall contracts; current canonical
Transition v19 / CaseState v16 and LMDB 37/40 remain unchanged. Source acquisition
alone does not retroactively broaden v1 Recall.

## Current agentless Case runtime

The scoped [experience inspection](#qualified-experience-relations) above is an
independent read-only consumer. It does not change the runtime's S/W preparation,
admission or provider path.

[`case_runtime.rs`](../cmd/yai/src/case_runtime.rs) owns one disposable
transition algorithm. It is not an Agent, workflow, scheduler, or state owner.
Each iteration reloads CaseState, reconciles prepared/indeterminate filesystem
effects, resolves normative readiness before provider invocation, repairs stale
or missing derived memory, performs qualified retrieval
and Residency planning, compiles a fresh Projection/ContextFrame, invokes the
current provider/model, persists ProviderResult, and advances a valid candidate
through the existing Operation/Decision/Grant/effect boundary. The next
iteration starts from the newly committed Case generation rather than an
in-process narrative.

`yai case run`, `resume`, `status`, and `stop` expose bounded operator control.
A versioned JSON run checkpoint records only disposable execution-attempt
metadata: budgets, counters, pending ProviderResult identity, last derived
artifact/effect refs and stop reason. It owns no Case facts. Stops distinguish
completion, normative unconfigured/blocked, denial, malformed/provider failure, unresolved effect, budget
exhaustion, operator stop and invariant failure; a stopped run does not close
the Case.

One `yai.case_runtime_admission.v1` record in a separate LMDB database provides
single-host cross-process mutual exclusion for active Case advancement. The
claim binds Case, run, opaque owner token, PID and bounded expiry. LMDB write
serialization makes acquisition exclusive; same-owner renewal is explicit,
live competing owners fail closed, and expired or demonstrably dead local
owners can be reclaimed. Normal stop, completion, budget stop and
`AWAITING_REVIEW` release it. This metadata is not Transition history,
participant authority, an ExecutionGrant, or Case continuity.

Invocation, operation, semantic-context and cumulative estimated-input budgets
are enforced before transport or effect. Semantic size and conservative
rendered-input estimates are distinct from optional provider-reported token
usage. Mandatory current truth cannot be displaced by optional memory; if it
alone exceeds the configured budget, invocation fails explicitly.

Deterministic real-HTTP characterization performs 26 invocations and 24 real
Grant-controlled writes, including one DENY followed by a compliant proposal,
provider/model replacement after a committed ProviderResult, bounded 12-item
context, and restart after canonical-result, Grant, visible-effect and
post-FINALIZE/pre-memory boundaries. A visible filesystem effect with no
FINALIZE is reconciled before the next provider call. A separate 128-iteration
state/memory/context test grows more than 380 Transitions while keeping
retrieval and frames bounded. No Agent, Workflow or Orchestrator object
participates.

## Physical ownership after source refoundation

| Surface | Executable role | Classification |
|---|---|---|
| `cmd/yai/src/main.rs` | small process entrypoint calling the native CLI | product bootstrap, not an application API |
| `cmd/yai/src/cli/` | command registry, parser, help, product dispatch and output projection | native product frontend; some current orchestration remains CLI/store-coupled |
| `cmd/yai/src/command_adapters.rs` | adapt CLI operation IDs to existing handlers | command compatibility seam, not a second domain or public interface registry |
| `cmd/yai/src/conversation_controller.rs` | host-independent commit, thread projection, retry/cancellation posture, and ordinary conversation execution over shared semantic/provider boundaries | native REPLAI consumer and host-independent typed actions; no terminal, Case, provider, or content owner |
| `cmd/yai/src/conversation_cli.rs` | Advanced mutable draft preparation and SEND plumbing plus immutable Turn/content inspection | automation/reference-client boundary; no model or resource authority |
| `cmd/yai/src/cognitive_cli.rs` | Advanced argument parsing and rendering over shared cognitive execution | no second execution algorithm |
| `cmd/yai/src/cognitive_execution.rs` | shared exact realization and finite composition used by controller and Advanced CLI | process-local control, no canonical owner |
| `engine/yai-engine/src/conversation.rs` | ordered typed Turn/content/provenance contracts and private immutable byte ownership | one durable application-content owner for non-reconstructible original bytes; no execution owner |
| `cmd/yai/src/case_runtime.rs` | bounded disposable Case iteration, stop/budget checkpointing, automatic reconciliation and memory repair | product-reachable transition algorithm; never canonical owner |
| `cmd/yai/src/provider.rs` | Case admission/attachment compatibility, HTTP transport and typed invocation/result residue | product-reachable provider boundary |
| `engine/yai-engine/src/context.rs` | bounded typed Projection compilation, ContextFrame construction, provenance and the OpenAI-compatible render contract | product-reachable derived semantic compiler/render boundary |
| `engine/yai-engine/src/residency.rs` | deterministic mandatory/retained/reintroduced/omitted semantic selection and budget accounting | product-reachable pure derived planner; no persistent authority |
| `engine/yai-engine/src/memory.rs` | deterministic operational-memory derivation, provenance validation, supersession and qualified bounded retrieval; legacy MemoryCandidate summary compatibility | product-reachable derived algorithm/store contract; never canonical authority |
| `engine/yai-engine/src/memory_hierarchy.rs` | structural Episodes, typed semantic assertions, version-bound consolidation normalization, support/contradictions/retention and hierarchy rebuild | W20 derived algorithms, no new canonical memory owner |
| `engine/yai-engine/src/memory_index.rs` | deterministic memory representation/profile/embedding contracts, corpus/index manifests, BM25, exact cosine, hybrid RRF and atomic disposable index storage | product-reachable derived algorithm/store contract; no authority or independent owner |
| `cmd/yai/src/controlled_effect.rs` + `engine/yai-engine/src/effect.rs` | controlled proposal/admission/recovery orchestration and the Grant-validating Rust filesystem carrier | product-reachable first constitutional effect family |
| `cmd/yai/src/review.rs` | Case-native typed participant actions and effective Decision recording; never carrier execution | product-reachable human review boundary |
| `engine/yai-engine/src/governance.rs` + `cmd/yai/src/policy.rs` | deterministic source compiler, immutable PolicyArtifact/lifecycle contracts and thin operator surface | product-reachable Case-independent governance authoring boundary; no Case authority |
| `engine/yai-engine/src/case_policy.rs` + `cmd/yai/src/case_policy.rs` | exact Case PolicyBinding contract, deterministic EffectivePolicy materializer and thin Case-policy operator surface | canonical binding transition algorithm plus derived normative view; no operational authority |
| `engine/yai-engine/src/admission.rs` | immutable DecisionBasis, closed-world applicability, Case-role/evidence eligibility and policy-bound Decision/Grant admission | product-reachable operational admission; no policy authoring/materialization ownership |
| `cmd/yai/src/filesystem.rs` | read-only filesystem compatibility observation | product-reachable compatibility boundary |
| `cmd/yai/src/replay.rs` | legacy inspect/dry-run/isolated import and replay reports | product-reachable state compatibility boundary |
| `cmd/yai/src/graph_runtime.rs` | graph relation materialization, rebuild and query | product-reachable derived owner |
| `cmd/yai/src/analytics.rs` | DuckDB schemas, extraction and reports | product-reachable derived owner |
| `engine/yai-engine` | canonical Transition/CaseState semantics, LMDB authority, typed semantic-context compiler, legacy decoder, and reusable derived algorithms | product-reachable semantic/data authority |
| `cmd/yaid` + selected `system/` sources | daemon IPC, fixture loops, C journal/projection/hot snapshot | product-reachable process/platform boundary |
| separate C component archive | gates, carriers, process/observation and compatibility mechanics | component characterization; not product capability |
| tests/labs/history | current proof, research, and historical specification | evidence, never implementation authority |

There are 55 surviving headers under `include/yai`, down from 101. They are a
source compatibility surface for the product C subset and characterized
platform components; the repository does not install them. No external
consumer is known. The daemon socket JSON, JSONL record shapes, and public Rust
crate remain compatibility risks because external absence cannot be proven
from one checkout.

## Requirement/current/gap register

| Constitutional requirement | Current implementation | Gap |
|---|---|---|
| one canonical Transition Ledger with transactional CaseState | implemented in LMDB for typed payloads; provider, review, resource and Workflow are live consumers | operational checkpoint/compaction policy and demonstrated future consumers |
| carrier consumes an ExecutionGrant | governed filesystem write, confined process runner and admitted MCP tool call reuse exact grants and current authority; C carriers remain component-only | general database/HTTP mutation and broader process profiles are not admitted; no carrier registry owner exists |
| PREPARE/EFFECT/FINALIZE with indeterminate recovery | filesystem reconciliation and resource fenced publication preserve terminal/indeterminate truth; unused authority may expire before PREPARE, while prepared work must settle | no blind retry or invented reconciliation for unknown process/MCP outcomes; broader resource-specific reconciliation remains future work |
| filesystem attachment confinement | descriptor-relative reads/imports and the controlled write carrier preserve admitted roots and reject symlink/namespace substitution in qualified Linux tests | broader platforms require independently qualified confinement |
| distinct ProviderResult, Observation, EffectReceipt | separate Rust types and canonical roles for filesystem/process/MCP effects and bounded resource reads; compatibility export retains old receipt-shaped rows | future resource families require their own truthful result and reconciliation contract |
| Case plus materialized CaseState | implemented and replayable for provider/review/resource/operation/grant/effect refs and exact policy bindings | extend only for demonstrated future consumers; migrate daemon hot/fixture state only if it becomes canonical input |
| summary is presentation only | canonical reducers and migrated paths do not parse it; old projection/frame and analytics records use the compatibility decoder | migrate or retire remaining legacy-only producers and views |
| frontends consume one application meaning | bounded controller/review actions and authorized engine queries coexist with CLI argument/output adaptation and store-coupled orchestration | harden a frontend-independent typed boundary before qualifying interface export or native Studio; no CLI-output parsing or independent semantic registry for new clients |
| Projection/Residency/ContextFrame/KV separation | typed Projection, pure `yai.residency_plan.v1`, independent ContextFrame and distinct render identity are implemented; opaque continuation is optional and tokens/KV are absent from canonical state | semantic units and rendered-size estimation are conservative rather than tokenizer-authoritative; no ContextDelta consumer |
| provenance-bound memory | OperationalMemory remains derived; W19/H19 source-revalidate qualified BM25/exact-cosine retrieval; W20 adds Episodes, evidence-bound assertions and recorded-result consolidation rebuild through multi-family RetrievalSet v3 | ANN/learned reranking remain deferred; W20 generation-based retrieval retention is not universal deletion/privacy policy or general semantic paging |
| agentless long-horizon execution | synchronous Case runner repeatedly consumes canonical reality, derived memory/residency and the controlled effect boundary with explicit budgets/stops, typed human pause/resume, LMDB run admission and restart tests | generalized operation families, distributed admission and daemon scheduling are absent |
| provider replacement preserves semantic continuity | real HTTP Provider A→filesystem FINALIZE→Provider B, same-provider model replacement, continuation invalidation and provider restart are deterministic product tests; I05 adds bounded governed arbitration | learned/economic routing, general cold-state substitution and native state protocols are absent |
| derived data rebuilds from canonical state | graph and OperationalMemory rebuild from typed transitions; W19 corpus/index manifests are content-addressed, stale/corrupt-aware, atomically replaceable and add no LMDB DB; profile replacement creates an independent namespace | adaptive background scheduling, compression and full typed analytics inputs |
| governance source/artifact history | exact-byte source identity, typed deterministic parse/IR, immutable v5 Tenant ownership/validity and append-only authenticated lifecycle/revoke share LMDB while remaining independent from Cases | external organization/SSO assertion, retention policy and distributed revoke |
| Case policy configuration and admission | Tenant-safe exact artifacts bind canonically; current Transition v19 retains typed Decision/review/temporal/Grant lineage and historical readers and adds source-bootstrap relations; cancellation/closure are durable barriers and historical basis is never rewritten | broader authority consumers and externally authenticated principals; single-host scheduling and Golden resource effects already exist |
| authenticated Principal and Tenant isolation | kernel eUID projection, immutable Principal/Tenant catalog, Owner/Member admin checks, immutable Case Tenant, Principal/Participant links, Tenant-filtered reads and cross-Tenant filesystem-root alias rejection; shared-resource fencing is implemented | local OS trust only; no SSO/account directory, membership removal or general VM/container trust boundary |

These are current bounded contract limits, not a second live maturity registry.
The [Roadmap](../ROADMAP.md) alone owns generic maturity, programs and selection.
