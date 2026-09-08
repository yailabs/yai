# YAI — living project control

Authority: living public project control. This is the sole live roadmap for
macro state, maturity, strategic programs, engineering selection and promotion.
[Constitution](docs/constitution.md) owns invariants;
[Architecture](docs/architecture.md) owns executable truth. Reports are bounded
evidence, not competing status pages. Git owns chronology.

## At a Glance / Current Snapshot

| Question | Current answer |
|---|---|
| Project target | Model-independent semantic cognitive-state and governed execution substrate whose durable Case state can be compiled toward context-compatible and model-native persistent execution state. |
| Selected engineering boundary | **SEMANTIC.STATE.REFOUNDATION.0 — SELECTED_NOT_STARTED**. Selection is project direction, not authorization to mutate runtime in this documentation closure. |
| Latest major completed product boundary | Golden governed Case lifecycle plus guided `init` → `open` → in-Case actions; subsequent catalog discovery, single `/connect` and separate model/system presentation are published. |
| Most important structural gap | Existing typed state and bounded Projection/ContextFrame do not yet constitute explicit general semantic cognitive state or a general State Compiler. |
| Executable foundation | Transition v18 / CaseState v15; immutable owned content; authority/effects; I01–I06; Workflow/Handoff; derived access paths; native REPLAI; LMDB 37/40. |
| Primary research boundary | State Read and State Update over model-native persistent computational state, with qualified reconstruction and YAI semantic admission. Both OPEN. |
| Current compatibility path | Governed exact OpenAI-compatible context/text, bounded typed content/functions/JSON when mechanically qualified; optional opaque continuation, not a native state contract. |
| Target stateful path | Semantic state → bounded compilation → provisional SemanticStateFrame/Delta → YVEX lowering → persistent model state plus immediate input. Not implemented. |
| Human Golden acceptance | **PENDING_OPERATOR**; automated PTY evidence cannot supply this verdict. Continuity canary **NOT_RUN**. |
| YVEX external posture | Partial synthetic text/functions/JSON qualification observed; real Case SEND received HTTP 413. Full external Golden lifecycle **not qualified**. [Exact observations][connection]. |
| Next decision point | Authorize a bounded source-refoundation contract with state/admission and compatibility oracles; retain external capacity/setup-latency blockers rather than relabeling them solved. |

**I01–I06 COMPLETE. I07 UNSELECTED.** Source refoundation, W19/H19/W20,
TEST.TOPOLOGY.0, REPLAI R4/R5 and Golden/guided CLI remain completed evidence.
No wave is reopened and no I07–I10 is invented. The formerly “post-I10” direction
is now organized under programs S/C/Q below; the old horizon is not a numbering
dependency. Its [target doctrine](docs/semantic-state-execution-target.md)
remains non-executable. This roadmap authorizes neither that runtime program nor
H20/W21/W22. The qualified operator-updated REPLAI pin is preserved.

## Adopted thesis and cognitive-state boundary

**YAI owns semantic cognitive state. YVEX owns computational cognitive state.**
This is an architectural decision/target, not a claim of a deployed dual-stream
runtime. A model owns learned computation; a future model may learn to read and
update persistent computational state. It never thereby acquires Case authority.

```text
External world / admitted Case events
  → canonical history + authoritative current state
  → qualified semantic cognitive state S_t
  → semantic state compilation
  → SemanticStateFrame / SemanticStateDelta (provisional names)
  → YVEX computational-state lowering
  → model-native persistent state M_t + immediate token/input stream
  → model

Model result / computational update → YVEX computational evidence
  → semantic proposal / consequence → YAI validation + authority + admission
  → new canonical semantic state
```

| Boundary | Responsibility | Must not acquire |
|---|---|---|
| YAI | Transition history, CaseState, owned content meaning/provenance, Participants, identity/scope/disclosure, Policy, Decisions/reviews/Grants, Resource relations, Workflow/Handoff; target objectives/facts/obligations/unresolved state, semantic selection/working-state/deltas, proposal admission and cross-model continuity | tensors, KV, latent banks, layer layouts, physical state paging or GPU placement |
| YVEX / execution substrate | Exact model/deployment/runtime truth; target computational capabilities, semantic-to-model-state lowering, layout, State Read/Update realization, KV/recurrent/SSM/latent state, paging/residency, checkpoints, rollback/invalidation, layers/kernels and physical evidence | Case authority, semantic memory ownership, Policy, Workflow truth or a shared canonical database |
| Model | Learned computation; target learned State Read/Write behavior under the exact substrate contract | self-admission of proposals, resource permission or canonical semantic authority |

Physical responsibilities are assigned here, not certified as existing YVEX
capabilities. YAI must require truthful public contracts and accept typed refusal;
no private ABI, runtime administration or model-family branch closes a missing
contract. Transformer KV, MLA, SSM, RWKV recurrence and latent banks are possible
lowerings, never Case ontology.

The target has two streams: ordinary token/activation/residual computation and
cross-context persistent, model-native, independently versioned/resident cognitive
state, readable and eventually updateable by the model. **State Read and State
Update are OPEN target capabilities now**, not LATER because consumers require
research. Context-only execution is a compatibility realization, not the final
architecture.

