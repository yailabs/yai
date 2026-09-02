# Foundation Hardening 17 — Adaptive Workflow Semantic Closure

State: semantic implementation and qualification published; evidence binding
pending.

- Baseline: `e58b08996649a30ebe3446afc4ebbfe4ef2aadfd` on `master`, equal to
  `origin/master` and the published remote before H17.
- Semantic commit: `2df9bb5c2bbd7efc53ed54527111522af329cf93`, published as
  `harden: close adaptive workflow composition semantics` and verified equal
  across `HEAD`, `origin/master` and `refs/heads/master` before this evidence
  binding update.
- Historical dirty work: all 13 entries remain preserved and excluded. Their
  tracked checksum is
  `3fdb219654405e6fd40b5c0d1b02b94c04fadef5aa57a139aa5fb8fd6db7777e`.
- External provider qualification: `blocked_external_dependency`; both
  required provider variables are absent.

## Direct archaeology and owner recheck

Fresh source/history inspection covered `yai-dev` commits
`94ba627091afbf1100ab386ac1de3d4fb1d2502c`,
`001752f52545fe84a30017ec735794a8f04d189c`,
`840aee9464211e7af6d6811de32320faea14806e` and
`cffb318b980456f2671a297e14a6b05f5ac68320`. The exact inspected sources
included `core/orchestration/flow/model/flow.c`,
`core/orchestration/handoff/runtime_handoff.c`,
`include/orchestration/planning.h`, and the later
`src/orchestrator/{workflow,handoff}` implementations and consumers.

Legacy retained useful explicit record, dependency/gate, current-work and
request/result/reconciliation intent. It had no mechanism stronger than H17
for content-bound amendment lineage, serializable cross-Case cancellation,
concurrent cycle closure, qualified Subflow recovery, or fail-closed topology
digest replay. Mutable flow-controller directories, overwrite/current JSON,
actor/provider routing and the global Orchestrator remain rejected. H17 goes
beyond the executable legacy corpus.

Hardening found no independent lifecycle missing from Case Transitions.
Amendment remains Case-canonical; EffectiveTopology remains derived;
Subflow remains same-Case composition; Handoff remains a split Case protocol;
MultiCaseProcess remains a derived graph. Semantic-owner delta is zero.

## Defects closed

Three real failures were preserved before correction:

1. a syntactically valid nonexistent target evidence ref was accepted by a
   Handoff result;
2. a target could record `succeeded` after its Case cancellation;
3. deep executable Subflow nodes were looked up in the root Definition by
   local node ID, failing with `workflow_node_not_found`.

The store now resolves every Handoff evidence ref against one exact target
Transition/fact, makes terminal target lifecycle and result mutually ordered
inside the LMDB transaction, and carries qualified effective node identity
through WorkItem materialization and deterministic proposal creation.
Acceptance, decline, result, reconciliation and amendment payloads revalidate
their existing content-bound IDs during replay. No schema meaning changed.

## Amendment and EffectiveTopology

The admitted 32-revision chain was committed and every prefix rebuilt against
its exact parent, revision, previous digest, resulting digest, patch identity,
binding and operations. Missing, reordered and field-corrupt middle links fail
at the first inconsistency. A 32-process same-revision race yields one winner,
31 stale refusals and one generation advancement. Exact semantic proposal
retries are idempotent; a previously adopted patch gets the explicit
`workflow_plan_patch_already_adopted` posture.

Replay derives topology only from exact immutable Definitions, binding and the
ordered amendment chain. Each committed resulting digest is an upgrade guard:
if changed materializer code derives different bytes, load fails closed before
unresolved work. This digest chain is sufficient; no separate materializer
version or cache owner is introduced. Frozen facts remain canonical, while
future unresolved posture may be computed only after Definition and topology
integrity succeeds.

Patch limits are exact at 256 KiB, 32 operations and 16 added nodes. The
declared 64-added-edge cap is dominated by the 32-operation cap in v1 and
therefore cannot be reached by one candidate; this is recorded rather than
misrepresented as an executed boundary. Unknown `plan_patch.v99` and path
separator ambiguity fail closed.

## Model patch origin

