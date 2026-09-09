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

## Semantic / working / experiential computational state — adopted target

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

```text
YAI: canonical history/current state → S_t
    → Compile(S_t, I_t, A_t, R_t, B_t) → Semantic Working State W_t
    → public cognitive-state boundary
YVEX: E_t = Lower(W_t, Model, StateProfile)
    → Experiential Computational State: model-native, resident, replaceable
    → State Read → Exec(X_t, E_t)
        ├── Y_t: non-authoritative output
        ├── E_t+1: State Update → subsequent computational execution
        └── P_t optional: explicit semantic proposal
              → YAI normalization + evidence/authority/policy + admission → S_t+1
```

The target equations are:

```text
W_t = Compile(S_t, I_t, A_t, R_t, B_t)
E_t = Lower(W_t, Model, StateProfile)
Exec(X_t, E_t) → (Y_t, E_t+1, P_t optional)
```

The notation is not a source-type, schema or API commitment:

| State | Meaning | Ownership and limit |
|---|---|---|
| S_t — Semantic State | Durable model-independent Case meaning with provenance, authority and epistemic distinctions | YAI, through existing canonical and derived owners; not a universal state store or promotion of every derived claim |
| W_t — Semantic Working State | What must count for this execution: bounded, task-relative, Participant-scoped, provenance-carrying | Compiled by YAI from S_t; derived working state, not another semantic history or the whole Case by default |
| E_t — Experiential Computational State | Model-native realization of W_t, subsequently accumulated through execution | Computationally owned by YVEX/model: model-specific, potentially opaque, independently resident, derived and replaceable; never a second semantic ledger |