```text
S_t = YAI semantic cognitive state
M_t = Lower(S_t, exact model, exact state profile)
```

M_t is derived, model-specific, replaceable, computational and potentially opaque.
S_t is durable, semantic and model-independent; its authority still comes only
from existing admission rules. DeepSeek → Qwen can invalidate M_t, never the Case.
Recompilation targets qualified semantic state, not bitwise recovery of another
model's hidden activations or an identical stochastic answer. An update that has
not been semantically admitted cannot be the sole copy of a durable Case fact.

Semantic state is not a mutable mega-object: Transition Ledger is canonical Case
history; CaseState is authoritative current materialization; immutable owned
payload survives separately; graph/index/memory/retrieval/analytics are derived
access paths. Governance and security retain their established histories.
Admission requires durable identity/provenance/relation, not every byte in the
ledger. “State Fabric” denotes this composition, never a Store, daemon, database,
registry or service. Human and model Participants differ in identity, evidence,
authority, capability and substrate, not in automatic ownership: neither owns a
Case merely by participating. An Agent remains a possible composition.

## System Maturity

Maturity applies to the **bounded property in each row**, not an entire domain
or production guarantee. 🟢 ESTABLISHED has executable positive/negative evidence
for that scope; 🟡 PARTIAL has foundations but an incomplete generic boundary;
🔴 OPEN is an adopted unresolved property; ⚪ LATER lies beyond current dependency
horizons. Temporal execution status is separate. Counts describe rows only,
never percentage completion. Evidence promotion still requires human review.

<!-- maturity-summary:start -->
ESTABLISHED=28 PARTIAL=22 OPEN=14 LATER=4 TOTAL=68
<!-- maturity-summary:end -->

<!-- maturity:start -->
### Case language and canonical representation

| ID | Property | Maturity | Evidence / precise boundary |
|---|---|---|---|
| K01 | Atomic canonical ledger and current CaseState replay | 🟢 ESTABLISHED | [Canonical/recovery proofs][kernel], [Golden replay][golden]; duplicate/stale/partial commits refuse. |
| K02 | Immutable ordered content, owned payload and provenance | 🟢 ESTABLISHED | [I01][i01], [I03][i03]; object-first publication, original Turn unchanged after derivation. |
| K03 | Case lifecycle and portable continuity lifecycle | 🟡 PARTIAL | [Termination][temporal] exists; generic export/clone/machine migration is not a qualified lifecycle. |
| K04 | Historical typed-reader and materialization compatibility | 🟢 ESTABLISHED | [Golden schema/reopen tests][golden], [I06][i06]; known predecessors read, unknown versions refuse; no universal migration claim. |

### Authority and governed semantic mutation

| ID | Property | Maturity | Evidence / precise boundary |
|---|---|---|---|
| A01 | Immutable policy source/IR/publication supply chain | 🟢 ESTABLISHED | [Governance hardening][governance], [Golden deck][golden]; bounded constrained grammar, no model policy interpreter. |
| A02 | Exact Case binding and READY EffectivePolicy materialization | 🟢 ESTABLISHED | [Materialization][policy], [Golden][golden]; publication alone grants no authority. |
| A03 | ALLOW / DENY / REQUIRE_REVIEW, review and finite Grant | 🟢 ESTABLISHED | [Admission][admission], [Golden][golden]; DENY has no Grant/effect, model cannot self-approve. |
| A04 | Local Principal/Tenant/Participant and disclosure isolation | 🟢 ESTABLISHED | [Security][security], [Golden isolation][golden]; local POSIX trust model, not enterprise authentication. |
| A05 | Validity/revoke, generation and PREPARE authority cut | 🟢 ESTABLISHED | [Temporal governance][temporal]; contracting authority cannot erase prepared external uncertainty. |
| A06 | Production identity, credentials and privacy lifecycle | 🟡 PARTIAL | Local identity and credential references exist; SSO, membership removal and general retention/deletion remain unresolved. |

### Semantic Cognitive State

| ID | Property | Maturity | Evidence / precise boundary |
|---|---|---|---|
| S01 | Model-independent canonical continuity foundation | 🟢 ESTABLISHED | [Kernel][kernel], [I06 recovery][i06]; Case identity/history do not contain provider continuation. |
| S02 | Typed objectives and intent across semantic work | 🟡 PARTIAL | Conversation/work intent and Workflow tasks exist; no general objective lifecycle. Legacy Record nouns do not fill the gap. |
| S03 | Obligations and unresolved semantic state | 🟡 PARTIAL | Policy evidence obligations, pending review/effects and Workflow blockers exist; not a general obligation/state algebra. |
| S04 | Evidence-bound facts/claims with preserved epistemic class | 🟢 ESTABLISHED | [W20][memory]; bounded grounded/inferred/provider/control classes, never similarity-to-authority promotion. |
| S05 | Structural episodic experience | 🟢 ESTABLISHED | [W20][memory]; bounded derived Episodes and recorded-result reconstruction without re-inference. |
| S06 | Admitted resource and Workflow consequence references | 🟢 ESTABLISHED | [Golden][golden], [Workflow][workflow]; external observation/effect and progression are distinguished. |
| S07 | Semantic-state hierarchy | 🟡 PARTIAL | Operational/episodic/assertion derivations exist; no unified qualified semantic-state representation. |
| S08 | Multi-timescale semantic state | 🔴 OPEN | No tested timescale selection/update contract. Retention age is not cognitive timescale. |
| S09 | Task-local versus Case big-picture state | 🔴 OPEN | No explicit general separation and sufficiency oracle. |
| S10 | Semantic replacement and supersession | 🟡 PARTIAL | Binding replacement and mechanical derived supersession exist; general semantic replacement needs owner-specific admission. |
| S11 | Cross-model semantic continuity | 🟡 PARTIAL | [Effect/model replacement][continuity], [I05][i05]; mature cold-state substitution is not fully proven. |

