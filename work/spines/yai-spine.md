# Repository Roadmap

This is the active engineering roadmap for the current YAI core transition.

From SPINE.20 onward, `yai` uses one linear progression. Macro names are
labels inside a wave, not a second calendar.

The `interfaces` repo is no longer governed inside this roadmap. Its canonical
roadmap now lives in the `interfaces` repo at:

```text
docs/intf-studio-spine.md
```

That document owns the INTF.SPINE.* roadmap and the downstream STUDIO.SPINE.*
roadmap.

```text
SPINE.N = main delivery number
Macro = affected system areas
Subdeliveries = nested work inside that one delivery
```

## Current Status

| Repo | Role | Status | Next |
|---|---|---|---|
| `yai` | Canonical local AI operational control system. | Completed foundation through SPINE.52A Model-Native Actor / Adapter Roadmap Rebase. | SPINE.53 Projection Delta Schema. |
| `yai-dev` | Development lab, concept mine, harness and scenario workspace. | Old/current repo renamed to `yai-dev`; useful material is extracted into `yai` by explicit SPINE waves. | DEV.0 role note, then wave-coupled cleanup. |
| `console` | Operator client / TUI / human UX. | Downstream consumer of projections and interfaces. | CONSOLE.CANON.0 later. |

`interfaces` has been removed from this roadmap because it now owns a dedicated
interface/client spine in the `interfaces` repository:

```text
interfaces/docs/intf-studio-spine.md
```

That document governs both `INTF.SPINE.*` and the downstream `STUDIO.SPINE.*`
client roadmap.

## Completed Foundation

Foundation waves NEW.0 through NEW.19 are complete prehistory for the linear
SPINE.20+ roadmap. They established skeleton guards, persistent journal,
control gate, filesystem carrier, graph, memory candidate, reconcile,
projection, query, Rust engine integration, daemon IPC, daemon-backed loop,
target layout, Rust/C source movement, engine bridge split, case-world records,
live case context, observability doctrine, interaction threads and build/guard
realignment. SPINE.6A compressed engineering docs. SPINE.6B made
bidirectional yai-dev extraction and residue normalization mandatory for future
implementation waves. SPINE.20 established the local `YAI_HOME` runtime layout
and install/uninstall/doctor targets. SPINE.20A rebased the active roadmap so
pack materialization becomes the next canonical spine step. SPINE.21 made
packs first-class case materialization units and defined their format,
lifecycle and materialization boundaries. SPINE.22 freezes the filesystem and
runtime foundation before data-plane implementation begins. SPINE.23 introduces
hot state as the first live data plane: an in-process cache plus
`YAI_HOME/run/hot-state.json` snapshot. SPINE.23A expands the compressed
SPINE.23-SPINE.64 plan into smaller implementable SPINE.23-SPINE.120
deliveries and makes command surface declaration mandatory.
SPINE.24 hardens the hot-state runtime snapshot as schema
`yai.hot_state.v1`, temp-and-rename writes and missing/corrupt status handling.
SPINE.24A retroactively maps SPINE.20-SPINE.24 primitives to canonical command
views, manual command tests and expected output lines.
SPINE.25 links hot state to case session, case world, case context, active
thread, participant view and decision/receipt-driven projection staleness.
SPINE.26 makes projection freshness consumer-aware: model/agent views require
stricter refresh posture than operator, audit and debug views. SPINE.27
stabilizes the manual command surface for `yai hot status`, `yai doctor` and
`yai projection inspect`. SPINE.28 freezes hot state as a non-authoritative live
cache before durable LMDB record-plane work begins. SPINE.28A renames the
canonical core repository to `yai` and the old concept-mine repository to
`yai-dev`. SPINE.28B removes local virtualenv roots, archives README-only
ingest stubs and marks transitional C data shims as temporary engine
drain surfaces. SPINE.29 defines LMDB as durable indexed record lookup under
`YAI_HOME/store/lmdb`, adds `yai store status` and keeps hot state, journal,
graph, facts and memory separate before the write path begins. SPINE.30 adds
the Rust LMDB write path, minimal `records_by_id`, `records_by_case` and
`records_by_kind` indexes, `yai store summary` and daemon-loop mirroring from
journal to LMDB. SPINE.31 adds `yai store record get` and
`yai store record list` over the id, case and kind indexes without adding
journal fallback reads. SPINE.32 adds derived subject and receipt indexes plus
CLI list filters for both. SPINE.33 freezes LMDB CLI/manual validation.
SPINE.33A through SPINE.33I harden the carrier/control/receipt/divergence
substrate. SPINE.33J through SPINE.33M reserve retrieval, model runtime,
Context Compiler and RuntimeGraph boundaries. SPINE.34 freezes the LMDB record
plane as `yai.record.v1` durable indexed lookup before journal replay begins.
SPINE.35 hardens the journal replay boundary with diagnostic parser inspection
and no LMDB writes. SPINE.36 adds explicit journal replay to LMDB with dry-run
reporting and basic idempotent replay. SPINE.37 hardens replay idempotency with
journal identity, schema compatibility, replay cursor metadata and
`yai journal replay-status`. SPINE.38 adds `yai.replay_report.v1` durable replay
diagnostics with `journal_identity`, `compatibility`, `cursor_line`,
`records_written`, `records_duplicate`, `invalid_entries` and failed report
evidence. SPINE.39 freezes journal replay as the controlled materialization
path from append-only replay/audit residue into LMDB before graph persistence
begins. SPINE.40 defines Graph persistence as durable typed relations and
RuntimeGraph as the in-memory active case working set; HNSW finds candidate
nodes, HNSW is not graph truth, Context Compiler renders controlled views and
Projection does not disappear. Persistent truth on disk. Computational shape in memory.
SPINE.41 adds the first minimal graph relation write path:
`yai.graph_relation.v1` relations are derived from LMDB records, written to
`lmdb_graph_relations_v0` and inspected by case with
`yai graph materialize --case <case_ref>` and
`yai graph relations --case <case_ref>`. RuntimeGraph remains planned.
Ladybug integration remains future. SPINE.42 adds an active-minimal
RuntimeGraph in-memory working set loaded from graph relations with
`yai graph runtime-load --case <case_ref>`. The working set is
per-command ephemeral; resident service, HNSW and Context Compiler remain
future.
SPINE.43 adds RuntimeGraph rebuild: `yai graph rebuild --case <case_ref>
--from journal --path <journal.jsonl>` runs journal replay to LMDB, graph
relations materialization and RuntimeGraph load, while `yai graph rebuild
--case <case_ref> --from graph-relations` rebuilds from existing graph
relations. Rebuild emits `yai.runtime_graph_rebuild_report.v1`,
`runtime_graph_rebuild` and `runtime-summary` evidence. RuntimeGraph is not
durable truth. RuntimeGraph is not durable truth and resident service planned
remains the posture. The active lab path is `labs/filesystem-loop`.
SPINE.44A adds an active operator review loop. SPINE.44B adds the CLI review
interaction surface: `control pending`, `control show`, `control review --interactive`, `control watch`, `control wait`, `next_commands` and
`not_a_tty` behavior. `pending_operator` remains visible.
`subject:linenoise-terminal is prompt surface` and
`subject:operator-reviewer is review authority`; approve may execute the safe reviewed write, while deny,
defer and quarantine keep `carrier_attempted: false` and
`execution_performed: false`. The active lab path is
`labs/filesystem-loop`.

