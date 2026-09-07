# Semantic state fabric and execution compilation

Authority: **architectural target / post-I10 program — not current implementation
truth**. [Roadmap](../ROADMAP.md) owns sequencing; [Constitution](constitution.md)
owns invariants; [Architecture](architecture.md) owns executable status. This
document freezes no schema, Rust type, module, protocol or storage owner.

## Status and claim vocabulary

- **IMPLEMENTED FACT** means a bounded source/test-backed property, with its
  evidence and limitations; it is not a universal production guarantee.
- **ARCHITECTURAL DECISION / TARGET** means an adopted direction or invariant,
  not a claim that every necessary mechanism exists.
- **HYPOTHESIS / RESEARCH PRESSURE** means a mechanism or scaling proposition
  that must earn implementation through a consumer and falsification.

The reconciled baseline includes I01–I06, TEST.TOPOLOGY.0, native REPLAI R4 and
published R5 vendor removal. **I01–I06 COMPLETE. I07 UNSELECTED.** REPLAI is the
sole native interactive editor; R5 changed neither cognitive semantics nor the
qualified pin. Their reports remain the authority for their bounded evidence.
The post-I10 Semantic State / Execution Compilation program is **RECORDED** as
a future target. Its entry requires explicit Interlock closure and program
authorization; the horizon name neither specifies I07–I10 nor reserves their
implementation. The next boundary must be chosen from current repository pressure.

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
| Target realization | Adapters lower admitted semantic input into a target-native form; physical execution belongs below that boundary | Existing provider render and typed realization exist; future recurrent/native-state lowering is hypothetical |

**State Fabric** names only this architectural composition. It is **not** a
database, daemon, canonical owner, registry, service, StateFabricStore or mutable
global WorkingMemory. A working copy of canonical intent, authority or content
does not change the authority of its source. Existing governance and security
histories remain with their established owners; a Case binds their exact facts.

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
invocation. Post-I10 should investigate their evolution toward:

```text
Canonical / derived Case state + current intent + authority/disclosure
    + target capabilities + active dependencies + execution budget
        ↓
Working-state compilation
        ↓
Provider-independent execution representation
        ↓
Target/provider/YVEX lowering
```

`ExecutionFrame`, if used, is a provisional name for that intermediate semantic
representation, **not an implemented component or frozen schema**. It must not
duplicate Projection, ContextFrame, CaseState or the current intent owner.
Typed capability requirements can constrain compilation without placing a model
family, wire protocol, session ID or physical deployment into semantic identity.
Evidence must decide whether existing contracts evolve or a distinct contract
is justified; no additional persistent owner is presumed.

Possible lowerings include ordinary messages/context, typed multipart input,
opaque continuation plus semantic delta, recurrent checkpoint plus input delta,
or future native state/memory handles. Only current text/typed adapter and
optional continuation contracts are implemented footholds; the other forms are
compatibility hypotheses, not promised adapters. Every lowering remains subject
to exact target qualification and honest refusal. Context compilation cannot
grant permission to execute operations.

YAI should be model-aware through typed capability contracts and model-independent
in semantic ownership. Transformer, SSM, RWKV, Mamba and future architecture
names are not semantic dispatch rules. YVEX/provider owns physical engines,
sessions, placement, KV/recurrent state and computational evidence. Future
checkpoint/native-state forms must remain disposable optimizations with a
semantic reconstruction route, not new Case authority. No private YVEX protocol
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

ProviderResult → interpretation/proposed typed delta → typed validation →
applicable authority/policy/evidence closure → admitted Transition → new
CaseState is the intended admission pattern. Recording output proves that it
was returned, not that its assertions are true. An effect still requires the
existing Operation/Decision/Grant and effect/reconciliation boundaries; admission
to history alone is not permission. An inference remains inference.

Existing operational proposal normalization and W20 consolidation are limited
footholds, not a generic mutation language. Post-I10 must establish actual
producers, consumers, scope and failure semantics before adding any delta API.
There is no automatic model-to-memory or model-to-state write path implied.

## Architectural falsifiers — future acceptance, not current passes

| Evaluation | Controlled setup and observation | Falsifier |
|---|---|---|
| Cold-model substitution | Mature Case, different qualified exact target, zero inherited continuation, no full-history injection; compare required task facts, provenance and authority against canonical state | Correct continuation requires old provider state, unqualified guesses or dumping history |
| Case-age locality | Vary history length radically while holding current state, task and disclosure equivalent; measure sufficient working-set entries/units and actual tokens where available | Model-visible context grows linearly with unrelated history, or stays small only by losing required evidence |
| Derived-state amnesia | Snapshot canonical history/current replay and owned-object identities; drop only disposable graph/index/retrieval/memory artifacts, rebuild with pinned algorithms and recorded results | Canonical truth changes, original payload is lost, or semantic rebuild silently requires fresh inference; re-encoding cost is accounted separately |
| Provider-state amnesia | Remove continuation/session/KV/runtime hot state below the provider boundary; replan and reconstruct from admitted sources | Case semantics or authority depend on lost computational state; efficiency degradation alone is not failure |
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

After explicit Interlock closure and program authorization, select work on semantic working
state, provider-independent representation, compilation, scoped demand paging,
context locality, proposed typed deltas, cold-start/model-substitution invariance
and target-native lowering from measured boundary pressure. Their dependency
order and implementation waves are not pre-numbered here. No H20/W21/W22,
REPLAI change, Studio or runtime work is authorized by this documentation.