I_t is current intent, A_t Participant authority/disclosure, R_t relevance to the
task/dependencies, B_t the execution budget. X_t is immediate input. Compilation
must explain what matters now, for whom, under which scope/budget and from which
sources. Relevance cannot override disclosure. Case age does not require the
entire Case to be lowered; boundedness cannot excuse dropping necessary state.
YVEX need receive only qualified W_t/input, not all of S_t. Provisional
SemanticStateFrame/Delta describe W_t representation/changes, not latent layouts.
The bounded source implementation now composes qualified SemanticState and compiles
SemanticWorkingState before Projection/ContextFrame lowering. This is not a claim
of universal task sufficiency, semantic paging or a public E_t consumer; see
[current architecture](architecture.md#current-provider-and-context-behavior).

### Two evolution loops, two admission meanings

Computational loop: **E_t → execution → E_t+1 → subsequent execution**.
Lower describes initialization/reconstruction; Exec may evolve E_t frequently
without recompiling W_t or creating semantic events. E_t → E_t+1 does not imply
S_t → S_t+1. A model may eventually write its computational stream without
permission to write canonical Case meaning.

Semantic loop: **explicit model/human proposal P_t or observed external
consequence → typed normalization → evidence/authority/policy/admission →
admitted Transition → S_t+1**. P_t is optional and distinct from both output Y_t
and computational E_t+1. Internal state updates do not become proposals by
default; ordinary prose does not bypass typed normalization. Admission may
refuse a proposal, including one repeated or retained in E_t.
No proposal means no semantic mutation from that internal update, not omission
of required governed Invocation/Result/effect evidence. Recording execution is
distinct from admitting its proposed consequence; external observations and
human actions still use their applicable owners, not a mandatory model path.

Changing DeepSeek to Qwen may invalidate E_t, never the Case. Recompilation
selects required W_t from S_t under current intent, authority, relevance and
budget, then realizes a new exact model/profile's E_t. Semantic recovery does
not promise identical activations, stochastic output or preservation of
unadmitted computation. E_t cannot be the sole copy of an admitted Case fact.

YVEX owns model/deployment truth, computational capabilities, layouts,
KV/recurrent/SSM/latent physical state, State Read/Update realization,
paging/residency, checkpoints, rollback/invalidation and kernel/resource
evidence. These are target ownership assignments, not assertions about a
currently published YVEX ABI. YAI owns semantic selection, working-state/delta
meaning and semantic admission. YVEX gains neither whole-Case visibility by
implication nor Case, Policy, semantic-memory or Workflow authority; YAI gains
no tensors, latent banks, layer layouts or GPU placement.
The model supplies learned computation, not authority to commit its output.

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
Canonical / derived Case state + current intent + authority/disclosure
    + relevance + target capabilities + active dependencies + execution budget
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
S_t → W_t + input/task → independently admitted consequences → S_t+1
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
slow/stable semantic classes. YAI may classify and select relevant material
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

Existing operational proposal normalization and W20 consolidation are limited
footholds, not a generic mutation language. Future implementation must establish actual
producers, consumers, scope and failure semantics before adding any delta API.
There is no automatic model-to-memory or model-to-state write path implied.

## Architectural falsifiers — future acceptance, not current passes

| Evaluation | Controlled setup and observation | Falsifier |
|---|---|---|
| Whole Case versus working state | Compile W_t for one task/Participant from a mature S_t; independently check necessity, relevance, scope, budget and source provenance | The whole Case is lowered automatically, unauthorized state leaks, or boundedness hides missing required evidence |
| Cold-model substitution | Mature Case, different qualified exact model/profile, invalidate old E_t, recompile W_t/E_t, zero inherited continuation or full-history injection | Correct continuation requires old E_t, unqualified guesses, Case recreation or dumping history |
| Case-age locality | Vary history length radically while holding current state, task and disclosure equivalent; measure sufficient working-set entries/units and actual tokens where available | Model-visible context grows linearly with unrelated history, or stays small only by losing required evidence |
| Task/local versus big-picture | Switch task/subgoal while preserving relevant Case and slow/stable constraints; change or supersede previously relevant material | Irrelevant experience dominates, superseded facts retain authority or task switching loses required constraints |
| Full working-state reconstruction versus delta | Compare semantic meaning, provenance, scope and dependencies of full W_t compilation and qualified delta application, including removal and restart | Stale or superseded material survives, source closure differs or only a byte-identical latent state is checked |
| Derived-state amnesia | Snapshot canonical history/current replay and owned-object identities; drop only disposable graph/index/retrieval/memory artifacts, rebuild with pinned algorithms and recorded results | Canonical truth changes, original payload is lost, or semantic rebuild silently requires fresh inference; re-encoding cost is accounted separately |
| Computational-state amnesia | Through a qualified operator/provider boundary discard E_t, preserve S_t and reconstruct task-sufficient W_t/E_t; include current continuation-loss controls | Admitted Case semantics depend on lost computational state; efficiency degradation or unadmitted computation loss alone is not failure |
| Bounded persistent-KV State Read | Compile scoped W_t, lower it into an exact model/profile's persistent prefix/KV, observe actual reuse, then invalidate/rebuild under changed scope/model | Mere cache persistence is called semantic qualification, reuse is not demonstrated, or hidden/outdated state bypasses W_t admission |
| State-augmented pretrained read | Compare exact backbone/adapter/profile with controlled W_t/E_t access, including targeted post-training and a frozen-backbone option | Model ignores supplied state, uses unrelated/leaked material or appears correct only through full prompt history; no read qualification follows from adapter existence |
| Learned update without semantic proposal | Exec changes E_t with P_t absent; separately submit valid/invalid explicit P_t under current authority | Every computational micro-update requires a semantic Transition, or E_t/Y_t admits a fact without semantic validation; required execution evidence must still be recorded |
| State-page locality | Start bounded, request one relevant causal/state region under explicit scope and expansion budget | Full-history replay into model context is required, expansion is unbounded or references bypass disclosure |
| Authority isolation | Same Case and task, different admitted Participants/disclosure scopes; include unauthorized, stale and cross-Tenant references | Hidden content/counts leak, scopes are merged, or a handle grants ambient shared-state access instead of refusal |

These evaluations must measure admitted semantics, not only the model's prose.
Choose concrete task oracles, workload bounds and failure thresholds before
claiming a pass. W19/H19/W20 rebuild tests, provider-substitution tests and I06
recovery are positive controls, not proof of the entire future matrix.
TEST.TOPOLOGY.0 still separates proof class from provider mode: loopback can
prove transport/admission/recovery, not live YVEX interoperability, semantic
quality or universal context locality. Publication must label missing external
evidence rather than substitute a fixture.

The learned-read/write and trajectory tests remain unqualified hypotheses, not
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
