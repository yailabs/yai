# Core Enforcement Status

CORE.SPINE.C1 records the honest, current enforcement state of the YAI core
runtime. This is a status surface, not a capability claim. It is derived from
the code and tests present in this checkout. Where code and this table disagree,
the code wins and this table is the bug.

## Reading Rules

```text
facts are derived assertions, not operational truth
projections are views, not authority
model/provider output is proposal, not authority
execution_performed must not be claimed unless a carrier actually executes
the model_provider carrier remains skeleton/planned unless code proves otherwise
```

## Status Vocabulary

The `Current status` column uses only these values:

```text
implemented          present and exercised by smoke/tests
implemented_limited  present but bounded (test-owned, ephemeral, or partial)
inspect_only         can be read/derived but does not yet gate execution
skeleton             inspectable surface, no execution
planned              not present, scheduled
external_unknown     lives in another repo / not verifiable here
absent               not present and not yet scheduled here
```

## Enforcement Level Vocabulary

The `Enforcement level` column uses only these values:

```text
interposed             a carrier actually mediates a host effect
deterministic_control  deterministic control/record logic in-process
derived_read_model     a non-authoritative view derived from truth
review_no_execution    routes to operator review, no effect executed
fixture_like           primitive/contract material, not an evaluator
skeleton_no_execution  inspectable surface only, no effect
unknown                not yet verified in code
```

## Status Table

| Component | Current status | Evidence path | Enforcement level | Gap / next action |
|---|---|---|---|---|
| case binding | implemented | system/case/case_handle.c, include/yai/case/case_handle.h | deterministic_control | none (SPINE.51B) |
| control gate | implemented | system/control/gate.c, system/control/gate_outcome.c | deterministic_control | none |
| decision | implemented | system/control/decision.c, system/control/decision_basis.c | deterministic_control | none |
| authority scope | implemented | system/control/authority_scope.c | deterministic_control | none |
| visibility scope | implemented | system/projection/visibility_scope.c | deterministic_control | none |
| resource scope | implemented | system/effect/resource_scope.c | deterministic_control | none |
| capability lease | implemented_limited | system/control/capability_lease.c (permits_execution), system/effect/dispatch_admission.c, tests/smoke/control-lease-dispatch/ | deterministic_control | admission gate (lease+decision, fail-closed) now enforced and tested (CORE.ENFORCE.1); carrier signatures do not yet require the admission token — next increment |
| filesystem carrier | implemented | system/effect/carriers/filesystem_carrier.c | interposed | none (review-gated write executes) |
| process carrier | implemented_limited | system/effect/carriers/process_carrier.c | interposed | test-owned signal control, arbitrary PID blocked — harden to a clearer controlled-execution contract — CORE.CARRIER.1 |
| network/http carrier | skeleton | system/effect/carrier_skeleton.c | skeleton_no_execution | first interposed non-fs/non-process path — CORE.CARRIER.2 |
| database carrier | skeleton | system/effect/carrier_skeleton.c | skeleton_no_execution | remains skeleton until a carrier executes |
| git carrier | skeleton | system/effect/carrier_skeleton.c | skeleton_no_execution | remains skeleton until a carrier executes |
| service/endpoint/socket/listener carriers | skeleton | system/effect/carrier_skeleton.c, system/effect/carrier_coverage.c | skeleton_no_execution | remains skeleton until a carrier executes |
| model_provider carrier | skeleton | system/effect/carrier_skeleton.c | skeleton_no_execution | must stay skeleton/planned until code executes — replan in CORE.MODEL.1 |
| review loop | implemented | system/control/, cmd/yai (control pending/show/review/watch/wait) | review_no_execution | none (SPINE.44A-C) |
| effect hashing | implemented | system/effect/effect_hash.c | deterministic_control | none |
| receipts | implemented | system/effect/receipt.c, system/effect/receipt_guarantee.c, system/control/receipt_requirement.c | deterministic_control | none |
| journal | implemented | system/store/journal.c, system/store/journal_file.c | deterministic_control | append-only; tamper-evidence is the gap — CORE.JOURNAL.1 |
| LMDB record plane | implemented | engine/yai-engine/src/lib.rs (SPINE.34 freeze) | deterministic_control | none |
| replay | implemented | engine/yai-engine/src/reconcile.rs, journal replay (SPINE.36-39) | deterministic_control | none |
| graph relations | implemented_limited | system/graph/, lmdb_graph_relations_v0 (SPINE.41) | derived_read_model | active_minimal write path |
| RuntimeGraph | implemented_limited | system/graph/runtime_graph.c (SPINE.42-44) | derived_read_model | per-command ephemeral; resident service planned |
| DuckDB fact plane | implemented | SPINE.46-51 fact plane (frozen yai.fact.v1) | derived_read_model | none; facts are not truth |
| projections | implemented | system/projection/ (SPINE.52) | derived_read_model | none; projections are views, not authority |
| memory/context surfaces | implemented_limited | system/memory/memory_candidate.c | derived_read_model | candidate shim; consolidation is future (SPINE.59+) |
| policy rule primitives | implemented_limited | system/control/policy_rule.c, system/control/obligation.c | fixture_like | promote to a deterministic evaluator — CORE.POLICY.1 |
| policy engine | planned | (none in checkout) | skeleton_no_execution | minimal deterministic evaluator — CORE.POLICY.1 |
| C data-plane shims | implemented_limited | system/{store,graph,index,memory,projection,reconcile} | deterministic_control | transitional; long-term home is engine/yai-engine — CORE.DATA.1 |
| Rust yai-engine data-plane | implemented_limited | engine/yai-engine/src/lib.rs, engine/yai-engine/src/reconcile.rs | deterministic_control | parity/migration order vs C shims — CORE.DATA.1 |

## Honest Posture Statements

```text
No carrier other than filesystem and process executes a host effect today.
network_http, database, git, service, endpoint, socket, listener and
model_provider are skeleton surfaces (system/effect/carrier_skeleton.c) and
must report carrier_attempted: false / execution_performed: false.

Facts (DuckDB) and projections are derived read models. They explain, score,
filter and report. They do not authorize, approve, deny, execute or mutate
durable operational truth.

Policy primitives (policy_rule, obligation) exist as contract material. There is
no policy evaluator yet; policy enforcement is not claimed.

CapabilityLease is derived and inspectable (SPINE.51B). Whether it is enforced
before carrier dispatch — rather than only reported — is the open question for
CORE.ENFORCE.1.
```
