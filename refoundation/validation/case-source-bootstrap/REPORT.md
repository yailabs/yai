# CASE.SOURCE.BOOTSTRAP.0 — bounded source acquisition

Baseline: `0e302b22968e34588b6b79ffe5c846d32397f015` (`master`,
`origin/master` and remote master reconciled before mutation).
Intended commit: `feat: bootstrap governed Case source acquisition`.
Pre-publication state: implementation and deterministic/product qualification
PASS; publication requires staged-diff inspection, isolated commit, push and
remote equality. The containing commit's SHA belongs in the post-commit handoff,
not in this report. This report is bounded evidence; ROADMAP owns live status.

## Archaeology and chosen ownership

Current authority inspected: ROADMAP, executable architecture, semantic-state
target, cumulative runbook, H8 governance intake, W9 binding/EffectivePolicy,
W10 admission, GOVERNANCE.COGNITIVE.CONTEXT.0, Golden and RECALL.TRACE.0.
Resource admission, confined discovery/content, SQLite/HTTP/MCP, policy catalog,
Principal/Participant, Workflow/progression and canonical replay were inspected.

The legacy `yai-dev` checkout was at
`5c1c7b9d099eea9f2947146cd821d6501c4a6ddf` (drained historical tree).
Relevant historical source at `45c36bd0c` included
`tools/gen/deterministic_governance_ingestion.py`,
`lib/exec/runtime/source_ingest.c` and
`tests/integration/source-plane/source_owner_ingest_bridge_v1.sh`.
The useful properties were staged intake, explicit source identity/provenance,
bounded validation and mediated acquisition. Current H8 and resource contracts
are stronger executable owners for these properties. Historical workspace/source
planes, caller-hint default acceptance, token-prefix trust and ingress registry
ownership were not recovered. The current drained layout is not permission to
restore those owners. No historical source tree was copied.

Before: separate product actions existed for policy intake, resource attachment,
discovery, content admission and observations; they did not compose a durable
declared perimeter with source-level coverage/resume.

After:

```text
declared perimeter / roles (Case history)
    → inventory
    → exact initial policy files → existing policy candidate/catalog
    → explicit validation/publication/binding → EffectivePolicy
    → existing current Operation/Decision/resource acquisition
    → exact content admission or observation backing
    → canonical source progress → replay-derived inventory/resume
```

There is one source frontier under existing resource access and product
application ownership. Policy semantics/catalog, content, observations and
authority retain their owners. Three source files separate the existing semantic
contract, transactional store composition and product orchestration; they are not
three registries or independent stores. Workflow was not needed as a second job
ledger: compact source progression is already reconstructible from the Case.

## Contracts and security boundary

`yai.source_perimeter.v1` accepts an explicit name, linked Participant, optional
existing Resource definitions and sources with logical name, exact resource,
action, roles, media declaration and bootstrap flag. Inputs are bounded to 64 KiB
and 128 sources per Case. Local files/directories and existing SQLite/HTTP
requests are supported. Positive SQLite acquisition captures bounded result
metadata; the HTTP oracle is deliberately denied. This does not qualify generic
MCP/wiki/GitHub/cloud connectors or whole-database capture.

The authenticated Tenant Owner must be linked to the exact declared Participant.
Initial setup authority is available only before any policy has ever been bound
to the Case. It reads only an explicitly declared policy-bearing regular file,
inside the existing confined resource envelope. No sibling crawl is authorized.
Catalog publication and Case binding remain explicit. Unbinding/revoking does not
re-enable setup. Ordinary sources remain pending until EffectivePolicy is READY;
each acquisition then uses current typed admission. Source visibility also uses
the same current EffectivePolicy evaluator, not a new decider or historical ALLOW.
An unrecorded request cannot fabricate canonical source-provenance evidence.

Roles `policy`, `knowledge`, `operational` are intent, not authority. Dual-role
setup references the same original PolicySourceArtifact and revision. A valid
policy-shaped JSON document declared only as knowledge is ordinary exact content:
the product test verifies no PolicyArtifact appears and publication refuses.
No classification, inference, OCR or knowledge derivation runs.

Source identity binds Case, logical name/perimeter, Principal/Participant,
resource/configuration, action, roles and media/setup declarations. Revision
identity binds that source plus sorted path/digest/byte-count material; different
retry observation/admission IDs do not invent a different content revision.
Progress binds source, attempt and exact predecessor. Stale progress refuses.
An acquired revision must close against the exact canonical acquisition
observation and every admitted item; forged empty/full coverage is rejected.

