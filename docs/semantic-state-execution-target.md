# Semantic cognitive state and execution compilation

Authority: **architectural target — not current implementation truth**.
[Roadmap](../ROADMAP.md) alone owns live maturity, programs and sequencing; [Constitution](constitution.md)
owns invariants; [Architecture](architecture.md) owns executable status. This
document freezes no schema, Rust type, module, protocol or storage owner.

## Status and claim vocabulary

- **IMPLEMENTED FACT** means a bounded source/test-backed property, with its
  evidence and limitations; it is not a universal production guarantee.
- **ARCHITECTURAL DECISION / TARGET** means an adopted direction or invariant,
  not a claim that every necessary mechanism exists.
- **HYPOTHESIS / RESEARCH PRESSURE** means a mechanism or scaling proposition
  that must earn implementation through a consumer and falsification.

I01–I06, TEST.TOPOLOGY.0, native REPLAI R4/R5, Golden and guided Case CLI are
historical implementation/evidence anchors, not a second status registry here.
**I01–I06 COMPLETE. I07 UNSELECTED.** REPLAI remains the sole native editor;
R5 vendor removal changed neither cognitive semantics nor its then-qualified
pin, and the later qualified repin is preserved. The formerly “post-I10” target
is organized by the Roadmap's semantic-state programs, not blocked on invented
I07–I10 implementations. Source work requires separate explicit authorization.

## Primitive and ownership — architectural decision

A conventional model harness commonly centers invocation or an agent loop:
input → context → model → tools → loop → output. This describes a common
organization of computation, not a claim that all harnesses own an Agent or a
judgment about their quality.

YAI instead centers **governed transformation of durable semantic and operational
state**. A model is one possible compute Participant. Agents, chats, Workflows,
humans and automated evaluations can participate; models, providers, files,
databases, endpoints, services, devices and other resources may be related to a
Case through the appropriate admitted contracts. These are architectural
possibilities, not a claim that every attachment/capability family exists today.
None becomes universal owner of continuity, memory, authority, resources,
historical state or execution truth.

YAI owns durable semantic continuity; model-visible context is a disposable
execution working set. System memory architecture is not one Memory database.
The following five concerns compose existing boundaries; they are not five new
source subsystems:

| Concern | Authority/ownership | Implemented foothold and target limit |
|---|---|---|
| Canonical history and owned objects | Transition Ledger owns Case history; content/artifact owners preserve non-reconstructible payload; external references retain exact provenance | I01 immutable content and I03 derived-content lineage exist; arbitrary external object families are not thereby implemented |
| Authoritative current materialization | CaseState is current admitted state for a generation, rebuildable from history, never an independent mutable truth | Lifecycle, Participants, logical bindings and current authority refs exist; no universal state bag is proposed |
| Derived semantic state | Graph, indexes, operational/episodic/semantic memory, retrieval and analytics remain rebuildable | W19/H19/W20 establish bounded access/derivation contracts, not an automatic semantic authority |
| Execution working state | Consumer-specific selection of intent, scope, authority/disclosure, dependencies, target capabilities and budget | Bounded S/W compiler with context compatibility exists; generic state paging and universal task sufficiency remain targets |
| Target realization | Adapters lower admitted semantic input into a target-native form; physical execution belongs below that boundary | Existing provider render and typed realization exist; persistent State Read/Update are adopted targets with unimplemented consumers, not current capabilities |

**State Fabric** names only this architectural composition. It is **not** a
database, daemon, canonical owner, registry, service, StateFabricStore or mutable
global WorkingMemory. A working copy of canonical intent, authority or content
does not change the authority of its source. Existing governance and security
histories remain with their established owners; a Case binds their exact facts.

## Historical experience / semantic / working / computational state — adopted target

**YAI owns semantic cognitive state. YVEX owns computational cognitive state.**
This extends the existing continuity boundary, not the set of canonical owners.
YAI's semantic state spans admitted objectives, facts/claims with their epistemic
class, obligations, unresolved state, resource/Workflow consequences and
relationships. It does not flatten CaseState, policy and derived memory into a
single mutable object. A human and a model participate through identity,
capability, scope and authority contracts; neither automatically owns the Case.

The adopted execution target has two streams: ordinary token/input/activation
computation and the **Experiential State Stream**. “Experiential” describes
computational state accumulated across ongoing execution and available beyond
immediate context. It implies no consciousness, emotion, identity or personhood.
The persistent stream has independently versioned/resident representation,
State Read and eventual State Update. Both capabilities are **OPEN target
requirements now**, not removed from the architecture because current consumers
are absent. Concrete mechanisms remain research pressure. Context-only execution
is a compatibility realization, not the final paradigm.