### State Compilation and Cognitive-State Boundary

| ID | Property | Maturity | Evidence / precise boundary |
|---|---|---|---|
| C01 | Provider-independent working-state selection | 🟡 PARTIAL | Projection/Residency/ContextFrame are bounded compilation footholds, not the definition of memory. |
| C02 | Authority/disclosure-aware general compilation | 🟡 PARTIAL | Current admitted view and mandatory-state selection are enforced; generalized state-page compilation is absent. |
| C03 | Explicit SemanticState representation | 🔴 OPEN | Provisional concept, no general IR/schema or new owner selected. |
| C04 | SemanticStateFrame | 🔴 OPEN | Provisional representation, not an alias that promotes ContextFrame into a finished compiler. |
| C05 | SemanticStateDelta | 🔴 OPEN | Needs source/destination, provenance, invalidation and full-rebuild equivalence; not the deferred context transport delta. |
| C06 | Active working-set compilation and Case-age locality | 🔴 OPEN | Bounded historical fixtures are positive controls, not task-sufficient age independence. |
| C07 | Exact scoped state references / demand expansion | 🔴 OPEN | Current exact IDs are foundations, not a semantic paging API or ambient access. |
| C08 | Context-compatible lowering with exact execution lineage | 🟢 ESTABLISHED | [I03][i03], [I06][i06], [Golden][golden]; bounded generic adapter proof in loopback, not every live deployment. |
| C09 | Target capability negotiation | 🟡 PARTIAL | Semantic suitability and mechanical shapes are separate; public persistent-state capabilities are not integrated. |
| C10 | Public YAI/YVEX cognitive-state contract | 🔴 OPEN | No implemented semantic-frame-to-model-state consumer in YAI; private protocols are excluded. |
| C11 | Persistent model State Read | 🔴 OPEN | First-class target; requires truthful computational capability and consumer qualification. |
| C12 | Persistent model State Update | 🔴 OPEN | First-class target; computational write is not semantic admission. |
| C13 | Model-state feedback semantic admission | 🟡 PARTIAL | Operational/PlanPatch proposals pass existing admission; arbitrary computational-state feedback has no semantic normalizer. |
| C14 | Cross-model recompilation | 🔴 OPEN | No general semantic-state/profile lowering equivalence or cold-model task oracle. |

### Cognitive execution and Participants

| ID | Property | Maturity | Evidence / precise boundary |
|---|---|---|---|
| E01 | Pinned/ordered arbitration, exact plans and isolated lanes | 🟢 ESTABLISHED | [I02][i02], [I05][i05]; explainable eligibility, no model-name inference or hidden transport substitution. |
| E02 | Production semantic suitability evidence | 🟡 PARTIAL | Exact evidence contract exists; operator attestation is not a production semantic evaluation. |
| E03 | Exact typed realization and delivery-safe result lineage | 🟢 ESTABLISHED | [I03][i03], [I05][i05]; stale evidence and uncertain delivery fail closed. |
| E04 | Explicit finite composition and host intent | 🟢 ESTABLISHED | [I04][i04], [I06][i06]; direct bypass or one explicit speech/image prerequisite; no modality inference. |
| E05 | Real-provider context fit and setup performance | 🟡 PARTIAL | [Observed 413 and serial probes][connection]; qualified small probes do not establish Case capacity or instant setup. |

### Operational resources and external effects

| ID | Property | Maturity | Evidence / precise boundary |
|---|---|---|---|
| O01 | Confined filesystem read/search and governed write | 🟢 ESTABLISHED | [Golden][golden], [fencing][fencing]; admitted Linux profile, no ambient filesystem. |
| O02 | Discovery candidate → explicit immutable admission | 🟢 ESTABLISHED | [Golden][golden]; digest/drift and disclosure checked; discovery itself is not attachment. |
| O03 | Bounded process/test-runner capability | 🟡 PARTIAL | [Golden][golden]: real constrained Linux x86_64 runner; not a general writable build environment or shell. |
| O04 | Database resource | 🟡 PARTIAL | [Golden][golden]: named SQLite query on bounded quiescent image; mutations denied, live/WAL/general drivers absent. |
| O05 | Ordinary HTTP resource | 🟡 PARTIAL | [Golden][golden]: exact scoped GET, no redirects; state-changing HTTP not admitted. |
| O06 | MCP resource/tool boundary | 🟡 PARTIAL | [Golden][golden]: bounded 2026-07-28 stateless Streamable HTTP, exact catalog/schema revalidation; not the full MCP ecosystem. |
| O07 | Observation/effect separation, fencing and uncertainty | 🟢 ESTABLISHED | [Fencing][fencing], [Golden][golden]; PREPARE before effects, exact terminal evidence or INDETERMINATE, never blind retry. |