SPINE.44C aligns the review matrix across `labs/filesystem-loop` and
`labs/filesystem-loop`: approve, deny, defer, quarantine,
`pending_operator`, `next_commands`, `wait timeout`, bounded `watch`,
`carrier_attempted: false` and `execution_performed: false` are tested. Model
proposal observed, model cannot approve, and automatic proposed-op gate import
is future work.

Review matrix guard: model proposal observed; model cannot approve; automatic
proposed-op gate import is future work.

SPINE.45 freezes graph schema, `yai.graph_relation.v1`, graph materialize,
graph relations, RuntimeGraph runtime-load/runtime-summary, runtime graph
rebuild, `yai.runtime_graph_rebuild_report.v1`, fanout, fanin, neighborhood,
causal path, bounded traversal, edge-kind filter, path found, path not_found
and empty case. RuntimeGraph is not durable truth; plain output remains
parseable and color-aware graph inspection remains doctrine. HNSW future,
Context Compiler future and Ladybug future persistence integration remain
future. `review_request`, `review_decision` and `control_pending` are
graph-visible when refs exist; approve is visible and deny, defer and
quarantine preserve no-execution posture.

SPINE.45A collapses the repository documentation root canon. `docs/` is now
curated architecture documentation only, `labs/` owns experiments, runbooks,
outputs and reports, and `work/` owns spines, waves, inventories, agent notes
and archives. CLORI did not execute in this slot and remains out of scope for
this delivery.

Research Lab bridge exists at docs/research-lab-bridge.md and governs how external
Obsidian research material may be promoted into YAI repo canon.
Research index exists at docs/research-index.md as the repo-side hub for promoted
research links and future math/source traceability.
The research-derived alpha sequence is frozen by the repo math spine and the
external YAI Research and Implementation Delivery Spine:

```text
work/spines/research-math-spine.md
obsidian://open?vault=YAI%20Research%20Lab&file=00_Index%2FYAI%20Research%20and%20Implementation%20Program
```

SPINE.45B rebases the roadmap before DuckDB facts so facts, retrieval,
compiled context, model-session state, graph revision and counterfactual
semantics are first-class doctrine. No runtime behavior is implemented in this
wave. SPINE.46 now starts as a bitemporal fact plane.

SPINE.46 defines DuckDB as the analytical fact plane with schema
`yai.fact.v1`, bitemporal common fields and `yai facts status`, `yai facts
schema` and `yai facts init`. A fact is not just a row. A fact is a temporally
scoped, provenance-bearing assertion. Facts are not truth and cannot authorize,
execute, approve, deny, mutate operational truth or replace evidence. SPINE.46
initializes `fact_receipt`, `fact_decision`, `fact_projection`,
`fact_carrier_outcome`, `fact_divergence`, `fact_replay`,
`fact_runtime_graph`, `fact_model_behavior`, `fact_policy_outcome`,
`fact_memory_quality`, `fact_retrieval_quality` and `fact_provider_runtime`.
No fact extraction occurs; `facts_extracted: 0` remains visible until SPINE.47.

SPINE.47 implements the first real extraction from LMDB records into DuckDB:
`fact_receipt`, `fact_decision` and `fact_projection`. Extraction is
derivation, not migration. Records remain truth in LMDB. Facts are not truth.
The new surface is `yai facts extract --case <case_ref> --kind
receipt|decision|projection|core` and `yai facts summary --case <case_ref>`.
Fact IDs are deterministic as `fact:<kind>:<source_record_id>`, extraction is
idempotent extraction, and the second run reports duplicates. Bitemporal
columns keep `transaction_time`, `valid_time_start`, `valid_time_end` and
`known_at`; revision fields `revision_of`, `superseded_by` and `retracted_by`
remain empty because there is No fact revision yet. The valid_time_end
sentinel: 0 means open-ended.

Do not schedule future work with the old NEW numbering. The next active
delivery is SPINE.48.

## DuckDB Fact Plane Doctrine

Journal records chronology. LMDB stores record truth. Graph persistence stores
relation truth. RuntimeGraph computes over active case relations. DuckDB stores
analytical, temporal, provenance-bearing facts derived from records, graph
relations, receipts, decisions, projections, carriers, model behavior, replay
reports, runtime graph diagnostics, retrieval and memory.

The schema identity is `yai.fact.v1`. The DuckDB file lives at
`YAI_HOME/store/facts/yai-facts.duckdb`.

Every core fact row distinguishes transaction time from valid time:

```text
transaction_time
valid_time_start
valid_time_end
known_at
```

Every revisionable fact keeps correction posture:

```text
status
revision_of
superseded_by
retracted_by
```

Initial tables:

```text
fact_receipt
fact_decision
fact_projection
fact_carrier_outcome
fact_divergence
fact_replay
fact_runtime_graph
fact_model_behavior
fact_policy_outcome
fact_memory_quality
fact_retrieval_quality
fact_provider_runtime
```

Facts are not truth. They can explain, score, measure, compare, filter and
report. They cannot authorize, execute, approve, deny, mutate operational truth
or replace evidence.

Command surface:

```text
yai facts status
yai facts schema
yai facts init
```

SPINE.46 has No fact extraction. `facts_extracted: 0` is the required visible
posture; SPINE.47 starts extraction.

SPINE.47 extraction:

```text
fact_receipt
fact_decision
fact_projection
facts extract
facts summary
idempotent extraction
deterministic fact IDs
fact:<kind>:<source_record_id>
valid_time_end sentinel: 0
revision fields
No fact revision
```

## Case Runtime Semantics Doctrine

YAI is not a RAG and not a chat memory system.

YAI is a local case runtime that maintains:

```text
append-only history
bitemporal facts
typed graph dependencies
RuntimeGraph working sets
governed native and external retrieval
compiled context
model-session bindings
validated model participation
```