Inventory is Owner/declaring-Participant scoped. Its denominator is explicit
declarations, never an unknown company perimeter. It distinguishes declared,
acquiring, acquired, denied, awaiting-review, inaccessible, needs-processing and
revoked. Governance readiness and incomplete ordinary acquisition are independent.
Perimeter declaration imports existing Resources and then declarations; it is
idempotent but not an atomic all-or-nothing multi-resource transaction. A failed
declaration can be reissued; inventory covers only declarations actually admitted.

## Qualified product story and invariant results

The actual application `./yai`, real LMDB, confined files and real SQLite were
used. The denied HTTP endpoint is a local resource peer, **not a model fixture**.

| Oracle | Observed result |
|---|---|
| Policy-only | A second fresh Case acquires its policy original, explicitly publishes/binds and reports READY without an ordinary corpus. |
| Full perimeter | Initial six declarations include dual-role policy, two documentary files, missing file, SQLite metadata and denied HTTP. Before publication only policy is acquired. Afterwards: four acquired, one denied, one inaccessible. |
| Incremental | New exact sources enter the existing governed Case without setup authority. A policy-shaped knowledge-only source remains content, not policy. |
| Dual role | One original policy backing is used for policy + knowledge; knowledge produces no claims or separate byte copy. |
| Denied / sibling | HTTP request count is zero. Forbidden remote payload and undeclared sibling marker are absent from every retained file in the disposable YAI home. Engine negatives refuse sibling setup and setup re-entry. |
| Partial/resume | `--limit 1` stops after one ordinary source; a new process resumes the remainder. The missing source is later restored and acquired. Completed exact sources are skipped; replaying a completed source adds no generation. |
| Duplicate/update | Different logical names with identical bytes retain different source/revision identity and equal content digest. Same source unchanged refresh retains revision and exact admission backing. Changed bytes create R2; R1 remains independently readable. |
| Revocation | Explicit source relationship revoke prevents even historical-revision reads. Policy catalog revoke blocks current reads without requiring a new Case generation; old ALLOW is not reused. |
| Cross-Case | Same policy original is reused by independently authorized Cases; bindings and Case source identity remain distinct. A source not attached to the second Case is unavailable there. Ordinary content remains Case-bound. |
| Operational distinction | SQLite Resource/configuration identity remains operational; the captured revision references the exact ResourceObservation result and digest, not the database itself or inferred knowledge. |
| Restart/rebuild | Reopened product and engine stores reproduce source state; Case verification/rebuild equals Transition replay. Derived policy/graph/memory/context loss and rebuild do not change canonical source relations. |
| Read purity | Inventory/read/verification append no Transition. Current permission is revalidated; reads do not invent Operation provenance or authority. |

The interruption proof is a bounded stop, separate process resume and failed-read
recovery. It is not a claim of exhaustive SIGKILL/power-loss testing at every
instruction. Canonical request/progress identities and immutable per-item backing
provide the recovery contract; completed item admissions can be reused when a
source-level attempt was incomplete.

## Retained executable evidence

All runs use working directory
`/home/mothx/computer-science/projects/YAI/yai` and the baseline plus this wave's
worktree. Independent runs are not spliced into a single Case proof. The product
checkpoint (Run 2) ran while Run 1 was finishing; Run 3 followed Run 1. Ordered
actions inside each Case are retained separately. Subsequent cleanup removed only
incidental formatting changes to pre-existing historical/Recall code; no runtime
behavior was altered after the publication union.

### Run 1 — deterministic publication union, source product and Golden local

Run ID: `yai-bootstrap-publication-02`; start `2026-09-11T14:40:21Z`, finish
`2026-09-11T14:48:28Z`; actual exit **0**.

```sh
PATH="/tmp/yai-governance-qa-venv.Jf2vcj/bin:$PATH" make check characterization test-golden-local
```

Pre-state: implementation/tests frozen; disposable test homes/resources; existing
QA virtual environment provides `pyte`; no external-provider lane requested.
Runtime/source/test files were not changed during this run. Closure documentation
and O08 maturity were reconciled afterwards and checked separately.
[Unedited selected stdout lines](publication-excerpt.log) retain topology metadata,
the source characterization/result and both Golden results. The roadmap counts
in that excerpt are correctly **pre-promotion** counts, not final maturity.
Full local stdout/stderr was `/tmp/yai-bootstrap-publication-02.log`.

Its source product child is run `yai-source-bootstrap-mdnv6mun`, fresh
`/tmp/yai-source-bootstrap-mdnv6mun/home`, `provider_mode=no_provider`.
[Unedited ordered command/result excerpt](source-run-excerpt.jsonl) retains its
exact baseline, environment, actual CLI argv, exits and first twelve actions;
individual stdout truncation, if any, is explicitly marked by the runner.
The fixture performs the remaining assertions and emits:

```text
source_bootstrap_product=PASS policy_first=true dual_role_one_backing=true denied_payloads=0 resumed=true revisions=true current_revocation=true models=0
```