### Workflow / composition / Handoff

| ID | Property | Maturity | Evidence / precise boundary |
|---|---|---|---|
| W01 | Exact Workflow progression, amendments and bounded Subflow | 🟢 ESTABLISHED | [Workflow][workflow], [H17][workflow-hardening], [Golden][golden]; PlanPatch proposal cannot adopt itself. |
| W02 | Explicit same-Tenant Handoff | 🟢 ESTABLISHED | [H17][workflow-hardening], [Golden][golden]; independent source/target history, no transfer of authority/resources. |
| W03 | Product Agent composition | ⚪ LATER | Participant/capability/Workflow foundations only; no Agent implementation or canonical Agent owner. |

### Derived state and semantic access paths

| ID | Property | Maturity | Evidence / precise boundary |
|---|---|---|---|
| M01 | Provenance-bound operational/episodic/assertion derivation | 🟢 ESTABLISHED | [W20][memory], [Golden rebuild][golden]; derived failure cannot change canonical truth. |
| M02 | Qualified hybrid indexing and selected-source revalidation | 🟢 ESTABLISHED | [W19][index], [H19][index-hardening]; bounded BM25/exact-cosine oracle, hidden state cannot alter visible ranking. |
| M03 | Graph/causal access | 🟡 PARTIAL | Typed graph/rebuild and Handoff relations exist; generic scoped causal paging and invalidation remain incomplete. |
| M04 | Analytical extraction | 🟡 PARTIAL | Existing DuckDB extractors are derived; not all declared fact families have producers. |
| M05 | Semantic access quality and scale | 🟡 PARTIAL | Qualified exact/lexical/vector selection exists; learned ranking/compression, background indexing and broader scale need evidence. |

### Product interfaces

| ID | Property | Maturity | Evidence / precise boundary |
|---|---|---|---|
| X01 | Registry-backed CLI plus short guided Case setup | 🟢 ESTABLISHED | [Guided CLI][guided]; exact automation retained, no global current-Case authority or silent trust. |
| X02 | Native REPLAI Case workbench | 🟢 ESTABLISHED | [R4][replai], [R5][r5], [presentation][presentation]; real PTY and controller seams, no second terminal. |
| X03 | Frontend-independent application/API consumption | 🟡 PARTIAL | Typed Rust host exists; generic remote API/SDK product and authentication are not qualified. |
| X04 | Studio | ⚪ LATER | No implementation; frontend would consume Case semantics, not own them. |

### Qualification

| ID | Property | Maturity | Evidence / precise boundary |
|---|---|---|---|
| Q01 | Proof/provider topology and deterministic publication union | 🟢 ESTABLISHED | [TEST.TOPOLOGY.0][topology], [current regression][presentation]; classification/reachability and no fixture-to-external promotion. |
| Q02 | Golden local cumulative product lifecycle | 🟢 ESTABLISHED | [Golden][golden], [current free/Workflow rerun][presentation]; real executable/stores/adapters, loopback model only. |
| Q03 | External YVEX product qualification | 🟡 PARTIAL | [Public probes and Case refusal][connection]; no complete external Golden PASS or model-quality claim. |
| Q04 | Human Golden acceptance | 🔴 OPEN | [Runbook](docs/zero-to-current.md) prepared; PENDING_OPERATOR, not automated human PASS. |
| Q05 | Long-lived continuity canary | 🔴 OPEN | Operator procedure recorded; retained cross-upgrade Case not exercised by this closure. |
| Q06 | Cognitive-state comparative evaluation | 🔴 OPEN | No qualified cross-state/dual-stream benchmark; spectrum below defines falsifiers, not results. |

### Federation / scale

| ID | Property | Maturity | Evidence / precise boundary |
|---|---|---|---|
| F01 | Single-host multi-Case scheduling and admission recovery | 🟢 ESTABLISHED | [H13][runtime]; bounded workers, Tenant fairness and canonical-first acknowledgement recovery, not distributed consensus. |
| F02 | Cross-host state/resource coordination | ⚪ LATER | Local scheduling/fencing do not prove distributed admission, revocation or conflict closure. |
| F03 | Federated semantic continuity | ⚪ LATER | Requires mature local state/admission plus explicit replication/disclosure semantics. |
<!-- maturity:end -->

Derived access structures feed semantic state and working-state compilation;
their strategic role is not “text to stuff into prompts.” Exact current questions
use typed relations, causal questions use qualified traversal, chronological
questions use history, and similarity retrieval serves questions requiring it.
Canonical truth, current materialization, derived inference, retrieval and model
claim remain distinct. Repetition never upgrades their epistemic class.

