# Guided Case CLI — Product/backend closure

Authority: bounded implementation and validation evidence. Baseline
`f6c7c8bb0e303fae741fbd795f3604a18c5762a5`; existing master/worktree,
HEAD/origin/master/remote master reconciled equal before mutation.
Intended commit: `feat: simplify Case CLI with guided authoritative setup`.
Pre-publication status: implementation, deterministic publication union,
characterization, Golden local, documentation/layout/topology and formatting
PASS. Explicit whitelist and owner-scoped staged diff reviewed; publication and
remote-equality verification follow.
The containing commit SHA belongs in the post-publication handoff, not here.

## Implemented boundary

The old primary journey exposed seven Participant bootstrap commands and repeated
Case/operator/executor flags. Short `./yai init` and `./yai open [NAME]` now
compose the existing owners. Missing values use the qualified REPLAI editor;
explicit automation flags and old `case workbench`/`prompt` stay supported.
`case open` is an alias of the single compiled `yai.case.open` operation.
No current-Case file, global selection, child shell or subprocess CLI dispatch
exists. Case/Participant/Resource short names are exact prefix spelling, not
fuzzy matching or a new alias registry.

The frontend shows identities/roles before consent. The existing LMDB owner
commits at most sixteen allowed Participant transitions atomically under current
Tenant-owner authority. Mixed Case/payload, stale generation and invalid identity
cannot publish half a setup. Existing partial Cases are preserved; reopening a
ready Case makes no canonical change. Creation is a separate explicit commit:
cancelling subsequent Participant setup may leave the explicitly created empty
Case, never a falsely complete profile. Human Principal identity is never linked
to the model executor. Policy, trust, qualification, review and Grants remain
separate authority chains.

Inside the same Case, `/connect`, `/attach`, `/policy publish`, `/admit`,
`/workflow bind` and `/review` guide missing input through existing typed
application actions. Review displays the exact operation before approve/deny/defer
and uses the authenticated human Participant. Bare `/retry` resolves the latest
canonical Turn of that Participant/current thread; the unchanged recovery
algorithm still enforces exact intent and uncertain-delivery safety. Explicit
forms remain available. `/help` is compact; `/help all` preserves full discovery.

The cumulative [ZERO-TO-CURRENT runbook](../../../docs/zero-to-current.md) now
uses this journey. It is not a new disconnected manual. The operator's existing
`/tmp/yai-golden-human` data was not used, reset, repaired or inspected by tests.

## Legacy archaeology and ownership verdict

Read-only `../yai-dev` at `5c1c7b9d099eea9f2947146cd821d6501c4a6ddf`:
`c48191f36355f266fd620877470db6ea066ec3a0` distinguished provider/model CLI identity
but used global runtime provider-selection files; legacy session/current-Case
metadata depended on the old runtime. `5bbbc72b80064f6f56610c9c1da379a8dc4599f1`
and the historical CLI surface tests supported help/dispatch consistency but
not current Tenant-scoped admission. Recover exact provider-versus-model naming
and shared command discovery, not global selection state, Subject/runtime
ownership, host scans or historical editor code. Current canonical authorized
Case/Participant contracts and REPLAI are the stronger executable mechanisms.

Schema delta **0**: Transition **v18**, CaseState **v15** remain unchanged.
Semantic owners **+0**, operational owners **+0**, databases **+0**, LMDB **37/40**.
Registry schema v1 is unchanged; its content digest necessarily changes for
the new Product operation. No provider/cognitive/effect/review schema changes.
REPLAI pin and R5 vendor removal unchanged. Root README unchanged.
I01–I06 COMPLETE; I07 UNSELECTED. No post-I10/H20/W21/W22 implementation.

## Qualification

Actual command evidence is retained in [publication.json](publication.json),
[product.jsonl](product.jsonl), [golden-free.jsonl](golden-free.jsonl),
[golden-workflow.jsonl](golden-workflow.jsonl) and
[final-checks.json](final-checks.json). Each Product/Golden stream retains its
own run ID, order, cwd/home, commands/actions, process exits and produced IDs;
runs are never spliced together. PTY output is raw hex; retained prefixes/suffixes
are explicitly marked excerpts. Repeated full history dumps are omitted; exact
replay checks and terminal/result identities remain. Tests classify separately:

| Proof | Provider mode | Scope |
|---|---|---|
| Product | no_provider | PASS: real launcher/PTY init, open, cancellation, existing Case, exact selection and human/model separation; declined trust registers no target |
| Recovery | no_provider | PASS: atomic Participant batch, stale/mixed rejection, replay and reopen |
| Publication union | no_provider + loopback_fixture | PASS: `make test-release characterization`; static/unit/component/contract/product/recovery and existing bounded endurance remain reachable |
| Golden Product | loopback_fixture | PASS: free/Workflow through guided actions, real SQLite/HTTP/MCP/process, review, retry, replay and isolation |
| External YVEX | external_yvex | NOT_RUN / DEPLOYMENT_LIMITATION: no operator endpoint/model supplied |

YVEX EXTERNAL FINDINGS: **DEPLOYMENT_LIMITATION**. No YVEX source, CLI, protocol,
model-loading or deployment administration was used. No live interoperability
or model intelligence claim follows from the loopback qualification.
CONTINUITY_CANARY = NOT_RUN; operator-owned state remains untouched.
HUMAN_GOLDEN_CASE = PENDING_OPERATOR.

The final Golden runs are `yai-golden-free-6l2dcju4` (free, 15 model dispatches)
and `yai-golden-free-u_pewz89` (Workflow, 18). Both exercise ALLOW/DENY/REQUIRE_REVIEW,
process exit 1 then 0, exact immutable admissions, canonical replay and derived
index rebuild. The Workflow run validates/adopts the model's candidate PlanPatch;
the free run includes scoped Handoff/isolation. Bare retry and reopening do not
repeat completed provider work. These are deterministic model fixtures, never a
real DeepSeek/Qwen or human claim.

Coverage audit: 71 historical smoke leaves remain reachable, 84 current Make
leaves, zero duplicate shared Make leaves in the combined invocation; 467 catalog
entries and 368 Rust tests discovered. Counts supplement, not replace, the proof
classes above. CLI Clippy passes with `-D warnings`; engine Clippy retains only
the existing `too_many_arguments`/`should_implement_trait` exceptions. The
development PTY test caught a missing Product-root registry entry before closure;
registry/help/completion are now audited together. No test semantics were relaxed
to obtain shorter command syntax.

## Remaining boundary

Resource definitions, policy sources and Workflow definitions remain exact
administrator-supplied documents; a short prompt is not a policy authoring UI.
Discovery admission still needs the inspected candidate digest. Advanced exact
IDs remain appropriate for Handoff and PlanPatch forensic/admin actions. No
hidden automatic trust, policy publication, model-name suitability or global
resource discovery was introduced to shorten commands. Real YVEX/native-function
qualification and operator acceptance remain separately required.