Golden free: `yai-golden-free-b178ke5c`, final verdict order 164, PASS,
`loopback_fixture`, 15 model dispatches, external request attempted false.
Golden Workflow: `yai-golden-free-hk7ksrwo`, final verdict order 148, PASS,
`loopback_fixture`, 18 model dispatches, external request attempted false.
Both retain exact Invocation/W identities and recompiled-W equality after
rebuild. They are **local** model-fixture qualification, not external or human
acceptance. The ordinary deterministic union also retains semantic/compiler,
locality, disclosure, delta/full, historical/Recall, replay and recovery gates.

### Run 2 — cumulative product checkpoint, no model

Run ID: `yai-source-operator-proof.3L2uvj`; fresh independent
`YAI_HOME=/tmp/yai-source-operator-proof.3L2uvj/home`, `NO_COLOR=1`.
Actual shell sequence exit **0**. [Full unedited product transcript](product-checkpoint.log)
retains execution order (`set -x`), each real `./yai` action and observed output,
using `tests/cases/06-source-bootstrap/perimeter.json` and its real policy/files.
This is separate from the dynamic company fixture, not an invented transcript.

The ordered actions are init → create → role/link → declare → inventory →
acquire bootstrap → inventory → explicit publish → acquire limit 1 → resume →
read → verify → revoke → negative read. Nonnegative commands ran under `set -e`;
the final read was explicitly required to fail. Exact observed checkpoint:

```text
"generation": 17,
"materialization": "equivalent_to_replay",
"mutation": "none",
"schema": "yai.case_state.v16"
```

The raw transcript records source IDs, role/readiness/coverage and the later
generation 18 revocation. It ends with `yai: source_not_available` and
`source_operator_product_checkpoint=PASS human_acceptance=NOT_RUN`.
It is automated product evidence; Francesco's Human Golden remains pending.

### Run 3 — adversarial transactional contracts

Run ID: `yai-bootstrap-contract-final`; executed after Run 1 with unchanged
implementation/tests; fresh per-test stores. Exact command, actual exit **0**:

```sh
CARGO_TARGET_DIR=target cargo test --manifest-path engine/Cargo.toml source_bootstrap_tests --offline -- --nocapture
```

[Unedited stdout/stderr](contract.log) retains both exact test names, assertions
and 2 passed / 0 failed. These are independent engine contracts, not product
transcripts or provider substitutes. They prove outsider/unlinked scope refusal,
sibling/re-entry refusal, source-provenance nonfabrication, forged coverage and
stale-progress refusal, policy revocation without Case generation, schema reader
negatives, restart/replay and derived cache amnesia.

Development failures were not counted as PASS: an early bootstrap used the
policy-catalog byte bound rather than the existing smaller confined-resource
bound, and early resume reused a non-current incomplete Operation. Both were
repaired under existing contracts before Run 1. An earlier full gate stopped on
missing `pyte`; it was not a publication pass. The existing qualified environment
was used for Run 1; no test was skipped to compensate.

### Run 4 — post-cleanup closure checks

Run ID: `yai-bootstrap-cleanup-validation`, after incidental formatting cleanup;
same working directory, unchanged executable semantics, fresh per-test stores.
The following ordered chain exited **0**, stopping on any failed step:

```sh
CARGO_TARGET_DIR=target cargo test --manifest-path engine/Cargo.toml source_bootstrap_tests --offline -- --nocapture
CARGO_TARGET_DIR=target cargo build --manifest-path cmd/yai/Cargo.toml --offline
make check-roadmap check-docs check-layout test-roadmap test-topology
python3 tools/validation/topology.py audit
```

[Unedited closure output](closure-validation.log) retains the 2 source contracts,
build, final 32/26/11/4 roadmap count, documentation/layout, 8 roadmap tests and
10 topology tests. Static audit reports 516 classified entries, 95 assertion
files, 408 Rust tests and no duplicated combined Make leaves. These are catalog
counts, not an aggregate substitute for the independent product/Golden proof.
`git diff --check` and staged `git diff --cached --check` pass. A trailing blank
line was omitted from the retained contract excerpt to satisfy the whitespace
guard; its observed result lines are unchanged.

## Cost characterization, not an SLA

All timings below are observed wall-clock CLI time from the **same Run 1 source
child**, including process/store opening and returned inventory; not pure parser
or I/O microbenchmarks. They do not establish Case-age-independent CPU cost.