## Strategic Programs

Programs are long-lived project-control axes, not new source modules or owners.
Letters are local shorthand, not the numbering of historical W/I/R waves.

| Program / purpose | Established foundation | Open maturity boundaries | Current pressure | Material advance | Explicit non-ownership |
|---|---|---|---|---|---|
| **R — Architecture Integrity & Refoundation**: preserve one honest owner per lifecycle | Source refoundation; Constitution; I01–I06 and Golden convergence | Duplicated/model-centric preparation assumptions; stale claims | Reconcile state selection and context rendering before adding representations | Remove a demonstrated duplicate while preserving its unique proof and consumer | No StateFabricStore, Agent runtime or module-per-noun redesign |
| **K — Case Kernel & Canonical State**: durable identity, history and current truth | K01–K04; immutable payload/reference distinction | Portable upgrade coverage and general state relations | Keep new state meaning replayable and generation-scoped | Typed relation with exact payload ownership, replay and recovery evidence | No blob ledger or universal attachment owner |
| **A — Authority & Governance**: govern semantic and operational admission | A01–A05; Principal/Participant, policy, review, Grant, temporal authority | Broader disclosure and credential integrations | Ensure state compilation/expansion cannot widen authority | Negative scope, revocation and current-generation proof for a new consumer | Neither model output nor a capability catalog grants authority |
| **S — Semantic Cognitive State**: preserve model-independent task and Case meaning | S01–S07/S10 foundations; typed history, evidence and consequences | Multiscale state, task/big-picture separation, generic semantic deltas | Identify actual semantic producers and admission/supersession contracts | Explicit bounded state model demonstrably preserving provenance and current meaning | No mutable mega-object; derived assertions do not become canonical facts |
| **C — State Compilation & Cognitive Boundary**: compile governed semantic working state | Projection/ContextFrame compatibility footholds; exact target contracts | C03–C07, C10–C12/C14: Frame/Delta, locality, State Read/Update, recompilation | Define one bounded compatibility consumer and future public boundary | Qualified full/delta equivalence and refusal; later a real persistent-state consumer | No tensors, latent layouts, KV or YVEX placement in YAI |
| **E — Cognitive Execution**: exact responsibility and provider-independent intent | I02–I06 planning, arbitration, realization, composition and host | Production suitability, capacity negotiation and setup costs | Resolve observed public-capacity/setup pressure without brand-specific routing | Same exact-plan guarantees with a new qualified execution shape/target | Cognitive arbitration is not transport failover or resource scheduling |
| **O — Operational World**: governed observations, attachments and effects | Golden filesystem/process/SQLite/HTTP/MCP/discovery verticals | General adapter coverage and external ambiguity | Keep future model state requests within the existing capability/admission path | New concrete consumer plus DENY/review, drift, recovery and isolation proof | No ConnectorStore/ToolStore; a database Resource is not YAI memory |
| **W — Workflow & Composition**: explicit progression and bounded delegation | Workflow amendments/PlanPatch, I04 composition and exact Handoff | Broader delegation and future Agent compositions | Preserve one resource/authority substrate in free and Workflow work | Replayable progression and explicit adoption across a new consumer | Workflow/Agent is not Case continuity or a private execution universe |
| **M — Derived State & Semantic Access**: rebuildable access and evidence derivation | W19/H19/W20; graph and analytics | Generic causal access, retrieval quality/scale and state-selection consumers | Reposition derived access as semantic compilation input, not prompt stuffing | Rebuild invariance plus relevance/provenance evidence for a bounded question | Index, summary, episode and similarity are not historical authority |
| **X — Product Interfaces**: thin, usable views over Case semantics | Guided CLI, native REPLAI, application controller, cumulative runbook | Broader API clients, setup ergonomics; Studio later | Keep short product actions and model/system output distinct | Product acceptance through typed seams, without shelling out or moving semantics into UI | REPLAI owns terminal mechanics only; no ChatStore |
| **Q — Qualification**: independent evidence for each claimed property | TEST.TOPOLOGY.0 and deterministic Golden lifecycle | External Golden, human/canary, cross-model and state evaluation | Retain real failures; define refoundation falsifiers before implementation | Correct proof/provider mode plus negative/recovery/product/external/human evidence as claimed | A fixture, generated report or PASS cache is not semantic authority |
| **F — Federation & Scale**: future distributed continuity | Single-host multi-Case scheduling/fencing | Cross-host coordination and federation | Establish local state/authority correctness first | Explicit conflict, disclosure, revocation and recovery contracts across hosts | No global Space or shared database by implication |

## Cognitive State Spectrum

This is an adversarial qualification spectrum, not another maturity registry or
an authorized milestone list. Its bounded verdicts do not enter maturity counts.
Golden supplies a common Case/task workload; it owns none of these semantics.

