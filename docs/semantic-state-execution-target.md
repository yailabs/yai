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
| Execution working state | Consumer-specific selection of intent, scope, authority/disclosure, dependencies, target capabilities and budget | Projection/Residency/ContextFrame and I02–I06 supply parts; a general working-state compiler is a target |
| Target realization | Adapters lower admitted semantic input into a target-native form; physical execution belongs below that boundary | Existing provider render and typed realization exist; persistent State Read/Update are adopted targets with unimplemented consumers, not current capabilities |

**State Fabric** names only this architectural composition. It is **not** a
database, daemon, canonical owner, registry, service, StateFabricStore or mutable
global WorkingMemory. A working copy of canonical intent, authority or content
does not change the authority of its source. Existing governance and security
histories remain with their established owners; a Case binds their exact facts.

## Semantic / computational state — adopted target

**YAI owns semantic cognitive state. YVEX owns computational cognitive state.**
This extends the existing continuity boundary, not the set of canonical owners.
YAI's semantic state spans admitted objectives, facts/claims with their epistemic
class, obligations, unresolved state, resource/Workflow consequences and
relationships. It does not flatten CaseState, policy and derived memory into a
single mutable object. A human and a model participate through identity,
capability, scope and authority contracts; neither automatically owns the Case.

The adopted execution target has two streams: ordinary token/input/activation
computation and persistent, cross-context, model-native cognitive state. The
latter has independently versioned/resident representation, State Read and
eventual State Update. Both capabilities are **OPEN target requirements now**,
not deferred out of the architecture because no current consumer implements
them. Their concrete algorithms and learned consumers remain research pressure.
Context-only execution is a compatibility realization, not the final paradigm.

```text
YAI: canonical history/current state → S_t
    → Compile(S_t, I_t, A_t, B_t) → active semantic working state W_t
    → public cognitive-state boundary
YVEX: M_t = Lower(W_t, Model, StateProfile)
    → State Read → Exec(X_t, M_t)
        ├── Y_t: non-authoritative output
        ├── M_t+1: computational State Update, retained below YAI admission
        └── P_t optional: explicit semantic proposal
              → YAI validation + authority + admission → S_t+1
```

The target equations are:

```text
W_t = Compile(S_t, I_t, A_t, B_t)
M_t = Lower(W_t, Model, StateProfile)
Exec(X_t, M_t) → (Y_t, M_t+1, P_t optional)
```

S_t is durable/model-independent semantic state under existing admission rules.
I_t is current intent, A_t authority/disclosure, B_t the execution budget.
YAI compiles W_t as a derived, bounded, task-sufficient working state; it is not
a new canonical owner or the whole Case by default. Locality and permitted
disclosure belong to compilation, not a later optimization. YVEX need receive
only admitted W_t and immediate input X_t. Provisional SemanticStateFrame/Delta
describe working-state representation/changes, not latent layouts.

M_t is derived, model-specific, replaceable, computational and potentially
opaque. Lower describes initialization/reconstruction; Exec may subsequently
evolve it without changing S_t or recompiling at every step. Continuous M_t+1
updates do not require a semantic event. Y_t is computational output; P_t, when
present, is a separate explicit semantic proposal, not a synonym for M_t+1.
Only P_t enters semantic admission, which may refuse it. No proposal means no
semantic mutation from that internal update, not omission of the established
governed Invocation/Result/effect evidence. Recording execution is distinct
from admitting its proposed semantic consequence.

Changing DeepSeek to Qwen may invalidate M_t, never the Case. Recompilation
selects the currently needed W_t from qualified S_t, current intent, authority
and budget. It does not promise identical hidden activations or preservation
of unadmitted computation. M_t must not be the sole copy of admitted Case truth.

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

Current Projection/ContextFrame select and render semantic material for an
invocation. They are bounded compilation footholds, not the definition of
memory. The selected future program should investigate their evolution toward:

```text
Canonical / derived Case state + current intent + authority/disclosure
    + target capabilities + active dependencies + execution budget
        ↓
Working-state compilation
        ↓
W_t: provider-independent active semantic working state
        ↓
Target/provider/YVEX lowering
```

`SemanticStateFrame` / `SemanticStateDelta` (earlier discussion used
`ExecutionFrame`) are provisional names for intermediate semantic
representations, **not implemented components or frozen schemas**. They must not
duplicate Projection, ContextFrame, CaseState or the current intent owner.
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
in different ownership domains. Exec may produce M_t+1 without P_t; YAI does not
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
| Cold-model substitution | Mature Case, different qualified exact target, zero inherited continuation, no full-history injection; compare required task facts, provenance and authority against canonical state | Correct continuation requires old provider state, unqualified guesses or dumping history |
| Case-age locality | Vary history length radically while holding current state, task and disclosure equivalent; measure sufficient working-set entries/units and actual tokens where available | Model-visible context grows linearly with unrelated history, or stays small only by losing required evidence |
| Derived-state amnesia | Snapshot canonical history/current replay and owned-object identities; drop only disposable graph/index/retrieval/memory artifacts, rebuild with pinned algorithms and recorded results | Canonical truth changes, original payload is lost, or semantic rebuild silently requires fresh inference; re-encoding cost is accounted separately |
| Provider-state amnesia | Remove continuation/session/KV/runtime hot state below the provider boundary; replan and reconstruct from admitted sources | Case semantics or authority depend on lost computational state; efficiency degradation alone is not failure |
| Bounded persistent-KV State Read | Compile scoped W_t, lower it into an exact model/profile's persistent prefix/KV, observe actual reuse, then invalidate/rebuild under changed scope/model | Mere cache persistence is called semantic qualification, reuse is not demonstrated, or hidden/outdated state bypasses W_t admission |
| Computational update without semantic proposal | Exec changes M_t with P_t absent; separately submit valid/invalid explicit P_t under current authority | Every computational micro-update requires a semantic Transition, or M_t/Y_t admits a fact without P_t validation; required execution evidence must still be recorded |
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

## Program boundary

The Roadmap alone selects boundaries and dependency horizons. This document
retains target meaning and falsifiers, not a second execution queue. Adopted
State Read/Update remain distinct from hypothetical implementations. No schema,
H20/W21/W22, REPLAI change, Studio or runtime refactor is authorized by this
documentation.
