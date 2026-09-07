# GOLDEN.CASE.LIFECYCLE.0

Authority: bounded implementation and qualification evidence, not a new semantic
owner. Baseline: `4d5a69c32622d3f2e37d399f8305001d3a4f27cc`, existing
`master` and physical worktree. HEAD/origin/master/remote master were equal
before mutation. Intended commit:
`feat: converge governed Case workbench and Golden lifecycle`.
Pre-publication status: implementation, complete local publication union,
Golden free/Workflow qualification and documentation/layout checks PASS.
Explicit wave whitelist and owner-scoped staged diff reviewed; publication and
remote equality verification follow. The containing commit SHA is deliberately
not recorded here.

I01–I06 remain COMPLETE; I07 UNSELECTED. REPLAI's qualified pin and R5 removal
are unchanged. No YVEX source, private protocol, deployment administration,
post-I10 implementation, H20/W21/W22, Agent owner or Studio is introduced.

## Architectural conclusion and owner reuse

The previous conversation host executed cognition but could not expose attached
operational resources as governed, model-requestable capabilities. The Product
entry is now `./yai case workbench CASE --participant HUMAN --executor MODEL`.
A human remains inside one Case. REPLAI owns terminal editing; the frontend calls
typed application actions, never a subprocess invocation of YAI.

The same finite application algorithm serves free work and Workflow CaseWork:

```text
atomic human Turn + exact delegated execution intent
  → current derived Case capability view
  → I05 arbitration / exact I02 plan / I03 native realization
  → canonical ProviderResult containing candidate function request
  → bounded schema normalization / current Case + Policy admission
  → Observation, immutable content admission, or Decision/Grant/fenced effect
  → canonical result feedback / next bounded cognition
```

Model prose is never a hidden operation protocol. Attachment is not permission.
The native function contract is mechanically probed separately from semantic
suitability. Operator onboarding requires explicit trust approval and records
OperatorAttested suitability, not invented mechanical semantic qualification.
The author and executor remain separate Participants; delegation transfers no
Principal link or review authority.

| Concern | Existing owner / new bounded contract |
|---|---|
| Conversation and work intent | Case Transition history; immutable Turn and v3 finite work intent |
| Resource envelopes / current content relations | ResourceAttachment / CaseState, bounded current materialization |
| Local addresses / executable definitions | Existing local_resource_bindings; digest-bound to canonical attachment |
| Policy, review and effect authority | Existing PolicyArtifact/EffectivePolicy/Decision/Review/Grant owners |
| Effect serialization and recovery | Existing resource-control fencing and canonical effect lineage |
| Immutable admitted file bytes | ConversationContentStore; object-first verified publication |
| Model selection/invocation/result | Existing provider governance and I02–I06 exact semantic seams |
| Workflow | Existing WorkflowDefinition/binding/progression/PlanPatch; no private executor universe |
| Capability catalog / graph / memory / index | Derived, rebuildable working/access state; not authority |

Semantic owners **+0**, operational owners **+0**, LMDB databases **+0**:
**37/40**. New adapters/carriers and typed relations do not introduce independently
owned databases, registries, daemons or history. Root README is unchanged.

## Reference world and authority

`tests/cases/04-golden/` retains the small defective retry implementation,
deterministic test, policy source, WorkflowDefinition and resource infrastructure.
The existing identifier convention yields `case:golden:free`,
`case:golden:workflow`, `case:golden:isolation`. Their workspaces, database
images and bindings are separate. The local authenticated human is both operator
and eligible reviewer; the model is a separate Participant without that Principal
link. This preserves the existing one-Principal/one-Participant-per-Case invariant.

Independent evidence: source shows the defect; database identifies the stable
release channel; ordinary HTTP reports the live canary status; MCP provides
base/caps and risk validation; the immutable issue states the requested behavior;
a discovered migration note supplies the attempt-origin constraint. Neither the
issue nor Workflow task supplies the complete test answer. The initial oracle
fails; the reviewed repair passes. The deterministic model fixture is an
automated protocol peer, not a claim about model intelligence.

| Resource | Admitted behavior | Boundary and demonstrated result |
|---|---|---|
| Filesystem workspace | Confined read/search and existing reviewed write | Descriptor-relative no-symlink access; protected path DENY; exact reviewed source replacement; real test consequence |
| Discovery / information | Bounded candidate enumeration, explicit admission, owned content read | Candidate digest/drift checked; canonical provenance; subsequent external source drift does not change owned issue |
| Process runner | Exact named executable/argv/cwd/env, timeout/output limits | Governed effect; real exit 7, timeout and final exit 0; no shell strings or ambient network |
| SQLite database | Fixed named bounded read-only query | Real SQLite engine over a quiescent bounded rollback image; columns/rows/source identity; mutation DENY and corruption refusal |
| Ordinary HTTP service | Exact named GET under bound endpoint/path/IP scope | Real HTTP peer, no redirect/host widening; unavailable peer refuses without a successful Observation |
| MCP server | Catalog, exact resource read and independently governed named tool call | Real Streamable HTTP, catalog/schema drift refusal, unknown tool DENY, ambiguous delivery retained |
| Compute target | Public OpenAI-compatible text/functions and JSON-object shapes | Real loopback transport; semantic evidence and trust separate; exact plan/lane/target preserved |