| Pressure | Evidence posture | Existing control / future falsifier |
|---|---|---|
| Long-lived Case on context-only target | PARTIAL | Current canonical continuity and bounded host tests; no retained mature operator canary qualification. Continue a bounded task without importing all chat history. |
| Cold, zero-continuation model replacement | PARTIAL | Exact target/lane replacement is tested in [I05][i05]; not a mature-Case cold-state oracle. A different qualified model must recover sufficient task state without inherited computational state or full history injection. |
| Active task versus Case-age locality | OPEN | Hold task/current meaning equivalent while radically varying history length; execution working sets should be comparable, not linear in Case age. Large genuinely necessary task context remains allowed. |
| Task-local versus Case/big-picture state | OPEN | Switch tasks without losing current constraints/obligations or disclosing unrelated state. A narrative summary alone cannot establish this property. |
| Incremental semantic-state delta | OPEN | Full reconstruction and admitted delta application must agree under replacement, deletion, stale generation and restart. No schema is frozen here. |
| Derived-state amnesia | ESTABLISHED, bounded | [Golden][golden] drops/rebuilds its disposable graph/index/memory and compares canonical truth. Owned immutable bytes and canonical ledgers are NOT disposable; this does not qualify every future derivation family. |
| Persistent / incremental State Read | OPEN | A public qualified consumer reads independently resident model-native state; no relabeling of a context window or persistent KV as this proof. |
| State Update plus semantic admission | OPEN | Computational updates remain derived; validate/admit proposed semantic consequences. Reject unauthorized updates and exercise invalidation/reconstruction without losing admitted Case truth. |
| Participants and explicit cross-Case Handoff | ESTABLISHED, bounded | [Security][security], [Golden][golden]: same Case with differing disclosure, typed refusal, exact Handoff without resource/authority cloning. Not federated sharing. |
| Multimodal and operational continuity | PARTIAL | [I04][i04] typed prerequisite restart and Golden operational continuity are separate qualified controls; no combined live-YVEX multimodal Golden claim. |

Provider-state amnesia is cross-cutting: loss of optional continuation must not
invalidate Case correctness. Future state experiments must specify what can be
reconstructed and what unadmitted computation may be lost. No YAI test gains
permission to administer YVEX internals or discard operator state.

## Current Execution Sequence

Temporal status is separate from maturity. There is exactly one selected
implementation boundary; recording it does not mean source work has started.

<!-- execution:start -->
| Boundary | Temporal state | Programs | Required after-state |
|---|---|---|---|
| SEMANTIC.STATE.REFOUNDATION.0 | SELECTED_NOT_STARTED | R K A S C E O W M X Q | Explicit qualified semantic cognitive state → bounded State Compiler → compatibility context projection → future model-state boundary, preserving existing owners, replay, authority, resources/effects, Workflow and Golden. |
<!-- execution:end -->

Before authorizing source mutation, resolve the bounded state producer/consumer,
admission and supersession rules, current-state versus derived-state distinction,
scope/generation checks, compilation budget and full/delta compatibility oracle.
Use archaeology to decide whether any serialized meaning actually changes.
There is no preselected module tree, database, schema, API or migration.

The external request-capacity and connection-probe costs remain E/X/Q pressure,
not a second selected wave and not evidence that the state refactor fixes them.
I01–I06 are completed anchors; I07 is unselected. Historical “post-I10” naming
does not require inventing four Interlock deliveries before selecting this work.

## General Substrate Progression

Horizons express dependencies, not dates or automatic implementation authority.
State Read/Update are OPEN target capabilities now even where their consumers
fall in later horizons.

| Horizon | Property pressure | Dependency / admission to work |
|---|---|---|
| Now | Semantic-state architecture/refoundation | Selected boundary above; bounded source contract still required |
| Near | Explicit semantic state, provisional Frame/Delta and context compatibility | Proven current-state ownership, producer/consumer and replay equivalence |
| Near | Semantic locality, working-set compilation and scoped state references | Authority-aware selection; exact resolution and refusal; no ambient handles |
| Near / Mid | Public YVEX cognitive-state boundary and first persistent State Read consumer | Truthful negotiated public capability and an actual implementing target |
| Mid | Cross-model recompilation, state evaluation, semantic admission of computational updates | Reconstruction/invalidation semantics plus independent cross-state oracles |
| Mid / Later | Learned/native State Read/Write consumers | Real model/runtime support, bounded effects and evidence; no promised architecture family |
| Later | Federation/distributed semantic continuity | Mature local semantics plus explicit distributed authority/conflict contracts |

## Product / Research Qualification Path

No named **v0.1 product release scope is selected** by this roadmap. Package
version strings are not a release commitment. Packaging follows qualified
product/research properties, not a maturity percentage.

| Gate / consumer | Current posture | What it authorizes |
|---|---|---|
| Deterministic software publication | Published local release/characterization evidence in [presentation closure][presentation]; not rerun by this docs closure | Classified no-provider and loopback properties, not live model interoperability |
| Golden local product lifecycle | Published PASS, deterministic model with real local persistence/resource adapters and protocol peers | Bounded free/Workflow, authority, replay/rebuild/recovery/isolation lifecycle; separate Golden lane, not folded into `make check` |
| External YVEX Golden | Partial probes; real Case request rejected with 413; complete lifecycle NOT QUALIFIED | Only the exact public interactions observed; no external Golden PASS |
| Human Golden | PENDING_OPERATOR | Only a human may report acceptance at an exact YAI/external identity |
| Continuity canary | NOT_RUN | Operator-owned retained Case across upgrades; never reset by automated qualification |
| Semantic-state refoundation | SELECTED_NOT_STARTED | No runtime/compiler/state-schema completion claim |
| Cross-model / cross-state qualification | OPEN | Future same-Case/task comparisons across compatibility and persistent-state modes |
| Product packaging/release | Scope UNSELECTED | No invented version, deployment or production-readiness guarantee |