Events are append-only. Facts are temporally scoped, provenance-bearing
assertions. Graph corrections are append-only patches. RuntimeGraph computes
over active case state. Retrieval candidates are not evidence until resolved.
ContextFrame is not truth. ContextDelta is not memory. CaseModelSession is not
authority. KV/cache state is runner-local optimization, not YAI memory.

Canonical object taxonomy:

```text
Event:
  Something that happened in the system: attempt, decision, receipt, review,
  model output, replay, graph materialization, context compilation.

Record:
  Durable indexed residue in LMDB.

Fact:
  Temporally scoped, provenance-bearing assertion stored in DuckDB.

Interpretation:
  Semantic reconstruction by model, system or operator; not automatically truth.

Graph relation:
  Typed dependency, support, contradiction, provenance, branch or causal relation.

RuntimeGraph node/edge:
  In-memory computational shape loaded from graph relations.

Retrieval candidate:
  Candidate material found by native or external retrieval; not evidence yet.

Projection:
  Controlled consumer view.

ContextFrame:
  Temporary compiled model/operator/audit/debug context.

ContextDelta:
  Incremental patch against a prior valid context frame.

CaseModelSession:
  Binding among case, actor, model, runner, context state and policy scope.

KV/cache state:
  Runner-local optimization; never YAI truth, memory or provenance.
```

Updated operational chain:

```text
User Prompt
→ HOT request/session/thread state
→ Task / Intention Structure
→ Case Scope / Actor Scope / Policy Scope
→ Retrieval Plan
→ Native Retrieval + External Retrieval candidates
→ Retrieval Receipts
→ Candidate Normalization / Dedupe
→ RuntimeGraph Expansion
→ DuckDB Fact Filtering
→ LMDB Record Resolution
→ Filesystem / Evidence Payload Fetch
→ EvidenceFrame / RetrievalSourceFrame
→ ContextFrame Base + Delta + Task Frame
→ Model Runner
→ Model Output / Patch Proposal / Retrieval Refinement Request
→ Runtime Validation
→ Records / Facts / Graph Relations / Receipts / Review Items
```

Compact version:

```text
Vector search proposes candidates.
RuntimeGraph restores case meaning.
DuckDB filters and scores structured facts.
LMDB resolves durable records.
Filesystem resolves payload evidence.
Policy gates visibility and action.
Context Compiler renders model-consumable context.
The model proposes.
The runtime validates.
The case appends residue.
```

Text / Vector / Symbolic / Graph / Fact / Payload taxonomy:

```text
Text:
  prompt, excerpt, rendered context, model response.

Vector:
  embedding, HNSW node, similarity candidate.

Symbolic ID:
  case_ref, event_ref, record_ref, fact_ref, node_ref, doc_ref, evidence_ref,
  receipt_ref.

Graph:
  typed relation, dependency, support, contradiction, provenance, branch.

Fact:
  structured temporal assertion.

Record:
  durable indexed residue.

Payload:
  actual material such as PDF, email, contract, invoice, image, log or file.
```

Rule:

```text
Text is not truth.
Vector is not evidence.
Symbolic refs bind materials.
Graph gives meaning.
Facts make assertions queryable.
Records preserve residue.
Payloads preserve source material.
```

## Native / Attached / Federated Retrieval Doctrine

Definitions:

```text
YAI native retrieval:
  Retrieval built by YAI over case material already materialized inside the case.

User attached retrieval:
  User-provided RAG, vector DB, file index or knowledge base participating as
  a candidate source.

Organization attached retrieval:
  Team/company retriever with stronger permissions, audit and redaction needs.

External retrieval provider:
  API, DMS, database search, email search, filesystem index or external search
  service.

Federated retrieval:
  Controlled orchestration across YAI native HNSW, user RAG, org RAG,
  filesystem index, database retriever or external provider.

Managed import from retrieval:
  Explicit promotion of external result into durable case material.
```

Canonical rule:

```text
A user RAG can participate in a case.
It cannot become the case.
```

Native retrieval finds YAI-owned candidates. External retrieval finds
user/provider-owned candidates. Neither is truth. Neither is policy authority.
Neither may directly justify a YAI decision. All retrieval output must be
resolved through case scope, graph, facts, policy, provenance and evidence
before it can enter a model context or a case decision.

External retrieval output may be candidate context. It becomes durable case
material only through explicit import, receipt, fact assertion and graph
binding.

Correct chain:

```text
User RAG
→ Retrieval Receipt
→ Normalization
→ Policy / Scope Gate
→ Graph / Fact Resolution
→ Evidence Candidate Pack
→ Context Compiler
→ Model Context
```

Forbidden chain:

```text
User RAG → prompt model
```

Planned RetrievalProvider contract fields:

```text
provider_ref
provider_kind
owner_ref
case_scope
subject_scope
trust_level
read_policy
redaction_policy
result_contract
supports_filters
supports_document_fetch
supports_snippet_fetch
supports_metadata
supports_scores
supports_embeddings
supports_provenance
supports_receipts
supports_incremental_sync
```

Trust levels:

```text
untrusted
attached_low_trust
case_scoped
organization_trusted
system_native
```

Planned RetrievalResult normalized schema fields:

```text
result_ref
provider_ref
provider_kind
query_ref
score
source_uri
external_doc_ref
snippet
metadata
returned_at
provider_claimed_provenance
yai_resolution_status
```

Resolution statuses:

```text
resolved
unresolved
rejected_by_policy
outside_case_scope
needs_import
needs_human_review
evidence_candidate
claim_candidate
duplicate_of_existing
stale
redacted
unsafe
```

Planned Retrieval Receipt fields:

```text
query_ref
provider_ref
case_ref
actor_ref
timestamp
query_text
query_embedding_ref
filters_applied
results_returned
results_selected
results_rejected
rejection_reasons
policy_scope
redaction_scope
cost
latency
provenance_quality
selected_for_context
promoted_to_case_material
```

Behavior facts to track later:

```text
retriever_latency
retriever_recall_estimate
retriever_noise_rate
retriever_scope_violation
retriever_duplicate_rate
retriever_selected_result_rate
retriever_evidence_quality
retriever_policy_rejection_rate
retriever_context_usefulness
retriever_cost
retriever_staleness_rate
```

## Context Residency Doctrine

A model must not reconstruct the active case from scratch for every request.

YAI maintains case-resident runtime state, compiled context frames and
incremental context deltas. A model runner may reuse prefix, session or KV state
only when the context residency contract proves that case, actor, policy,
model, runner and freshness boundaries still match.

KV/cache state is an optimization, not memory, truth, authority or provenance.