The policy deck uses the real source→compile/PolicyIR→validate→publish→bind→READY
supply chain. Direct ALLOW covers scoped reads and deliberately admitted bounded
process/risk operations; database mutation and non-admitted/protected operations
DENY; code change REQUIRE_REVIEW. The model cannot self-approve. Review itself
does not execute the effect. DENY creates neither Grant/PREPARE nor fake receipt.
Revocation, expiry and current generation remain independently revalidated by
the existing authority chain, not a connector-specific policy engine.

## Scope, failure and restart contracts

Resource reads revalidate current authority immediately before access and again
before canonical observation publication. A failed read does not manufacture a
successful Observation. External material is provenance-bearing, not semantic
truth. Immutable admission names the exact candidate, source Observation,
Operation, Decision, Principal, disclosure and verified content object. Complete
unreferenced objects may survive a failed commit; partial bytes cannot become a
canonical relation. The current relation is bounded to 128 admitted objects.

Process and MCP effects consume a finite Grant in atomic fenced PREPARE before
external work. Terminal observations/receipts finalize and release the fence;
possible delivery retains INDETERMINATE and does not redispatch. An interrupted
prepared process/tool call is not guessed safe. Filesystem reconciliation retains
its existing exact pre/post proof. General remote reconciliation without
authoritative external evidence is intentionally absent.