Current Semantic State is **not the whole of memory**. Qualified historical
experience H_t and what presently holds S_t are different views of established
owners. Neither introduces a second ledger or a universal semantic truth object.

```text
YAI: existing canonical history + immutable content + established owners
  ├── H_t: qualified historical experience
  └── S_t: current semantic state

H_t + S_t + question/scope/authority/budget → Recall → R_t^q
S_t + relevant R_t^q + task/Case constraints + authority/relevance/budget
  │ State Compilation
  ▼
W_t: bounded semantic working state
  │ public cognitive-state boundary
  ▼
YVEX: E_t = Lower(W_t, Model, StateProfile)
  │ State Read
  ▼
MODEL / Exec(X_t, E_t)
  ├── output Y_t
  ├── E_t+1 (computational loop)
  └── optional semantic P_t → YAI admission → H/S change
```

The target equations are:

```text
R_t^q = Recall(H_t, S_t, q, scope, authority/disclosure, budget)
W_t = Compile(S_t, R_t^q, active intent/task, Case big-picture constraints,
              authority/disclosure, relevance, execution budget)
E_t = Lower(W_t, Model, StateProfile)
Exec(X_t, E_t) → (Y_t, E_t+1, P_t optional)
```

The notation is not a source-type, schema or API commitment:

| State | Meaning | Ownership and limit |
|---|---|---|
| H_t — Qualified historical experience | Recorded occurrences, decisions, observations, effects, Workflow history and admitted material, including what no longer holds | Conceptual qualified view over existing ledgers/content/established owners; not a new historical owner or a guarantee of complete knowledge |
| S_t — Semantic State | What presently holds: model-independent Case meaning with provenance, authority and epistemic distinctions | YAI, through existing owners; current authority, derived assertion and model claim stay distinct |
| R_t^q — Recall Trace | Bounded reconstruction for question/task q through possibly discontinuous Events and Episodes | Derived, reconstructible, disposable; Participant/disclosure scoped, temporally coherent and causally coherent only where supported; exact source closure or explicit missingness |
| W_t — Semantic Working State | What must count for this execution: bounded, task-relative, Participant-scoped, provenance-carrying | Target compilation of current S plus relevant Recall, task and Case constraints; not the whole Case or a new memory history |
| E_t — Experiential Computational State | Model-native realization of W_t, subsequently accumulated through execution | Computationally owned by YVEX/model: model-specific, potentially opaque, independently resident, derived and replaceable; never a second semantic ledger |