Provider-result proposals now require one exact ProviderInvocationStarted and
ProviderResult pair with identical semantic lineage, causal links to exactly
one qualified Workflow execution, an invocation causal link, and the exact
`plan_patch` output contract. Another Case/Tenant/node, ordinary text output,
ambiguous history, or a completed prior execution cannot originate a patch.
Identical bytes from distinct legitimate executions retain distinct causal
origin. A stale model patch remains history and cannot rebase or self-adopt.

## Subflow recovery

Root → S1 → S2 → S3 → S4 replay uses the injective qualified identity
`root/s1/s2/s3/s4/node`. Exact child Definition ID and digest are required at
every level; corrupting only C fails the root closed, while restoring the exact
record restores the prior resolution without repair mutation. Deep ModelWork
recovery retained one execution, one invocation and one result. Deep
DeterministicWork retained one proposal and one Operation. Duplicate child
instances remain isolated by their qualified path. Definition node IDs cannot
contain the path separator.

## Handoff closure

Result refs are a closed target-local canonical-fact projection. Wrong
handoff/acceptance/source/target/Tenant, missing acceptance, decline, terminal
target lifecycle, nonexistent refs and refs from another Case/Tenant fail.
`succeeded` is explicitly a target report about the bounded Handoff contract;
empty evidence is legal when the offer did not require evidence. It never
becomes a source EffectReceipt, Decision, Grant or physical-success fact.

Thirty-two result writers produced one canonical result: identical repeats
observed the same ID without generation churn and conflicting outcomes were
refused. Thirty-two reconcilers observed one source reconciliation and one
source generation advancement. Result/reconcile, cancel/accept,
cancel/result, cancel/reconcile and close/settlement races serialize through
the shared LMDB authority. Source reconciliation may settle audit truth after
cancellation but cannot resume progression. Close blocks an accepted or
resulted handoff until source reconciliation.

The active wait graph is reconstructed from CaseState derived from canonical
Transitions. Concurrent A→B/B→A admits one edge; concurrent A→B/B→C/C→A admits
two. Decline, reconciliation and accepted target termination remove the wait
edge from the active graph. A 64-Case, 63-edge graph rebuilt graph relations
twice without duplicates and refused the closing cycle. Graph records are not
authority and no Process owner is needed.

## Compatibility, runtime and footprint

Static WorkflowDefinition/Binding v1 Cases and v2 Cases retain existing
behavior. Current schemas remain WorkflowDefinition v2, Binding v2,
PlanPatch/Amendment/EffectiveTopology/Handoff v1, WorkflowResolution v2, and
Transition/CaseState v11. Unknown future adaptive schema input fails closed.

The W16 registry/parser/output boundary is unchanged; no parser bypass was
added and `cmd/yai/src/main.rs` remains 12 lines. Waiting Handoffs and Subflow
containers hold no worker or ResourceFence. New post-amendment/post-handoff
operations still traverse DecisionBasis, Decision, Review where required,
Grant, ResourceFence and Carrier.

LMDB remains 35 named databases with `set_max_dbs(40)`. There is no Amendment,
Topology, Handoff, Process or WorkflowRun database; no Agent, manager, worker
pool or scheduler was added.

## Foundation Recovery reclassification

- PlanPatch: `refounded_proven + adversarially-qualified`.
- Amendment: `Case-canonical + lineage/concurrency-qualified`.
- EffectiveTopology: `derived_no_owner + replay/upgrade-qualified`.
- Subflow: `refounded_proven + nested-recovery-qualified`.
- Handoff: `refounded_proven + adversarially-qualified`.
- MultiCaseProcess: `derived_no_owner + graph/replay-qualified`.

## YVEX EXTERNAL FINDINGS

`yvex_external_qualification_state=blocked_external_dependency` because
`YAI_EXTERNAL_PROVIDER_BASE_URL` and `YAI_EXTERNAL_PROVIDER_MODEL` are both
missing. No live request was made, no YVEX source or administration surface was
used, and no external finding is fabricated. Classification:
`DEPLOYMENT_LIMITATION`. Generic remote-delivery indeterminacy remains an
expected provider-contract limitation and was not changed by H17.

## Remaining Wave 18 boundary

Wave 18 alone remains provider qualification ownership, capability/trust/
health posture, selection, justified failover and optional YVEX-native
extension. H17 adds none of these.