Hierarchy:

```text
Durable Case Truth:
  Journal
  LMDB
  Ladybug
  DuckDB
  filesystem payloads

Runtime Case State:
  Hot State
  RuntimeGraph
  active facts
  policy scope
  active actor/task/thread

Context State:
  ContextFrame base
  ContextFrame delta
  task frame
  evidence frame
  retrieval source frame
  patch frame
  freshness metadata

Model Session State:
  prompt prefix
  active instruction frame
  case summary frame
  tool/result frame
  runner session id
  KV cache if supported

Model Tokens:
  actual input tokens consumed by transformer
```

Planned ContextFrame fields:

```text
context_frame_id
case_ref
actor_ref
consumer_kind
policy_scope
source_record_refs
source_fact_refs
source_graph_refs
retrieval_source_refs
token_budget
rendered_sections
freshness
```

Planned ContextDelta fields:

```text
delta_id
base_frame_id
changed_records
changed_facts
changed_graph_nodes
changed_retrieval_sources
invalidated_sections
new_sections
```

Planned CaseModelSession fields:

```text
session_id
case_ref
actor_ref
model_ref
runner_ref
provider_ref
base_frame_id
latest_frame_id
policy_scope
retrieval_provider_refs
cache_policy
session_status
```

Runner KV / cache fields:

```text
runner_cache_id
model_ref
runner_ref
context_hash
prefix_hash
tokenizer_profile
valid_for_frame_id
stale
```

Invalidation triggers:

```text
case_ref mismatch
actor_ref mismatch
policy_scope changed
redaction changed
fact revision
graph patch
retrieval provider trust changed
model_ref changed
runner_ref changed
tokenizer/template changed
context hash mismatch
```

## Temporal Graph Revision / Counterfactual Doctrine

Canonical history is not destructively modified. Facts are revised. Graph is
patched through append-only operations. Branches are derived worlds of the
case. Counterfactuals are governed simulations, not historical corrections. The
model proposes; the runtime and case validate.

Patch operations:

```text
assert
retract
supersede
annotate
invalidate
branch
merge_request
correction
counterfactual_intervention
```

Revision statuses:

```text
current
superseded
retracted
stale
contested
historical_only
branch_only
counterfactual
needs_review
```

Dependency closure:

```text
changed fact/relation/patch
→ affected facts
→ affected graph relations
→ affected projections
→ affected ContextFrames
→ affected memory units
→ affected retrieval units
→ affected decisions/model outputs
```

Computation rule:

```text
Safety-critical invalidation is synchronous.
Analytics, retrieval, memory and report recomputation may be asynchronous but
must be stale-aware.
```

## Computational Cost Doctrine

Retrieval:

```text
Native HNSW is sublinear approximate candidate retrieval but has build/update
cost and memory overhead. External retrieval has provider latency, cost and
trust overhead. Federated retrieval should prefer parallel provider calls later
but may be deterministic/sequential in v0.
```

RuntimeGraph:

```text
Traversal cost is O(nodes_visited + edges_scanned). It must be bounded by
max_depth, max_edges, case scope, subject scope and edge-kind filters.
```

DuckDB:

```text
Excellent for analytical facts, reports, scoring and batch filtering. It should
not become the per-token hot path.
```

Context Compiler:

```text
Main costs are candidate count, payload fetch, token estimation, dedupe,
ranking and packing. Hard filters must run before expensive payload fetch and
rendering.
```

Model runner:

```text
Main costs are prefill tokens, decode tokens, KV cache memory and backend
speed. Reduce input tokens first; then optimize generation.
```

## Case Runtime Invariants

```text
Events are append-only.
Facts are bitemporal, revisionable assertions.
Graph corrections are patches, not destructive edits.
RuntimeGraph is computation, not durable truth.
Retrieval candidates are not evidence until resolved.
External retrieval cannot enter model context without receipt/scope/provenance gating.
External retrieval cannot become case material without explicit import.
ContextFrame is not truth.
ContextDelta is not memory.
CaseModelSession is not authority.
KV cache is not YAI memory.
A model cannot approve its own operation.
A model proposal is not execution.
Vector proximity cannot justify a decision.
Policy gates before context inclusion and before action.
Revision invalidates dependent projections/context/memory/retrieval where needed.
```

## Canonical Macro Labels

```text
WORLD
RESIDUE
DATA
VIEW
CONTROL
MEMORY
RETRIEVAL
RECONCILE
HOST
MODEL
OBSERVABILITY
EXTERNAL
```

These labels describe impact. They are not numbered workstreams.

## Operational Wave Contract

Future implementation waves operate on both `yai` and corresponding `yai-dev`
residue when the touched concept already exists there.

Required rule:

```text
A wave is not complete until the corresponding yai-dev residue has been classified.
```

Each relevant SPINE.N must read the old files, extract or rewrite useful
concepts into the new grammar, update inventory, normalize yai-dev residue and
validate. Details live in `wave-template.md` and
`operational-extraction-contract.md`.

## Pack Materialization Doctrine

Packs are case materialization units.

A pack is not a marketplace product, DLC or prompt bundle. A pack is a way to
materialize methods, procedures, policies, templates, projection rules, output
schemas, memory seeds, actors/tools and validation fixtures into a case.

Pack kinds:

```text
native_yai
user
organization
vertical
pilot
reference
experimental
compat
```

SPINE.21 defines the pack format, lifecycle and materialization rules.

## Data Context Runtime / RuntimeGraph Doctrine

YAI separates durable truth from runtime computation.

```text
Truth lives on durable planes.
Computation happens in runtime working sets.
```

The canonical data/context split is:

```text
LMDB stores records.
Ladybug stores relations.
DuckDB stores facts.
RuntimeGraph computes over the active case.
HNSW finds candidate nodes.
Context Compiler renders controlled views.
```

Ladybug remains graph persistence, relation truth and rebuild contract.
RuntimeGraph is the in-memory active case graph / working set loaded from
durable truth. HNSW is a rebuildable proximity index over retrieval units bound
to RuntimeGraph nodes. Context Compiler builds temporary controlled projections
and model contexts from RuntimeGraph, retrieval candidates, policy, facts and
token budgets.

The retrieval roadmap therefore includes:

```text
SPINE.58D HNSW Candidate -> RuntimeGraph Expansion
```

This means HNSW finds vectors. RuntimeGraph gives meaning. ContextFrame remains
a temporary compiled artifact, not truth, memory, storage or graph.

## SPINE.20-SPINE.120 Linear Roadmap