The current external observation is against the operator-supplied public target
`http://127.0.0.1:18001`, model
`deepseek-v4-flash-mixed-mxfp4-release-v1`; [connection evidence][connection]
records synthetic text/functions/JSON qualification and the subsequent Case
`request_too_large` refusal. This is not source inspection, a loopback fixture,
or a diagnosis of private YVEX internals. DeepSeek is a reference consumer, not a
semantic branch. Qwen/model replacement is prepared, not externally qualified.
When no exact live target is available, report NOT_RUN / DEPLOYMENT_LIMITATION;
never replace it with a fixture and report PASS.

[ZERO-TO-CURRENT](docs/zero-to-current.md) remains the single cumulative human
product runbook. A future user-visible or Case-semantic change updates that same
procedure, not a delta-only manual. This documentation-only refoundation leaves
its commands unchanged because it changes no product behavior. Every relevant
handoff retains AUTOMATED REGRESSION (proof/provider axes), GOLDEN LOCAL,
EXTERNAL YVEX, CANARY, HUMAN, RUNBOOK and exact BLOCKERS. Human PASS cannot be
silently carried to a changed lifecycle.

Golden is also the future common workload for comparing context-only, reusable
prefix, persistent/incremental State Read, external State Update, learned
Read/Write and native-state consumers. These are experiment classes, not
implemented modes or implicit YVEX commitments.

## Cross-axis Traceability

| Engineering pressure | Maturity rows | Programs | Required independent evidence |
|---|---|---|---|
| Semantic-state refoundation | S02–S11, C01–C08/C13 | R K A S C M Q | Typed meaning/owner, admission negatives, replay/full-delta equivalence, compatibility Golden |
| Public persistent-state boundary | C09–C14, Q06 | C E Q | Negotiated public capability, actual State Read/Update, amnesia/recompilation and authority isolation |
| Real product provider acceptance | E02/E05, X01/X02, Q03–Q05 | E X Q | Exact target/capacity evidence, full external Golden, operator run and retained canary separately |
| Operational/Workflow convergence | O01–O07, W01/W02, M01–M05 | A O W M Q | Same admission/effect owners, real adapter transport, ambiguity/recovery and no authority leakage |
| Future federation | F01–F03, A06, K03 | K A S F Q | Explicit distributed conflict/disclosure and recovery; single-host success is insufficient |

These are references into the single matrix, not a second feature registry.
The [test classification authority](tests/classification.tsv) continues to own
test proof/provider metadata; a roadmap row never reclassifies test evidence.

## Explicit Nonclaims and Deferred Scope

| Claim not made now | Truthful boundary |
|---|---|
| Fully implemented SemanticState IR / general State Compiler | False. Typed current state and bounded selection/rendering are foundations. |
| SemanticStateFrame / SemanticStateDelta implemented | False. Names are provisional; no new source type, schema or owner is authorized here. |
| Integrated model-native persistent cognitive state / public YVEX cognitive-state contract | False. Adopted target, OPEN capability boundary, not an invented protocol. |
| Native State Read / State Update implemented | False. Both OPEN now; neither hidden in LATER nor claimed from KV/continuation. |
| General multi-timescale state / universally Case-age-independent working set | False. Target plus falsifiable research pressure. |
| Full cold-model substitution / Qwen external state qualification | False. Exact binding replacement does not establish cold-state recovery. |
| Agent implementation / Studio | False. Later product compositions; no Agent owner or new terminal. |
| Complete external YVEX Golden acceptance | False. Probe success coexists with a real Case 413; exact capacity/integration pressure remains unresolved. |
| Human Golden PASS / continuity canary compatibility | False unless independently reported at the relevant revision; current PENDING_OPERATOR / NOT_RUN. |
| Instantaneous provider setup or qualified performance | False. Published serial synthetic probes perform real inference; latency is not a new state-architecture proof. |
| Universal database/HTTP/MCP/process/framework support | False. Golden's implemented operations are bounded, governed verticals, not ambient tools or unrestricted shell. |
| Named v0.1 scope / generic production readiness | Unselected; local proof is not product release qualification. |

No I07, H20/W21/W22, post-I10 runtime implementation, Studio, private YVEX
client, shared state database or computational-state ownership transfer begins
here. Historical wave exclusions remain scoped to their reports: later Golden
resources and guided product actions are not erased by an older non-goal.

## Progression and Promotion Discipline

**Implementation existence != generic maturity.** A green row means its stated
bounded property is established, not that the surrounding domain is complete.
Promotion requires the relevant combination of a justified canonical owner,
executable contract, negative evidence, replay/recovery, product consumer,
external execution and human evidence where those are claimed. A link proves
traceability, not substantive correctness; reviewers must inspect its scope.