The initial process profile is unprivileged Linux x86_64, root-owned
descriptor-pinned ELF, Landlock ABI ≥6 and seccomp allowlist. Workspace and OS
runtime access are read-only; network, descendants, writable files and unsupported
syscalls are forbidden. Time/output/CPU/address-space bounds are enforced.
`prlimit64` permits only reading the current process's limits, not changing them
or inspecting another process; admitted `ioctl` requests are narrowly enumerated.
Unsupported confinement refuses. This is not a universal build runner.
The contract follows the kernel's
[Landlock UAPI](https://docs.kernel.org/userspace-api/landlock.html) and
[seccomp filter contract](https://docs.kernel.org/userspace-api/seccomp_filter.html).

Case work admits 1–24 invocations, bounded operations/effects and input estimate.
Budgets and per-step identities derive from immutable intent, so retry/restart
cannot reset them. Canonical completed Result→Operation→Observation/Receipt
feedback is reconstructed before new cognition. A prior ambiguous invocation
blocks cross-target redispatch. New steps may freshly arbitrate; old completed
steps keep their exact provenance. Cancellation gates later work and does not
pretend to abort a blocking transport.

WorkflowDefinition v3 explicitly distinguishes finite CaseWork completion from
an intermediate capability request. One exact Workflow execution admits one
Turn/intent, with canonical invocation-budget enforcement. Existing runtime-queued
executions cannot be taken over by the host. PlanPatch remains non-authoritative:
generate→inspect→validate→explicit adopt/refuse. Restart reconstructs progression;
free and Workflow modes call the same capability/effect seams.

## MCP and public provider conformance

Official MCP **2026-07-28**, **stateless Streamable HTTP** is the admitted profile.
This version uses `server/discover` and per-request version metadata, not a
fabricated legacy initialization/session handshake.
Implemented: server/version/capability discovery, `tools/list`,
`resources/list`, exact bounded textual `resources/read`, governed
`tools/call`, JSON and bounded SSE result decoding, prescribed method/name/
argument headers, local JSON Schema 2020-12 validation, opaque cursor pagination
(max eight pages/128 entries), request-local cache posture and revalidation before
use. No shared catalog cache is trusted to preserve tool semantics; drift refuses.
Unsupported schemas/external references, output kinds, protocols and unsafe
retry requests fail closed. No Apps, prompts, sampling, Tasks or session owner.

Normative sources inspected before implementation:
[transport](https://modelcontextprotocol.io/specification/2026-07-28/basic/transports/streamable-http),
[discovery](https://modelcontextprotocol.io/specification/2026-07-28/server/discover),
[tools](https://modelcontextprotocol.io/specification/2026-07-28/server/tools),
[resources](https://modelcontextprotocol.io/specification/2026-07-28/server/resources),
[pagination](https://modelcontextprotocol.io/specification/2026-07-28/server/utilities/pagination),
[caching](https://modelcontextprotocol.io/specification/2026-07-28/server/utilities/caching).

Native model calls follow the
[official Chat Completions contract](https://developers.openai.com/api/reference/resources/chat/subresources/completions/methods/create)
and [function-calling guide](https://developers.openai.com/api/docs/guides/function-calling).
OpenAI Docs skill guidance informed wire qualification and correlated function
feedback only; it did not introduce model/provider authority. The admitted
profile is one bounded offered function call or final text, with exact model and
schema validation. JSON-looking prose remains text. A separate actual JSON-object
probe supports Workflow PlanPatch. These are loopback contract proofs, not live
OpenAI or YVEX execution.

## Exact version and compatibility accounting

| Contract | Before → after / reason |
|---|---|
| Transition | v17 → v18: resource Observation, admission and effect payloads; exact work intent meaning |
| CaseState | v14 → v15: access envelopes, bounded admitted content, existing effect posture |
| Projection / ContextFrame | v7 → v8: scoped resource observations/content references and typed capability output contract |
| ProviderQualification | v4 → v5: separately evidenced native function and JSON-object realization shapes |
| WorkflowDefinition | v2 → v3: finite CaseWork output/completion contract |
| CognitiveCompositionRequest | v1 retained; v2 exact delegated executor; v3 bounded work and optional exact Workflow execution |
| Operation / ExecutionGrant | existing v2/v3 retained; resource variants v3/v4 |
| ResourceFence / ResourceControl event and state | existing v1/v2 retained; exact network fence v2 and network event/state v3 |
| New bounded values | local_resource_access_binding, resource_access, resource_request, resource_observation, case_content_admission, prepared_resource_effect, resource_effect_receipt, case_capability_view/output, resource_definition: v1 |

ConversationTurn/ContentStore remain v1; RetrievalSet v3 and RenderedInput v7 are
unchanged. Historical Transition v1–v17 and old materializations remain readable.
A dedicated published-I06 marker reopen and Workflow v2 marker migration test
verify upgrade without rewriting immutable definitions or canonical history.
Unknown/version-incompatible new fields fail closed.

## Qualification and retained executable evidence

Run files retain their own run ID, command/order, cwd, disposable home/pre-state,
actual exit and bounded unedited output or original PTY hex. Development runs,
failed attempts and final runs are not spliced into one causal transcript.
Tool-captured excerpts may carry explicit truncation markers; per-product JSONL
contains the separately captured actual actions/output. Peer termination -15 in
cleanup is not a provider qualification verdict.

| Proof class / provider mode | Evidence and authorized claim |
|---|---|
| Unit/component / no_provider | Release gate: envelope/schema, local SQLite, confined process, replay/atomicity and authority tests |
| Contract / loopback_fixture | Native function and typed provider tests; real adapter/HTTP dispatch, not model intelligence |
| Product / no_provider | `resource-effects-final.jsonl`: real filesystem/SQLite/HTTP/MCP/process, 50 CLI commands, generation 83, no fake Observation after stopped peers/corrupt DB |
| Product / loopback_fixture | `local-union-08-free.jsonl` and `local-union-08-workflow.jsonl`: actual `./yai` + REPLAI PTY, independent resources, review, initial/final oracle, exact retries and replay |
| Recovery / no_provider + loopback_fixture | Store/resource tests, capability host crash after Result/Observation, I01–I06 and prior review/effect/runtime recovery in release |
| Endurance / declared local modes | Existing bounded publication endurance remains reachable; no external model dependency |
| External / external_yvex | NOT RUN / DEPLOYMENT_LIMITATION; missing endpoint/model fails with exit 3, no fallback |
| Manual / external_yvex | PENDING_OPERATOR; cumulative runbook structurally audited, never a human PASS |

The complete publication command is `env TMPDIR=/tmp
PATH=/tmp/yai-golden-validation.WJzn38/venv/bin:$PATH make test-release`;
the separate cumulative product command is the same environment with
`make test-golden-local`. Exact final exits are recorded in the run summaries.
Characterization is included in the publication union rather than ritual
reexecution of identical leaves. Golden's explicit cadence is separate from
`check`; the classification audit proves old local coverage remains reachable,
new Golden leaves are independently reachable and neither lane admits external
providers. The external Golden target runs free and Workflow with operator inputs,
never the deterministic model/encoder fixture.

Final publication run `golden-release-05` (`release-05.json`) exits **0**.
Final Golden union (`local-union-08.json`) exits **0**. Its independent canonical
checks, not merely model text, establish:

| Causal run | Mode / dispatches | Canonical assertion |
|---|---|---|
| `yai-golden-free-97sg0q9n` | free / 15 loopback model dispatches | nine resource-observation kinds, ALLOW/DENY/REQUIRE_REVIEW, no Grant for DENY, exact owned issue/note admissions, process exits **[1, 0]**, reviewed filesystem FINALIZE |
| `yai-golden-free-r07pa0c7` | Workflow / 18 loopback model dispatches | same canonical lifecycle plus actual PlanPatch validation/explicit adoption and restart-safe Workflow progression |

Both runs separately verify the same-Case W19 index rebuild and unchanged
canonical history. The topology audit reports coverage parity (71 old smoke
leaves remain reachable; 83 current local Make leaves; zero duplicate Make
leaves in the combined union), with 465 classified entries/367 Rust tests.
These counts supplement, not replace, the proof/provider classification.
`process-final.json` independently proves actual confinement, nonzero exit,
timeout and output bounds without a provider. `resource-effects-final.json`
retains the real failed-peer/database and indeterminate-effect product run.

The full local reference additionally proves legal Handoff without authority
transfer, cross-Tenant refusal, empty/differently disclosed isolation resources,
same provider not merging Cases, and exact history invariance after CaseState,
operational-memory/graph and W19 embedding-index drop/rebuild. A separately
qualified local encoder fixture supplies the index proof. Episodes/assertions
use existing deterministic extractors and remain derived; new observations do
not automatically become semantic assertions. New resource families do not gain
unimplemented analytics extractors.

Failures fixed during qualification: old Workflow v2 metadata admission,
a stale expected refusal diagnostic, PTY-driver backpressure while printing
inspection output, and a fixture feedback-envelope lookup. Stronger independent
canonical assertions also detected that the original reference runner envelope
truncated the initial failing test before its exit status was available. The
reference now explicitly admits a bounded 64 KiB result envelope; both final
runs prove real initial exit 1 and final exit 0, not a timeout/truncated result
substituted for the software oracle. Separately, final process review narrowed
prlimit/ioctl access and qualified the resulting real interpreter behavior.
Failed release/union attempts retain their own distinct raw files. No security
guard was weakened to make a fixture pass. CLI strict Clippy passes; engine
Clippy retains the repository's pre-existing
too-many-arguments/should-implement-trait allowances.

## Archaeology verdict

Read-only `yai-dev` at `5c1c7b9d099eea9f2947146cd821d6501c4a6ddf`,
history `e659f15b5`, `2a4018147`, `e9ad7f498`, `cffb318b9`:
attachment permission markers and file-touch/upsert metadata do not implement
current authority or immutable admission. Historical database observers establish
stat/reachability, not SQL execution; MCP declarations do not prove a client.
Workflow transition/checkpoint nucleus stubs are weaker than current Workflow.
Historical ChatStore/runtime ownership is rejected. Recovered properties are
attachment≠permission, detach≠payload deletion, exact source identity and bounded
failure; current ResourceAttachment/Policy/effect/ContentStore own them.

## Cumulative operator acceptance and remaining boundary

[ZERO-TO-CURRENT](../../../docs/zero-to-current.md) is the single evolving,
human-executable procedure: infrastructure→bootstrap→free workbench→review/effect/
tests→Workflow/PlanPatch→restart→Handoff/isolation→rebuild→model replacement.
It contains actual `./yai` and interactive actions, not a Python model driver.
Reference Python serves only the external problem resources. Bootstrap identity
and optional encoder administration remain explicit administration.

YVEX EXTERNAL FINDINGS: **DEPLOYMENT_LIMITATION**. No exact operator-supplied
public endpoint/model was available; no live request or source/admin inspection
was performed. The demonstrated generic requirement is native function requests,
correlated tool feedback and separately qualified JSON-object output. Whether the
operator's DeepSeek deployment satisfies it is unresolved, not a claimed YVEX
defect. A text-only public surface cannot close this workflow; private protocols
are not substitutes.

DeepSeek: external acceptance pending. Qwen replacement uses the same explicit
`/connect ... --replace` qualification/binding path without recreating the Case;
no Qwen run is claimed. Provider sessions/KV/residency remain external.
General writable build runners, live/WAL database transactions, state-changing
HTTP, broader MCP features and remote reconciliation without terminal evidence
are outside this bounded reference profile. No internal reference adapter is
silently promoted to universal production compatibility.

CONTINUITY CANARY = NOT_RUN, operator-owned procedure recorded; no private
canary data touched or committed.
HUMAN_GOLDEN_CASE = PENDING_OPERATOR.
ZERO-TO-CURRENT = UPDATED.
No future Interlock or post-I10 wave is started.