SPINE.N is one implementable and manually verifiable delivery. Macro labels stay
inside each wave; they are not parallel schedules. A wave that changes command
surface must list added, changed, removed/deprecated commands, manual command
tests and expected key output lines in its delivery box.

SPINE.23 was completed before this expansion. It delivered more than the new
SPINE.23 title alone implies, so its implementation is recorded as:

```text
SPINE.23  Hot State Doctrine + ABI                                  done
partial coverage: SPINE.24 Hot State Runtime Snapshot
partial coverage: SPINE.25 Hot State Case Session / Context Integration
partial coverage: SPINE.26 Hot State Projection Freshness Integration
partial coverage: SPINE.27 Hot State CLI + Manual Validation
```

SPINE.24 through SPINE.26 reconcile the partial hot-state behavior already
landed in SPINE.23 and make it explicit, validated and manually inspectable.

```text
SPINE.20  Local Runtime Layout                                      done
SPINE.20A Pack Roadmap Rebase                                       done
SPINE.21  Pack Materialization Doctrine                             done
SPINE.22  Filesystem & Runtime Layout Freeze                        done
SPINE.23  Hot State Doctrine + ABI                                  done
SPINE.23A Roadmap Expansion + Command-Test Contract                 done
SPINE.24  Hot State Runtime Snapshot                                done
SPINE.24A Retroactive Command Surface Recovery                      done
SPINE.25  Hot State Case Session / Context Integration              done
SPINE.26  Hot State Projection Freshness Integration                 done
SPINE.27  Hot State CLI + Manual Validation                         done
SPINE.28  Hot State Freeze                                          done
SPINE.28A Repository Identity Cutover                                done
SPINE.28B Internal Source Surface Cleanup                            done

SPINE.29  LMDB Record Plane Doctrine + Schema                       done
SPINE.30  LMDB Record Write Path                                    done
SPINE.31  LMDB Record Read / Query Path                             done
SPINE.32  LMDB Case / Subject / Receipt Indexes                     done
SPINE.33  LMDB CLI + Manual Validation                              done
SPINE.33A Control / Carrier Substrate Primitives                    done
SPINE.33B Operation Dispatch + Multiplex v0                         done
SPINE.33C Carrier Contract v1 + Filesystem Adapter                  done
SPINE.33D Process Carrier v0 / Signal Control                      done
SPINE.33E Host Observation Probe v0 / Bypass Check                 done
SPINE.33F Carrier Coverage Matrix + Mode Taxonomy                  done
SPINE.33G Non-Process Carrier Skeletons                            done
SPINE.33H Carrier Outcome Harness                                  done
SPINE.33I Carrier Receipt / Divergence Hardening                   done
SPINE.33J Retrieval and Model Runtime Roadmap Rebase                done
SPINE.33K Context Compiler / Retrieval / MTP Roadmap Correction     done
SPINE.33L Provider Runtime / LAN Target Surface v0                  done
SPINE.33M Data Context Runtime / RuntimeGraph Doctrine              done
SPINE.34  LMDB Record Plane Freeze                                  done

SPINE.35  Journal Replay Doctrine + Parser Hardening                done
SPINE.36  Journal Replay to LMDB                                    done
SPINE.37  Replay Idempotency + Schema Version Handling              done
SPINE.38  Replay Diagnostics / Rebuild Report                       done
SPINE.39  Journal Replay Freeze                                     done

SPINE.40  Graph Persistence / RuntimeGraph Doctrine + Schema         done
SPINE.41  Graph Relation Write Path                                  done
SPINE.42  RuntimeGraph In-Memory Working Set                         done
SPINE.43  RuntimeGraph Rebuild from Journal / LMDB / Graph Store     done
SPINE.44  RuntimeGraph Query / Causal Path / Diagnostics             done
SPINE.44A Operator Review / Deferred Action Loop v0                  done
SPINE.44B CLI Review Interaction Surface                             done
SPINE.44C Review Loop Test Matrix + Lab Alignment                    done
SPINE.45  Graph + RuntimeGraph Freeze                                done
SPINE.45A Documentation Root Canon Collapse                           done
SPINE.45B Case Runtime Semantics / Retrieval Federation / Context Residency Roadmap Rebase done

SPINE.46  DuckDB Fact Plane Doctrine + Bitemporal Schema              done
SPINE.47  Receipt / Decision / Projection Facts                     done
SPINE.48  Model Behavior / Policy Outcome Facts                     done
SPINE.49  Memory / Divergence / Carrier Facts                       done
SPINE.50  Fact Reports + CLI Manual Validation                      done
SPINE.51  Fact Plane Freeze                                         done
SPINE.51B CaseHandle / CapabilityLease Boundary                     done

SPINE.52  Live Projection Frame Schema                              done
SPINE.52A Model-Native Actor / Adapter Roadmap Rebase               done
SPINE.53  Projection Delta Schema                                   planned
SPINE.54  Projection Freshness / Staleness Engine                   planned
SPINE.55  Projection Authority / Redaction Alignment                planned
SPINE.56  Projection Memory / Divergence Patch                      planned
SPINE.57  Model-Visible Delta Smoke + CLI                           planned
SPINE.58  Live Projection Freeze                                    planned

SPINE.58A Context Compiler / Retrieval Boundary Doctrine            planned
SPINE.58B Retrieval Unit Schema + Embedding Manifest                planned
SPINE.58C HNSW Candidate Index Build / Rebuild                      planned
SPINE.58D HNSW Candidate -> RuntimeGraph Expansion                  planned
SPINE.58E Context Assembly / Rerank / Token Budget Packing          planned
SPINE.58F Retrieval Residue + Cost / Recall Facts                   planned
SPINE.58G Context Compiler / Retrieval Freeze                       planned
SPINE.58H Native vs Attached Retrieval Doctrine                      planned
SPINE.58I Retrieval Provider Contract                                planned
SPINE.58J Retrieval Receipt Schema                                   planned
SPINE.58K Federated Candidate Normalization / Dedupe                 planned
SPINE.58L Retrieval Policy / Scope Gate                              planned
SPINE.58M External Candidate -> Case Import Boundary                 planned
SPINE.58N Context Residency Doctrine                                 planned
SPINE.58O ContextFrame Base / Delta Lifecycle                        planned
SPINE.58P CaseModelSession Binding                                   planned
SPINE.58Q Context State Invalidation                                 planned
SPINE.58R Incremental + Federated Model-Visible Context Smoke        planned
SPINE.58S Model-Visible YAI Grammar / Actor Participation Contract   planned
SPINE.58T Model-Native Training Corpus / Lab Evidence Boundary       planned

SPINE.59  Memory Consolidation Doctrine + Basis Model               planned
SPINE.60  Receipt-Backed Memory Consolidation                       planned
SPINE.61  Graph-Derived Memory Consolidation                        planned
SPINE.62  Policy / Subject Scoped Memory                            planned
SPINE.63  Memory Freshness / Confidence / Contradiction             planned
SPINE.64  Memory Projection Rules + Quality Facts                   planned
SPINE.65  Memory Consolidation Freeze                               planned
SPINE.65A Temporal Graph Revision Doctrine                           planned
SPINE.65B Graph Patch Operation Schema                               planned
SPINE.65C Retraction / Supersession Fact Binding                     planned
SPINE.65D Dependency Closure / Projection Invalidation               planned
SPINE.65E Branch / Fork / Merge Case Graph Semantics                 planned
SPINE.65F Counterfactual Case World Harness                          planned
SPINE.65G Graph Revision / Counterfactual Freeze                     planned

SPINE.66  Cross-Plane Reconcile Doctrine                            planned
SPINE.67  Hot vs Record Consistency                                 planned
SPINE.68  Record vs Graph Consistency                               planned
SPINE.69  Graph vs Memory Consistency                               planned
SPINE.70  Projection Stale / Authority Overclaim Detection          planned
SPINE.71  Reconcile Reports + CLI                                   planned
SPINE.72  Reconcile Freeze                                          planned

SPINE.73  Observability Facts Doctrine                              planned
SPINE.74  Trace / Span Records                                      planned
SPINE.75  Timing Samples                                            planned
SPINE.76  Freshness / Case View Quality Facts                       planned
SPINE.77  Model Behavior Facts                                      planned
SPINE.78  Memory Quality Facts                                      planned
SPINE.79  Observability Debug Commands                              planned
SPINE.80  Data Plane Milestone Freeze                               planned

SPINE.81  Ingest Source Material                                    planned
SPINE.82  Claim / Observation / External Receipt Import             planned
SPINE.83  Policy Material Intake                                    planned
SPINE.84  Model Output Intake                                       planned
SPINE.85  Ingest Freeze                                             planned

SPINE.86  Provider Subject Posture                                  planned
SPINE.87  Model Subject Posture                                     planned
SPINE.88  Local Provider Attachment                                 planned
SPINE.89  Remote Provider Attachment                                planned
SPINE.90  Model Output Claim Import                                 planned
SPINE.91  Naked Local Model Case Experiment                         planned
SPINE.92  Model Behavior Trace                                      planned
SPINE.93  Model Carrier                                             planned
SPINE.93A Model Runtime / Runner Boundary Doctrine                  planned
SPINE.93B Model Capability Probe + Decoding Manifest                planned
SPINE.93C Runner Adapter Contract + Invocation Envelope             planned
SPINE.93D MTP / Speculative Decoding Policy + Metrics               planned
SPINE.93E Runner Receipt + KV / Context State Boundary              planned
SPINE.93F Model Runtime Boundary Freeze                             planned
SPINE.93G Model Participation Profile + Behavior Manifest            planned
SPINE.93H Actor Adapter Contract + Evaluation Boundary               planned
SPINE.93I Specialist Model / Architecture Lab Boundary               planned
SPINE.93J Model-Native Actor Boundary Freeze                         planned
SPINE.94  Model / Provider Freeze                                   planned

SPINE.95  Policy Pack Skeleton                                      planned
SPINE.96  Policy Projection to Model                                planned
SPINE.97  Policy Compliance Experiment Harness                      planned
SPINE.98  Legal / Accounting / Business Domain Packs                planned
SPINE.99  Policy Memory                                             planned
SPINE.100 Unknown Provider Policy Handling                          planned
SPINE.101 Multi-Model Comparative Case Test                         planned
SPINE.102 Human Review / Authority Binding                          planned
SPINE.103 Case Audit Packet                                         planned
SPINE.104 Policy-Control Freeze                                     planned

SPINE.105 Filesystem Carrier Hardening                              planned
SPINE.106 Process Carrier                                           planned
SPINE.107 Host Observation Probes                                   planned
SPINE.108 Network / HTTP Carrier                                    planned
SPINE.109 Database Carrier                                          planned
SPINE.110 Repository / Git Carrier                                  planned
SPINE.111 Carrier Receipts Hardening                                planned
SPINE.112 Recovery / Compensation Posture                           planned
SPINE.113 Daemon Op Execution over IPC                              planned
SPINE.114 Host-Control Freeze                                       planned

SPINE.115 Protocol Fixtures from Core Evidence                      planned
SPINE.116 Interfaces Handoff Prep                                   planned
SPINE.117 Core Conformance Harness                                  planned
SPINE.118 Console Projection Handoff Prep                           planned
SPINE.119 YAI-dev Harness Prep                                      planned
SPINE.120 Local Core Milestone Freeze                               planned
```