| Existing evidence | Does not by itself establish |
|---|---|
| Bounded Projection / ContextFrame | General State Compiler |
| Retrieval/index or repeated assertion | Semantic truth |
| ProviderResult | Admitted semantic fact |
| Model binding replacement | Cold-state substitution |
| Bounded context size | Case-age locality |
| Persistent KV / opaque continuation | Native cognitive-state semantics |
| Golden loopback model | External YVEX qualification |
| Automated PTY | Human acceptance |

Failure evidence can narrow or demote a claim without reopening historical
deliveries. Unknowns stay unknown; no fabricated quality/latency/cost score and
no model-family inference can promote a target.

## Living-update Rules

Update this document in place; Git owns previous snapshots. When a wave closes:

1. Replace Current Snapshot and advance the single selected execution boundary.
2. Promote only affected matrix rows from evidence; recompute counts from those
   rows alone. Counts describe rows, never percentage completion.
3. Update affected programs, dependencies and spectrum verdicts without creating
   a second queue or a historical narrative wall.
4. Update product/external/human/canary posture independently. Remove a nonclaim
   only after its corresponding property is proven.
5. Keep Architecture as executable truth and target doctrine as non-executable;
   preserve historical report wording and cumulative acceptance infrastructure.
6. Run `python3 tools/checks/check-roadmap.py --summary` to derive the summary,
   then `make check-roadmap check-docs check-layout test-roadmap` to validate.

There is no parallel STATUS/FEATURES/PLAN document or maturity registry. The
guard validates structure, references, states, counts and one coherent selected
boundary; it cannot manufacture a green verdict.

## Evidence anchors

Completed work remains navigable without controlling the live sequence:
[foundation recovery](refoundation/foundation-recovery/FOUNDATION-RECOVERY-REPORT.md),
[W19][index] / [H19][index-hardening] / [W20][memory],
[I01][i01] / [I02][i02] / [I03][i03] / [I04][i04] / [I05][i05] / [I06][i06],
[test topology][topology], [REPLAI R4][replai] / [R5][r5] / [qualified repin][repin],
[post-I10 doctrine alignment](refoundation/validation/post-i10-semantic-state-alignment/REPORT.md),
[Golden][golden], [guided CLI][guided], [single connection][connection] and
[conversation presentation][presentation]. These reports are bounded historical
evidence, not a second live roadmap.

The control format and documentation-only delta are qualified in the bounded
[roadmap-refoundation report](refoundation/validation/semantic-cognitive-state-roadmap/REPORT.md).

[kernel]: engine/yai-engine/src/store/lmdb.rs
[continuity]: tests/characterization/agentless-case-runtime/test_agentless_case_runtime.sh
[governance]: refoundation/foundation-recovery/hardening-8/HARDENING-8-REPORT.md
[policy]: refoundation/foundation-recovery/wave-9/WAVE-9-REPORT.md
[admission]: refoundation/foundation-recovery/wave-10/WAVE-10-REPORT.md
[temporal]: refoundation/foundation-recovery/wave-11/WAVE-11-REPORT.md
[security]: refoundation/foundation-recovery/wave-12/WAVE-12-REPORT.md
[runtime]: refoundation/foundation-recovery/hardening-13/HARDENING-13-REPORT.md
[fencing]: refoundation/foundation-recovery/hardening-14/HARDENING-14-REPORT.md
[workflow]: refoundation/foundation-recovery/wave-17/WAVE-17-REPORT.md
[workflow-hardening]: refoundation/foundation-recovery/hardening-17/HARDENING-17-REPORT.md
[index]: refoundation/foundation-recovery/wave-19/WAVE-19-REPORT.md
[index-hardening]: refoundation/foundation-recovery/hardening-19/HARDENING-19-REPORT.md
[memory]: refoundation/foundation-recovery/wave-20/WAVE-20-REPORT.md
[i01]: refoundation/foundation-recovery/interlock-01/INTERLOCK-I01-REPORT.md
[i02]: refoundation/foundation-recovery/interlock-02/INTERLOCK-I02-REPORT.md
[i03]: refoundation/foundation-recovery/interlock-03/INTERLOCK-I03-REPORT.md
[i04]: refoundation/foundation-recovery/interlock-04/INTERLOCK-I04-REPORT.md
[i05]: refoundation/foundation-recovery/interlock-05/INTERLOCK-I05-REPORT.md
[i06]: refoundation/foundation-recovery/interlock-06/REPORT.md
[topology]: refoundation/validation/test-topology-0/REPORT.md
[replai]: refoundation/integration/replai-r4/REPORT.md
[r5]: refoundation/integration/replai-r5/REPORT.md
[repin]: refoundation/integration/replai-repin/REPORT.md
[golden]: refoundation/validation/golden-case-lifecycle/REPORT.md
[guided]: refoundation/validation/guided-case-cli/REPORT.md
[connection]: refoundation/validation/single-connect/REPORT.md
[presentation]: refoundation/validation/conversation-presentation/REPORT.md