q identifies the current question/task; X_t is immediate input. R_t^q denotes
Recall, not a relevance score (the earlier notation used R_t for relevance).
Compilation must explain what matters now, for whom, under which scope/budget
and from which sources. Relevance cannot override disclosure. Case age does not require the
entire Case to be lowered; boundedness cannot excuse dropping necessary state.
YVEX need receive only qualified W_t/input, not all of H_t or S_t. Provisional
SemanticStateFrame/Delta describe W_t representation/changes, not latent layouts.
**IMPLEMENTED FACT:** the bounded source implementation composes qualified
SemanticState and compiles SemanticWorkingState before Projection/ContextFrame
lowering. This is not a claim
of general temporal/as-of reconstruction, a Recall Compiler, universal task
sufficiency, semantic paging or a public E_t consumer. The existing Rust
`SemanticState` wraps history and current materialization for qualification;
the H/S target distinction does not rename/split that contract retroactively. See
[current architecture](architecture.md#current-provider-and-context-behavior).

### Independent computational and semantic evolution

Computational loop: **E_t → execution → E_t+1 → subsequent execution**.
Lower describes initialization/reconstruction; Exec may evolve E_t frequently
without recompiling W_t or creating semantic events. E_t → E_t+1 does not imply
S_t → S_t+1. A model may eventually write its computational stream without
permission to write canonical Case meaning.

Semantic loop: **explicit model/human proposal P_t or observed external
consequence → typed normalization → evidence/authority/policy/admission →
admitted history/current consequence → H_t+1 / S_t+1** through the applicable
owner (CaseState advances only through Case Transitions). P_t is optional and
distinct from both output Y_t and computational E_t+1. Internal state updates
do not become proposals by default; ordinary prose does not bypass typed normalization. Admission may
refuse a proposal, including one repeated or retained in E_t.
No proposal means no semantic mutation from that internal update, not omission
of required governed Invocation/Result/effect evidence. Recording execution is
distinct from admitting its proposed consequence; external observations and
human actions still use their applicable owners, not a mandatory model path.

Changing DeepSeek to Qwen may invalidate E_t, never admitted H/S continuity.
Reconstruction resolves relevant R_t^q and required W_t under current intent,
authority, relevance and budget, then realizes a new exact model/profile's E_t.
Semantic recovery does not promise identical activations, stochastic output or
preservation of unadmitted computation. E_t cannot be the sole copy of an admitted Case fact.

YVEX owns model/deployment truth, computational capabilities, layouts,
KV/recurrent/SSM/latent physical state, State Read/Update realization,
paging/residency, checkpoints, rollback/invalidation and kernel/resource
evidence. These are target ownership assignments, not assertions about a
currently published YVEX ABI. YAI owns semantic selection, working-state/delta
meaning and semantic admission. YVEX gains neither whole-Case visibility by
implication nor Case, Policy, semantic-memory or Workflow authority; YAI gains
no tensors, latent banks, layer layouts or GPU placement.
The model supplies learned computation, not authority to commit its output.

## Experience, temporal meaning and Recall — adopted target

An **Event/occurrence** is an individual recorded occurrence, observation or
admitted change; recording a claim does not prove the claimed event happened.
An **Episode** is a derived structured grouping of related Events/experience.
S describes what currently holds. A Recall Trace reconstructs what matters to q
across Events/Episodes, potentially far apart in time; W selects what counts
for computation, and E realizes that working state below YAI. W20's bounded
Episodes and SemanticAssertions remain derived and rebuildable, including
recorded-result normalization without re-inference. Those facts do not establish
the general target Recall process.

Temporal meaning must distinguish **event time**, **observation time**,
**admission/recording time**, **validity interval**, and **supersession/invalidation
time**. Unknown times remain unknown. Canonical reducer order remains its
existing sequence, not an inferred event timeline. Late observation or admission
must not rewrite what was recorded at an earlier point.

“What currently holds?”, “What did the Case record/believe at time u?”, “What
happened before X?”, “What supported Y then?” and “What later contradicted or
superseded Y?” require different temporal queries. Historical access uses
**current** disclosure authorization: a historical permission is not permission
to read now. A superseded assertion can be valid historical Recall material
without remaining current truth. Lack of evidence must surface as missingness,
not a completed narrative.

Chronological order is not causality. Temporal order, dependency, support,
contradiction, supersession, causal contribution and provenance are relation
classes, not a frozen exhaustive enum. Each relation retains its exact basis,
time/validity and epistemic posture. A model-inferred causal link is not an
observed or structurally established link, even when traversal finds it useful.

### Qualified recall process, not another memory subsystem

The target Recall Compiler/process extends derived semantic access:

```text
task/query → exact/current anchors → candidate discovery
    → temporal / causal / exact / lexical / vector / learned access paths
    → bounded path expansion
    → identity + Case/Tenant/Participant + current disclosure revalidation
    → validity / supersession checks for the requested temporal meaning
    → exact source closure → bounded Recall Trace
```

Authorization also constrains candidate discovery; final revalidation does not
excuse exposing hidden candidates, counts or relationships during search.
Choose the strongest available access path. Exact current-state resolution
must not degrade to vector search. Budget path depth, source count and material;
retain inclusion/omission and incompleteness posture. A Trace is not top-k
retrieval, `summary.md`, or a model-generated story promoted into truth.
Failure of derived accelerators requires qualified reconstruction/fallback or
honest refusal, not invented experience. No EpisodeStore, RecallStore,
MemoryStore, ExperienceStore or StateFabricStore follows from these concepts.

### Associative navigation and exact backing

The target has two complementary paths: a **fast associative/latent path** for
recognition/relevance and a **slow exact path** for precise values, identities,
facts and historical evidence. Compression cannot become the sole backing for
exact claims when exact sources exist. Trace closure must resolve the relevant
Transition/content identity, artifact version/digest, observation/effect proof,
Decision/provenance or historical resource evidence, or explicitly report the
missing source. A current mutable endpoint is not evidence of its past bytes.
“Source closed” asserts identity/support closure, not omniscience or the truth
of every recorded claim.

Learned memory navigation is research within M/S/C/Q, not a new program. It may
propose candidate Episodes, associations, traversal paths, ranking/search priority
and relevance. It may not decide truth, authority, disclosure, current validity,
supersession, canonical causality or exact source identity. A deterministic
resolver revalidates every selected source. Graph, vector index, BM25,
embeddings, SAE features and learned rankers remain derived access paths.

Sparse Autoencoders (SAEs) are optional experiments, not dependencies.
YAI-side sparse/semantic representations could supply an associative feature
vocabulary. Model-activation SAEs are separate model/YVEX-side research
instrumentation for State Read. **Z_semantic != E_model**: YAI vectors are not
a model residual stream. Any mapping needs an explicitly trained and qualified
model-side bridge; this assigns no adapter/training ownership to YAI.

## Continuous feedback and refresh — adopted target, not a background runtime

“Continuous” means cognitive continuity/state refresh need not wait for a new
human prompt. It does not mean infinite execution, repeated LLM calls without
authorized work, automatic semantic mutation or an uncontrolled Agent loop.

| Loop | Trigger and consequence | Authority boundary |
|---|---|---|
| Semantic / world | Action, observation or explicit proposal → normalization, evidence, authority/admission → H/S change | Existing typed owners only; CaseState remains Transition-mediated |
| Experience assimilation / recall | Admitted H/S change → rebuild/update derived experience/access structures → affected Recall may change | Derivation cannot invent or commit history |
| Working-state refresh | Relevant ΔH / ΔS / ΔRecall → qualified ΔW or full W recompilation | Fresh scope/source identity, mandatory state and sufficiency must survive |
| Computational reconciliation | Full W / qualified ΔW → YVEX Lower/Reconcile → refreshed E | Public capability and compatibility evidence; YAI does not write latent state |
| Model computational | E plus current computation → E′ | Internal updates may be frequent; no automatic semantic Transition |
| Semantic return | Explicit P or observed consequence → YAI admission → H/S change | Repetition of E, Recall or a learned association never grants authority |

**E_t → E_t+1 does NOT imply S_t → S_t+1.** Neither assimilation nor refresh
is a new canonical mutation path. Current full-recompilation delta equivalence
does not qualify these complete prompt-independent loops.

Relevant tool/result completion, Workflow progression, admitted observation,
review outcome, policy/revocation, source/artifact change, effect reconciliation
or another Participant's admitted action may trigger refresh before another
human prompt. Raw external drift remains a candidate/invalidating signal under
its owner contract, not an automatically admitted new fact.

Two refresh classes have different correctness contracts:

- **Correctness-critical:** invalidate/recompile before the next affected
  State Read. Authority/disclosure revocation, Participant/Tenant/scope change,
  supersession affecting required state, or incompatible model/profile must
  prevent stale consumption; no “eventual” update can excuse unauthorized reads.
- **Cognitive enrichment:** qualified derived work may be asynchronous with
  explicit bounded staleness and budget, without weakening the first class.

If forbidden material may already be embedded in E and no qualified selective
removal exists, invalidate and reconstruct the affected realization (and any
dependent in-progress computation). Do not pretend to delete a feature from
latent state. Public invalidation/acknowledgement semantics need qualification;
none is claimed implemented by this doctrine.

## Persistent deliberation — research target

An assignment such as “think about this until tomorrow and bring justified new
conclusions” needs persistent **objective, Participant, scope, allowed semantic
sources, resources/effects if any, authority, compute/time budget, deadline,
output/result contract, cancellation and termination posture**. YAI owns those
assignment semantics, preferably through existing task/Workflow/authority and
execution owners. No ThinkingJobStore or autonomous authority is justified.

Distinguish internal deliberation (no environment action), deliberation with
qualified Recall (additional scoped experience), and operational verification
(real resources/effects through ordinary Decision/Grant/PREPARE/reconciliation).
The internal thinking loop is a model/runtime computational capability; an
Agent is an optional application composition above it, not its state owner.
Authorized work must stop/cancel or refuse honestly when its contract cannot
be satisfied. No current long-running/overnight thinking capability is claimed.

**L_{t,k}** denotes unfinished latent/in-progress deliberation for an assignment
at step k; **E_t** denotes reusable experiential computational state. They may
share future physical mechanisms but have different lifecycles. No Rust type or
wire schema is prescribed. YAI does not require raw hidden chain-of-thought text
as durable memory. Useful surfaced candidates may be conclusions, hypotheses,
contradictions, unresolved questions, requested evidence, progress/checkpoint
metadata or final proposals; their typed admission and epistemic class remain
explicit. Internal reasoning stays computational unless surfaced by contract.

| Continuity | Survives / may be lost |
|---|---|
| Semantic | Admitted H/S survives model loss/replacement; relevant R/W reconstructs from qualified sources |
| Experiential computational | E may persist/resume only with exact model/profile compatibility; replacement can require recompilation |
| Deliberation | L may checkpoint/resume when compatible; cold replacement may lose the exact unfinished trajectory while preserving the admitted assignment and H/S |

Consolidation likewise has three meanings: **semantic** derivation of Episodes,
relations, associations and summaries with provenance; **computational**
reorganization of model-native E for reuse; **parametric** changes to
model/encoder/adapter weights through training. None implies another. Parameter
updates do not automatically write Case memory. Training and latent execution
remain model/runtime work, not YAI stores or implementation requirements.

## Admission and payload — architectural decision with I01 evidence

Everything *durably admitted* needs identity, provenance and semantic relation.
This is not a requirement to serialize every byte into the Transition Ledger,
nor to canonize temporary drafts, microphone buffers or editor keystrokes.
I01 demonstrates the split: immutable ConversationContent owns bytes, while
Transition history owns submitted Turn/reference/provenance meaning. The
original import path is not durable identity. Original content cannot be
reconstructed merely by rebuilding the ledger's materializations.

The same ownership test must remain available for files, datasets, checkpoints,
media, large artifacts and future externally stored objects. External reference
durability does not promise that someone else's bytes remain available; identity,
integrity, retention and unavailable-source posture need an explicit contract.
Conversation content must not be disguised as a controlled ResourceAttachment
or automatically promoted to memory truth. The ledger is not a blob warehouse.

## One Case, scoped interaction views — decision/target

REPLAI interaction, a future Studio view, API interaction, Codex execution,
human review, automated evaluation and future Agent views can operate over the
same Case. This lists possible views, not seven implemented clients. Cross-chat
continuity is not copying memory between chat owners: each surface resolves
the same governed continuity under its own identity, scope and disclosure.
Thread organization does not partition authority or grant visibility.

Current threads become durably observable through committed Turns; an empty
controller-local thread need not survive restart. No canonical Space, ChatStore
or persistent empty-chat lifecycle is justified here. REPLAI owns terminal
mechanics; YAI owns SEND, intent, admission and execution semantics.

A future product-level Agent may compose Participant, cognitive role, bindings,
working-state profile, authority/disclosure, allowed capabilities/resources,
optional Workflow and budgets. It gains no ownership of Case, memory, model,
tools, resources, database, authority or history by being named Agent.

## Context locality — target and falsifiable hypothesis

The context window is the last model-visible level of an execution working-set
hierarchy, not YAI's durable memory. The desired scaling relationship is:

```text
Undesirable default: ContextSize ∝ CaseHistoryLength
Target:              ContextSize ∝ ActiveWorkingSet(CurrentTask)
```

This is a target, not an established asymptotic law or current universal
guarantee. Large context is valid when the task genuinely needs it. The aim is
not an arbitrary token ceiling, but to stop Case age being the default driver
of model-visible state. Compare task sufficiency and semantic correctness as
well as bytes/tokens; a tiny context that omits required evidence fails.
Host-side rebuild/query cost may still grow with history and must be measured
separately from model-visible context size.

## Working-state compilation — target; representation is provisional

Current SemanticState/SemanticWorkingState contracts provide bounded qualified
composition and selection, followed by context-compatible Projection/ContextFrame.
They are not the definition of memory. The target program still extends toward:

```text
Current S + relevant qualified Recall from H + intent/task + Case constraints
    + authority/disclosure + relevance + active dependencies + execution budget
        ↓
Working-state compilation
        ↓
W_t: provider-independent active semantic working state
        ↓
Public cognitive-state boundary → YVEX lowering → E_t
```

`SemanticStateFrame` (earlier discussion used `ExecutionFrame`) remains a provisional
generalized representation, not a fixed future wire schema. Current bounded W and
derived SemanticDelta v1 are implemented internal contracts; delta application
explicitly falls back to full recompilation. They neither mutate CaseState nor
implement computational State Update. Future contracts must not duplicate the
current semantic-state compiler or existing admission owners.
Typed capability requirements can constrain compilation without placing a model
family, wire protocol, session ID or physical deployment into semantic identity.
Evidence must decide whether existing contracts evolve or a distinct contract
is justified; no additional persistent owner is presumed.

Possible lowerings include ordinary messages/context, typed multipart input,
opaque continuation plus semantic delta, recurrent checkpoint plus input delta,
or future native state/memory handles. Only current text/typed adapter and
optional continuation contracts are implemented footholds. Persistent State
Read/Update are adopted targets; their concrete lowering mechanisms remain
hypotheses, not promised adapters. Every lowering remains subject
to exact target qualification and honest refusal. Context compilation cannot
grant permission to execute operations.

**Persistent KV alone != cognitive State Read.** This is not a prohibition on
KV as a physical realization. Intentional YAI semantic-working-state compilation
→ qualified YVEX persistent-KV/prefix lowering → actual model reuse is a possible
bounded first State Read experiment, including without new training. It must
prove source/scope provenance, exact model/profile compatibility, actual reuse
and invalidation/reconstruction. A cache existing, a prefix being retained or a
model being stateful by name proves none of those semantics automatically.
The experiment remains unqualified, not an implemented YAI/YVEX contract.

YAI should be model-aware through typed capability contracts and model-independent
in semantic ownership. Transformer, SSM, RWKV, Mamba and future architecture
names are not semantic dispatch rules. YVEX/provider owns physical engines,
sessions, placement, KV/recurrent state and computational evidence. Future
checkpoint/native-state forms remain derived computational state with a
semantic reconstruction route, not new Case authority. Their target role is
broader than caching ordinary context. No private YVEX protocol
or unobserved provider capability is specified here.

### Realization ladder — research options under the same boundary

| Mode | Conceptual realization | Required distinction |
|---|---|---|
| Context compatibility | W_t rendered/projected into ordinary input; prefill reconstructs computational state | Current bounded S/W compilation and context lowering are implemented; no experiential stream or universal task-sufficiency claim |
| Persistent context-derived state | W_t lowered into a reusable prefix/KV/hidden realization | Training-free experiment; exact source/scope/profile, observed reuse and invalidation need evidence |
| Experiential State Read | Independently persistent model-native E_t explicitly consumed by the model | Persistence must actually carry qualified working-state meaning, not merely exist |
| Experiential State Read + external update | YVEX incrementally updates E_t from qualified semantic working-state deltas | Current scope, supersession and full-reconstruction equivalence must survive; no YAI tensor writes |
| Learned Read/Write | A trained state pathway reads and updates E_t | Model-side learning must be independently qualified; computational write does not admit P_t |
| Native dual-stream | Model trained from inception for immediate computation plus persistent computational state | Future compatibility hypothesis, not a claim about any current model |

These are overlapping realization/evaluation classes, not six mandatory releases.
A qualified persistent-prefix/KV experiment can be a bounded State Read consumer
without training; an implementation need not traverse every class in order.
Possible E_t mechanisms include a persistent prefix/KV, hidden-state bank,
cross-attention memory, gated latent state, recurrent state, dual residual/state
pathway or architecture-native persistent memory. They are YVEX/model-side
realizations, not semantic categories YAI must inspect or branch on.

### State-augmented pretrained models — hypothesis, not YAI training

New pretraining is not a prerequisite imposed by this architecture. One future
experiment could instead use:

```text
pretrained backbone + model-state adapter + exact StateProfile
    + targeted post-training → state-augmented model
```

The backbone may initially remain frozen. A model-state adapter could contain
a State Read projection, gating, cross-state attention and an optional State
Update/write mechanism. Whether that realizes useful persistent state must be
proven; these are not promised algorithms, artifacts or source modules.
YAI owns none of the adapter, weights, training or latent representation.
Its research contribution is qualified W_t and semantic supervision/evaluation
structure. YVEX/model realizes the computational contract and exposes truthful
public capabilities or refusal; no private interface is invented here.

Cases and Golden may provide derived, authority-scoped semantic trajectories:

```text
(H_t, S_t) → R_t^q → W_t + input/task
    → independently admitted consequences → (H_t+1, S_t+1)
```

Compare required big-picture retention, rejection of irrelevant experience,
supersession, task switching without loss of constraints, long-history locality
and correct semantic proposals. Record exact source/generation/scope and
model/adapter/profile evidence where applicable. Do not treat all admitted
records as factually correct labels: observations, inferences and proposals
retain their epistemic class, and an independent task oracle remains required.
This is evaluation/research infrastructure, not a training-dataset owner,
canonical database or permission to disclose private Case trajectories.

### Multi-timescale semantic state — OPEN target

Distinguish possible immediate/local, task/subgoal, Case/big-picture and
slow/stable and episodic-recall semantic roles. YAI may classify and select relevant material
across those timescales; no tensor bank, layer partition, retention duration or
computational update schedule follows automatically. YVEX/model decides how
qualified W_t is realized. Slow state can still be superseded; persistence or
age confers no authority. No tested general timescale-selection contract is
claimed by the present memory/residency footholds.

## Scoped state access — research pressure

Investigate a bounded initial working set: objective, constraints, critical
current state and available semantic references. A consumer could then request
exact state resolution, expansion or causal tracing rather than receiving
every potentially relevant object up front. These are design questions, not new
model tools, capability APIs or operational read authority.

Any semantic handle must be a typed/scoped reference whose resolution validates
identity, Case/Tenant/Participant, generation, disclosure and applicable authority.
Stale or unauthorized expansion returns a typed refusal or a newly qualified
view. Bound expansion count, size and dependency depth; preserve provenance and
record what was supplied when execution evidence requires it. No URI syntax,
universal handle store or ambient database access is frozen.

Choose the strongest available access path, not vector similarity by default:

| Question | Appropriate basis |
|---|---|
| Exact current policy/state or a historical Decision | Deterministic current relations and exact artifacts; a historical ALLOW never authorizes current work |
| Relevant causal history | Typed causal traversal revalidated against canonical sources |
| Semantically similar historical material | Qualified lexical/semantic/vector retrieval, retaining epistemic class and lifecycle |
| Chronological reconstruction | Ordered Transition history and temporal projections |

Graph, vector index, BM25, embeddings and narrative summaries are not canonical
memory. W20 semantic assertions also remain derived, evidence-bound claims.
Similarity, repeated retrieval or provider agreement never inflate epistemic
class; semantic search is not current policy resolution.

## Model-proposed state deltas — target, never direct writes

Computational State Update and semantic update are different kinds of change
in different ownership domains. Exec may produce E_t+1 without P_t; YAI does not
canonize each latent/prefix/recurrent micro-variation. When an explicit P_t is
present, the intended semantic path is:

ProviderResult / explicit P_t → interpretation/proposed typed delta → typed validation →
applicable authority/policy/evidence closure → admitted Transition → new
CaseState is the intended admission pattern. Recording output proves that it
was returned, not that its assertions are true. An effect still requires the
existing Operation/Decision/Grant and effect/reconciliation boundaries; admission
to history alone is not permission. An inference remains inference.

Existing operational proposal normalization and W20 semantic consolidation are limited
footholds, not a generic mutation language. Future implementation must establish actual
producers, consumers, scope and failure semantics before adding any delta API.
There is no automatic model-to-memory or model-to-state write path implied.

## Architectural falsifiers — future acceptance, not current passes

| Evaluation | Controlled setup and observation | Falsifier |
|---|---|---|
| Whole Case versus working state | Compile W_t for one task/Participant from a mature S_t; independently check necessity, relevance, scope, budget and source provenance | The whole Case is lowered automatically, unauthorized state leaks, or boundedness hides missing required evidence |
| Cold-model substitution | Mature Case, different qualified exact model/profile, invalidate incompatible E/L and reconstruct relevant R/W and new E, zero inherited continuation or full-history injection | Admitted H/S is lost; loss of the exact unfinished L trajectory alone is not semantic discontinuity |
| Case-age locality | Vary history length radically while holding current state, task and disclosure equivalent; measure sufficient working-set entries/units and actual tokens where available | Model-visible context grows linearly with unrelated history, or stays small only by losing required evidence |
| Task/local versus big-picture | Switch task/subgoal while preserving relevant Case, slow/stable and episodic constraints; change or supersede material during active deliberation | Wrong-memory interference dominates, superseded facts retain current authority or task switching loses constraints |
| Full working-state reconstruction versus delta | Compare semantic meaning, provenance, scope and dependencies of full W_t compilation and qualified delta application, including removal and restart | Stale or superseded material survives, source closure differs or only a byte-identical latent state is checked |
| Derived-state amnesia | Snapshot canonical history/current replay and owned-object identities; drop only disposable graph/index/retrieval/memory artifacts, rebuild with pinned algorithms and recorded results | Canonical truth changes, original payload is lost, or semantic rebuild silently requires fresh inference; re-encoding cost is accounted separately |
| Computational-state and deliberation amnesia | Through a qualified operator/provider boundary separately lose/resume E and L; preserve H/S, reconstruct relevant R/W and requalify the assignment | Admitted semantics depend on latent state, incompatible checkpoints resume, or lost unfinished thought is falsely claimed preserved |
| Bounded persistent-KV State Read | Compile scoped W_t, lower it into an exact model/profile's persistent prefix/KV, observe actual reuse, then invalidate/rebuild under changed scope/model | Mere cache persistence is called semantic qualification, reuse is not demonstrated, or hidden/outdated state bypasses W_t admission |
| State-augmented pretrained read | Compare exact backbone/adapter/profile with controlled W_t/E_t access, including targeted post-training and a frozen-backbone option | Model ignores supplied state, uses unrelated/leaked material or appears correct only through full prompt history; no read qualification follows from adapter existence |
| Learned update without semantic proposal | Exec changes E_t with P_t absent; separately submit valid/invalid explicit P_t under current authority | Every computational micro-update requires a semantic Transition, or E_t/Y_t admits a fact without semantic validation; required execution evidence must still be recorded |
| Discontinuous Recall / as-of reconstruction | Long Case, few widely separated relevant Episodes, many similar distractors, late evidence and a superseded conclusion; ask what held and what was recorded at distinct times | Wrong trajectory, unsupported causal edges, historical material used as current truth, missing exact source closure hidden, or unbounded W; exact backing and explicit missingness controls required |
| Prompt-independent refresh | Complete tools/review/Workflow, admit observations and revoke disclosure without another prompt; compare full W with qualified refresh and future E reconciliation | Next affected State Read uses stale/forbidden material, or enrichment is treated as canonical admission; no qualified removal means invalidate/rebuild |
| Persistent deliberation usefulness | Same assignment and total compute/time budget, with/without memory and persistent deliberation; include State Read ablation, distractors, interruption and cancellation | No benefit over equal-compute controls, silent budget overrun, invented conclusions or side effects outside the normal governed path |
| State-page locality | Start bounded, request one relevant temporal/causal/state region under explicit scope and expansion budget | Full-history replay into model context is required, expansion is unbounded or references bypass disclosure |
| Authority isolation | Same Case and task, different admitted Participants/disclosure scopes; include unauthorized, stale and cross-Tenant references | Hidden content/counts leak, scopes are merged, or a handle grants ambient shared-state access instead of refusal |

These evaluations must measure admitted semantics, not only the model's prose.
Choose concrete task oracles, workload bounds and failure thresholds before
claiming a pass. W19/H19/W20 rebuild tests, provider-substitution tests and I06
recovery are positive controls, not proof of the entire future matrix.
TEST.TOPOLOGY.0 still separates proof class from provider mode: loopback can
prove transport/admission/recovery, not live YVEX interoperability, semantic
quality or universal context locality. Publication must label missing external
evidence rather than substitute a fixture.

Qualification must measure more than retrieval accuracy: recall precision and
completeness, temporal ordering, relation correctness against its declared
epistemic basis, supersession/current-versus-historical correctness, exact source
closure, recall/assimilation/W-refresh latency and stale-read/invalidation posture.
Compare task success with/without memory, wrong-memory interference, State Read
ablation and persistent deliberation against equal-compute baselines. Declare
oracles and missing/unsupported causal evidence; no thresholds or results are
fabricated here. Latency components and semantic correctness are separate axes.

The learned-read/write, Recall and trajectory tests remain unqualified hypotheses, not
new benchmark results. Computational-state deletion is performed only through
an authorized provider/operator contract; this document does not permit YAI to
administer producer internals or discard an operator's retained state.

## Program boundary

The Roadmap alone selects boundaries and dependency horizons. This document
retains target meaning and falsifiers, not a second execution queue. Adopted
State Read/Update remain distinct from hypothetical implementations. No schema,
H20/W21/W22, REPLAI change, Studio or runtime refactor is authorized by this
documentation.

The bounded S → W compiler is implemented; universal sufficiency, semantic paging
and optimized incremental compilation are not. E_t is not implemented inside YAI;
no current model is claimed to possess a native experiential stream. No
state-augmented training, model-state adapter or latent-state owner is introduced.
The source refoundation covers explicit qualified semantic-state meaning, bounded
W_t compilation, semantic full/delta equivalence and context compatibility, not
E_t in YAI. Current execution selection and maturity remain Roadmap-owned.

General temporal-causal memory, historical/as-of reconstruction, a Recall
Compiler, learned navigation, SAE memory, prompt-independent E reconciliation,
persistent internal deliberation and autonomous overnight thinking are **not
implemented claims**. Existing bounded replay, W20 and runtime loops do not
establish them. No trained adapter or looped/recurrent Transformer is claimed.
“Second residual” and recurrent/looped computation are possible model-side
research mechanisms, not YAI architecture names or requirements; no named model
is assigned such a design without qualified external evidence.