SPINE.45B roadmap rebase notes:

```text
SPINE.46 starts DuckDB as a bitemporal fact plane.
SPINE.58A-G define native Context Compiler / Retrieval.
SPINE.58H-R extend retrieval to external/federated sources and define context
residency so models do not reconstruct the active case from scratch every turn.
SPINE.65A-G must land before Cross-Plane Reconcile because reconcile must know
revision, branch and counterfactual posture.
```

SPINE.46 common fact columns:

```text
fact_id
case_ref
subject_ref
asserted_by_event_ref
source_record_refs
source_graph_refs
evidence_refs
transaction_time
valid_time_start
valid_time_end
known_at
status
revision_of
superseded_by
retracted_by
confidence
authority_scope
source_record_id
source_record_kind
source_schema
fact_schema
created_at_unix_ms
```

Rule:

```text
A fact is not just a row.
A fact is a temporally scoped, provenance-bearing assertion.
Facts are not truth.
```

SPINE.46 initializes `yai.fact.v1` with `yai facts status`,
`yai facts schema` and `yai facts init`. It creates these schema tables only:

```text
fact_receipt
fact_decision
fact_projection
fact_carrier_outcome
fact_divergence
fact_replay
fact_runtime_graph
fact_model_behavior
fact_policy_outcome
fact_memory_quality
fact_retrieval_quality
fact_provider_runtime
```

SPINE.46 has No fact extraction and reports `facts_extracted: 0`.

SPINE.47 derives `fact_receipt`, `fact_decision` and `fact_projection` from
LMDB records through `facts extract` and `facts summary`. Extraction is
idempotent extraction, Facts are not truth, deterministic fact IDs use
`fact:<kind>:<source_record_id>`, bitemporal columns are populated, revision
fields stay empty and valid_time_end sentinel: 0 means open-ended. No fact
revision is implemented in this wave.