| Perimeter / phase | Observed time | Material / coverage |
|---|---:|---|
| Small: declare 6 | 22.221 ms | 6 explicit declarations, no source payload acquired |
| Small: capture bootstrap policy | 41.069 ms | 1 acquired candidate, 5 pending; not published |
| Small: explicit publish/bind | 59.872 ms | READY; ordinary material still pending |
| Small: acquire first ordinary source | 150.777 ms | `--limit 1`; 2 acquired total |
| Small: new-process resume | 739.380 ms | 4 acquired, 1 denied, 1 inaccessible |
| Larger: add 32, total 40 | 477.536 ms | Prior scenario retained; 32 new exact files |
| Larger: inventory | 275.429 ms | Explicit source count, no payload acquisition |
| Larger: first batch | 7,671.700 ms | `--limit 8` |
| Larger: resume remainder | 41,290.295 ms | 32 new items / 1,558 bytes acquired across both batches; final 38 acquired, 1 denied, 1 revoked |

Current source visibility/admission and inventory repeatedly inspect canonical
history. Host cost grows materially; this measurement identifies future pressure
without adding an acquisition index/store or weakening current revalidation.
The 128-source contract is a supported bound, not a claim about organization size.

## Schema, ownership, product and remaining scope

- Transition **v18 → v19**: two typed source declaration/progress payloads.
- CaseState **v15 → v16**: compact replay-derived source relationships.
  Previous readers/metadata upgrade paths retain old histories; old schemas cannot
  encode new source fields. No historical Transition is rewritten.
- New subordinate declaration/progress/perimeter representations are v1.
  Policy IR/artifact, EffectivePolicy, Resource request/observation, content,
  Projection, ContextFrame, S/W and Recall schemas/semantics are unchanged.
- No independent semantic/operational owner added or moved. Existing resource
  access gains a Case-native acquisition lifecycle; history is canonical.
- **LMDB delta 0; 37 of 40 named databases.** No BootstrapJobStore, SourceStore,
  knowledge graph, content-ingestion registry or new cache.
- Product: `case sources declare|inventory|acquire|resume|publish|read|revoke`,
  explicit source/limit/refresh/revision controls and structured `--json`.
  See [executable contract](../../../docs/case-source-bootstrap.md).
- [ZERO-TO-CURRENT](../../../docs/zero-to-current.md) gains a separate optional
  no-model checkpoint. It does not reset Golden resources or the canary.
  README, REPLAI and YVEX are unchanged.

O08 advances **OPEN → PARTIAL** only. No new maturity rows: totals are
32 ESTABLISHED / 26 PARTIAL / 11 OPEN / 4 LATER, 73 overall. K/A/W/X/Q gain
bounded supporting evidence without additional maturity promotions. M07 stays
OPEN; M06/S12 keep their existing PARTIAL scopes; Recall/W are not extended.

Remaining gaps: generic inventory/connectors, automatic synchronization,
arbitrary policy interpretation/OCR, ordinary cross-Case content reuse, knowledge
derivation and source-aware Recall/W. Reviewed acquisition pauses fail closed;
general review-completion orchestration through source porcelain is not qualified.
Initial policy setup cannot be reused as a policy-refresh bypass. Ordinary
policy-role capture does not automatically publish a candidate. Directory captures
are bounded individual-file observations, not atomic filesystem snapshots;
existing non-empty content and byte/item/depth bounds apply. Source revoke affects
the frontier relationship, not global byte erasure or future Recall integration.

## External posture and INTERLOCK CHECK

Public YVEX roadmap inspected read-only; retained downloaded public-roadmap digest
`sha256:d4adb1df116430cc2a1430c43340d61a1e2f4d0deb0d8c15fe5bba5e7810d2e7`.
Its refoundation/runtime/performance progression remains producer-owned; no YVEX
source internals, endpoint generation or service administration was used.
Current producer control belongs to the
[public YVEX roadmap](https://github.com/yailabs/yvex/blob/models2/ROADMAP.md).

YVEX EXTERNAL FINDINGS: **no new findings**; no live qualification was executed
because this is deterministic YAI source acquisition, with no changed provider or
computational-state contract. External Golden stays **FAIL / incomplete** at its
retained consumer evidence; governance live model **NOT QUALIFIED**;
HUMAN_GOLDEN_CASE **PENDING_OPERATOR**; continuity canary **NOT_RUN**.
Source bootstrap uses zero model requests and zero model fallbacks.

INTERLOCK CHECK: existing Case/Resource/policy/content/admission contracts compose
inside YAI. The demonstrated seam requires no producer migration, external state
consumer or new public cross-owner dependency. No new Interlock selected;
**I07 UNSELECTED**. BOUNDARY is not used for this internal refactor. The next
implementation boundary remains UNSELECTED; M07 is not started.

Concurrent work in `docs/provider-governance.md`,
`tests/characterization/case-resource-access/test_reference_free.py` and
`refoundation/validation/external-golden-closure/` is preserved and excluded from
this commit. Their existence is not this wave's worktree residue or external PASS.