SPINE.48 derives `fact_model_behavior` and `fact_policy_outcome` from LMDB
records. `core` remains receipt + decision + projection. `behavior` means
model_behavior + policy_outcome. `all` means core + behavior.

```text
yai facts extract --case <case_ref> --kind model_behavior
yai facts extract --case <case_ref> --kind policy_outcome
yai facts extract --case <case_ref> --kind behavior
yai facts extract --case <case_ref> --kind all
```

Guard vocabulary: kind model_behavior, kind policy_outcome, kind behavior.

Model behavior facts measure model output posture, not authority. Policy
outcome facts measure policy/control posture, not policy authority. Facts are
not truth. model proposal is not execution. model cannot approve. automatic
proposed-op gate import is future work. `authority_overclaim`,
`unsupported_claim`, `review_required` and `policy_outcome` are analytical
fields only. The classifier is conservative and uses structured record fields
or stable markers only; No LLM-based classification is used.

SPINE.49 derives `fact_carrier_outcome`, `fact_divergence` and
`fact_memory_quality` from LMDB records. `operational` means carrier_outcome +
divergence + memory_quality. `all` means core + behavior + operational.

```text
yai facts extract --case <case_ref> --kind carrier_outcome
yai facts extract --case <case_ref> --kind divergence
yai facts extract --case <case_ref> --kind memory_quality
yai facts extract --case <case_ref> --kind operational
```

carrier facts measure carrier posture. divergence facts are not reconcile
action. memory facts are not memory. Facts are not truth. Bitemporal fields and
idempotent extraction remain mandatory.

SPINE.50 adds `yai facts report --case <case_ref>` and section-specific report
views for `--section receipts`, `--section decisions`, `--section projections`,
`--section policy`, `--section carriers`, `--section divergence`,
`--section memory` and `--section model`.

`facts summary` is counts. `facts report` is interpreted compact sections over
existing DuckDB facts only. facts reports are not truth, not audit packets, not
reconcile actions and not memory consolidation. Zero divergence/model facts are
valid when source records are absent: `none_observed` and `no_model_records`.
Memory reports include `memory_is_truth: false`.

SPINE.51 freezes the first DuckDB Fact Plane block as Fact Plane Freeze. The
frozen `yai.fact.v1` surface covers schema, init, extraction, `facts summary`,
`facts report`, guard, smoke and manual matrix for `fact_receipt`,
`fact_decision`, `fact_projection`, `fact_carrier_outcome`, `fact_divergence`,
`fact_model_behavior`, `fact_policy_outcome` and `fact_memory_quality`.

Facts remain analytical, temporal, provenance-bearing assertions over existing
records, relations, projections, receipts, decisions, carriers, memory
candidates and runtime diagnostics. Facts are not record truth, graph truth,
memory, policy authority, reconcile action or model authority.

The freeze keeps bitemporal fields `transaction_time`, `valid_time_start`,
`valid_time_end` and `known_at`; revision fields `revision_of`,
`superseded_by` and `retracted_by`; status posture values `current`,
`superseded`, `retracted`, `stale`, `contested`, `historical_only`,
`branch_only`, `counterfactual`, `needs_review` and `unknown`; and explicit
`facts_are_truth: false` plus `memory_is_truth: false`. Missing source records
are honest: zero divergence facts report `none_observed`, and zero model
behavior facts report `no_model_records`. Extraction remains idempotent
extraction.

SPINE.66-72 reconcile now includes:

```text
record/fact revision consistency
graph patch consistency
projection invalidation consistency
memory basis after retraction
retrieval candidate provenance consistency
context session invalidation consistency
counterfactual branch isolation
```

## Context Compiler / Retrieval Doctrine

Projection does not disappear. Projection evolves from a static controlled view
into a compiled live context: scoped, redacted, freshness-aware and
consumer-aware.

Context Compiler is the YAI-owned, model-neutral system that assembles a
projection/model context from records, graph persistence, RuntimeGraph, facts,
memory, retrieval candidates, policy and token budget. A compiled model context
is not truth. It is not memory, not the record plane, not the graph and not
policy authority.

Canonical statement:

```text
compiled model context is not truth
```

Canonical chain:

```text
Journal / LMDB / Ladybug / DuckDB
        ↓ rebuild/load
RuntimeGraph
        ↓ retrieval bindings
HNSW
        ↓ candidates
RuntimeGraph expansion + policy/facts filtering
        ↓
Context Compiler
        ↓
Compiled Projection / Model Context
        ↓
Model Runner
```

Use terms carefully:

```text
projection = generic controlled view
model context = projection rendered for a model/provider/runner
ContextFrame = temporary compiled artifact, not durable truth
```

HNSW is candidate retrieval. HNSW is not graph. HNSW is not memory. HNSW is not decision authority.
HNSW is not policy authority. HNSW is a rebuildable
proximity index over retrieval units; it stores vectors and refs, not
authority.

Future query work remains case / subject / policy scoped retrieval and must
return candidates with provenance, not final answers.

Canonical rule:

```text
A YAI decision may use HNSW/vector-retrieval candidates,
but no YAI decision may be justified by vector proximity alone.
```

Operational chain:

```text
LMDB / Ladybug / DuckDB / Memory
        ↓
Retrieval Projection Units
        ↓
Embedding Manifest
        ↓
HNSW Candidate Index
        ↓
Candidate Set
        ↓
Policy + Case Scope + Graph + Facts Filtering
        ↓
Context Compiler
        ↓
Model Context
```

Context assembly uses hard filters before scoring: case scope, subject scope,
authority, redaction, freshness, policy compatibility and operation
compatibility. Scoring is composite: vector similarity, graph relevance,
freshness, behavior facts, risk penalty and redundancy penalty.

## Model Runtime / Runner Doctrine

YAI may own runner boundaries and adapters. YAI should not initially become a
universal inference engine. YAI treats engines/runners as subjects invoked
through model carrier contracts. A runner produces outputs and receipts, not
case authority.

Boundary phrases for future guards: Model Runtime / Runner Boundary; not universal inference engine; runner output is not authority.

Boundary:

```text
Inference engine:
  loads model
  runs tokens
  manages KV/cache/model-specific execution

YAI:
  case binding
  projection
  authority
  prompt/invocation envelope
  model carrier
  provider/runner subject
  receipt
  model output import
  behavior facts
  memory/reconcile consequences
```

MTP belongs to model runtime / runner / decoding strategy. MTP is not retrieval.
MTP is not memory. MTP is not policy authority. MTP is not case truth.

```text
HNSW makes the model read less.
MTP makes the model write faster.
```

YAI tracks MTP as a runner capability and measured behavior: supports_mtp,
mtp_mode, mtp_depth, requires_mtp_weights, requires_draft_model,
backend_support, acceptance_rate, fallback_mode, structured_output_risk,
latency_delta and tokens_per_second_delta. MTP may accelerate generation; it
must not change output authority.

## Model-Native Actor / Adapter Doctrine

YAI should not move system authority, durable truth, policy enforcement,
memory truth or carrier execution into model weights.

YAI may train, adapt, prompt or profile models to become better YAI
participants.

A YAI-native model is not YAI. A YAI-native model is a model, adapter,
specialist model or runner profile trained or configured to better follow YAI
case, actor, context, policy, evidence, decision and receipt contracts.

Runtime-owned:

```text
authority
policy enforcement
case truth
record truth
graph truth
fact derivation
memory basis
carrier dispatch
audit residue
approval/review authority
```

Model-adaptable:

```text
actor discipline
case interpretation
context consumption
structured output
tool intent formatting
evidence-aware reasoning
policy-reading behavior
boundary self-reporting
critic/judge posture
routing recommendations
```

Model-native YAI work starts at Level 3 and above:

```text
Level 3:
  Actor adapters, LoRA, specialization profiles and behavior manifests.

Level 4:
  YAI protocol grammar, schema-aware output, structured model contracts and
  optional future protocol-token research.

Level 5:
  Small specialist models for context compilation, critic/judge, router,
  tool-intent classification and user/case simulation.

Level 6:
  Experimental model architecture research only. This belongs first in
  yai-dev/DGX labs and is not a foundation-model commitment.
```

Rules:

```text
A model may learn YAI grammar.
A model may learn YAI actor behavior.
A model may propose YAI-shaped decisions.
A model may produce structured tool intent.
A model may evaluate boundary compliance.

A model cannot become authority.
A model cannot approve its own operation.
A model cannot mutate durable truth directly.
A model cannot replace records, graph, facts, policy, memory or audit.
A model output is still a proposal until runtime validation.
```

Context residency note:

```text
Logical context residency may reduce model-visible tokens before KV/cache
residency exists.

KV/cache residency is runner-local optimization, not YAI memory, truth,
authority or provenance.

Token reduction can be accepted as early evidence.
Latency reduction must not be claimed until measured at runner/provider level.
```

SPINE.52A is a roadmap/doctrine update only. Command Surface: no command
surface changes.

## Provider Runtime / LAN Target Surface Doctrine

The normal future workflow should not require manual provider launch as normal
workflow. Manual provider launch remains a development fallback.

SPINE.33L adds the first inspectable provider runtime surface:

```text
runtime target: local / lan / external
device registry: YAI_HOME/config/devices.jsonl
provider supervisor: planned
provider start dry-run: active
provider logs: YAI_HOME/log/providers
model catalog: not_scanned / planned
model routing: planned
context compiler: planned
retrieval_hnsw: planned
decoding acceleration: planned
```

Rules:

```text
LAN device is not a provider.
LAN device is a runtime target.
Provider runs on a runtime target.
Model belongs to a provider instance.
Case attaches to provider instance.
```

SPINE.33L exposes dry-run plan posture only. It performs no remote start in
SPINE.33L, starts no provider process, performs no LAN discovery, scans no
model files, probes no GPU and invokes no model.

## Implementation Wave Rule

Each SPINE.N must be small enough to implement, validate and manually inspect
in one delivery. A wave may have subtasks, but it must not contain an entire
backend, model program or host-control family. If a proposed wave needs several
independent command surfaces, persistence paths, rebuild paths or smoke suites,
split it before implementation.

Every future wave delivery must include a Command Surface section. If the wave
changes no command surface, it must say so explicitly.

## External Repo Tracks

These tracks remain later and downstream from this `yai` roadmap:

| Track | Meaning |
|---|---|
| DEV | Keep `yai-dev` as lab, workshop, concept mine, provider/scenario harness and release rehearsal space. |
| CONSOLE.CANON | Align operator UX to projections and interfaces. |

The `interfaces` track is no longer listed here. It is governed by the dedicated
INTF + STUDIO spine in the `interfaces` repository:

```text
docs/intf-studio-spine.md
```

## Non-Goals

This roadmap does not touch `interfaces`, `console` or `yai-studio`, implement
data planes, or make archived docs active again.

It also does not govern the `interfaces` roadmap anymore. The `interfaces`
repo now owns its own INTF + STUDIO spine at:

```text
interfaces/docs/intf-studio-spine.md
```

## SPINE.44 RuntimeGraph Query

SPINE.44 RuntimeGraph Query / Causal Path / Diagnostics adds fanout, fanin,
neighborhood and causal path diagnostics as bounded traversal over active-case
RuntimeGraph data. RuntimeGraph is not a generic graph database. The edge-kind
filter is supported for fanout, fanin and neighborhood; max-depth bounds path
diagnostics. Plain output remains parseable and color-aware graph inspection is
future console/studio doctrine under `labs/filesystem-loop`.

## SPINE.44A Operator Review Loop

SPINE.44A adds an active review/deferred/quarantine loop before Graph +
RuntimeGraph Freeze. A filesystem write can enter `require_review`, become
`pending_operator`, and resolve by approve, deny, defer or quarantine.
`subject:linenoise-terminal is prompt surface`; operator reviewer authority is
separate. Deny, defer and quarantine keep `carrier_attempted: false` and
`execution_performed: false` under `labs/filesystem-loop`.

## SPINE.51B CaseHandle / CapabilityLease Boundary

SPINE.51B introduces runtime-resolved CaseHandle, SubjectHandle,
AuthorityScope, VisibilityScope, ResourceScope and CapabilityLease posture.

```text
refs are identifiers, not authority
bindings are relations, not capabilities
handles are runtime-resolved posture
capability leases are bounded operation permissions
```

Command surface:

```bash
yai case resolve --case <case_ref> --subject <subject_ref>
yai case scope --case <case_ref> --subject <subject_ref>
yai capability derive --case <case_ref> --subject <subject_ref> --operation <operation_family> --resource <resource_ref>
```

`case resolve` produces handle posture. `case scope` separates AuthorityScope
from VisibilityScope. `capability derive` intersects subject role, authority,
ResourceScope, policy basis and current generation posture. It reports
`subject_lacks_execute_authority`, `resource_outside_scope` and carrier
dispatch allowed posture instead of treating refs or bindings as permission.

SPINE.52 Live Projection Frame Schema should consume resolved visibility and
authority scopes. Future carriers should consume CapabilityLease/ResourceScope
before dispatch.
