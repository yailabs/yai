# Wave 8 — Governance intake and PolicyArtifacts

State: complete, committed and published.

Baseline: `3403ecdd2a321b689e41d747cbeb9d9e7c58e5e1`.
Final/published SHA: `7ce67afee34a3dbe879c2e5bee945602492be70c`.

## Direct legacy archaeology

Directly inspected `yai-dev` commit/epochs:

- `45c36bd0c13eee845a4505012fe4a2af44651433`: original executable
  ingestion/authoring integration;
- `575f76fcd`: canonical governance-root runtime loader cutover and real
  workspace attach eligibility consumer;
- `c5501101f`: law→governance unification/topology change;
- `1ebfb8d84`: neutral-sample cleanup, no semantic compiler strengthening;
- `2eeb8ea22`: contracts removal/protocol unification and regenerated files;
- `3cb2e94a4`, `a22476726`, `526c824bd`, `7b3c1b7bc`: embedded-copy removal and
  repeated topology drains;
- `2a4018147` and current `5c1c7b9d`: final governance intake residue reduced
  to a bridge comment/stub.

Inspected code, fixtures, schemas and tests under:

```text
governance/ingestion/{sources,parsed,normalized,candidates,review,templates,examples}
tools/gen/deterministic_governance_ingestion.py
tools/bin/yai-govern
tools/validate/validate_*governance*
governance/registry/governable-objects.v1.json
lib/runtime/session/session_utils.c
lib/runtime/session/utils/session_utils_helpers_events.inc.c
lib/runtime/session/utils/session_utils_surface_mutations.inc.c
lib/governance/loader/
src/decision/basis/decision_ingest_governance_drain.c
include/yai/knowledge/ingest/governance.h
```

The strongest historical parser was deterministic and non-LLM. It accepted
restricted RULE lines from Markdown/YAML and structured JSON; emitted typed-ish
facts, normalized authority/evidence/precedence/exception candidates; retained
source line refs; exposed unresolved/conflicts; and separated candidate,
review-ready, approval and attachability intent.

The real runtime attach consumer did not read ingestion candidate files. It
read a separate `governable-objects.v1.json` registry and checked target,
attachment mode, `runtime_consumable`, status and review state. No executable
candidate→registry promotion was found. Thus “candidate can be generated” did
not prove “candidate reaches runtime.”

Historical limitations found directly:

- no exact-byte/source digest identity;
- generated timestamps and month versions made rebuild output nondeterministic;
- candidate and review files were mutated in place;
- tracked JSON files, not canonical append-only persistence;
- missing `jsonschema` produced a warning rather than fail-closed validation;
- unresolved ambiguity was a review warning rather than a hard blocker;
- the approve path evaluated apply eligibility against pre-update candidate
  fields and tests accepted either `approved` or `apply_eligible`;
- broad schemas and topology metadata exceeded demonstrated consumers;
- registry/overlay/compliance/authority forests had no equivalent complete
  supply-chain consumer.

## Current YAI differential before implementation

Current YAI had only resource-attachment-local strings and enums:

- `ResourceAttachmentState.policy_id`;
- `policy_owner_participant_id`;
- `ReviewRequirement::{Automatic,RequireReview}`;
- `DecisionSource` and policy refs in the controlled filesystem effect.

These are real consumers but do not constitute source intake, shared immutable
artifacts, versions, publication or Case-independent policy history. Existing
LMDB already separated canonical Transition/CaseState, derived memory/context,
local resource bindings and runtime admission, and supplied the correct
physical storage technology.

## Refounded implementation

One cohesive `engine/yai-engine/src/governance.rs` owns the compiler and typed
contracts. One thin `cmd/yai/src/policy.rs` owns CLI rendering/dispatch. Existing
`store/lmdb.rs` owns four additional LMDB databases and atomic lifecycle writes.
No C semantics or public header were added.

Input `yai.policy_source_input.v1` is constrained JSON, max 256 KiB and 128
rules. Exact bytes produce `yai.policy_source_artifact.v1` SHA-256 identity.
Full bounded UTF-8 is retained for reproducibility and withheld by default on
inspection.

Implemented parsed facts:

- `operation_restriction` with typed ALLOW/DENY posture;
- `review_requirement` with typed boolean;
- `evidence_obligation` with pre/post observation, audit reason or source
  provenance obligation.

Each fact includes deterministic ID, source artifact and JSON location.
`yai.policy_ir.v1` deterministically orders/deduplicates rules, retains all
source refs, preserves unknown kinds as unresolved and detects contradictory
outcomes for the same typed selector. Blocking ambiguity/conflict never
qualifies.

`yai.policy_artifact.v1` is immutable and binds policy key/version/owner,
source/digest, full parsed facts/digest, IR/digest and deterministic validation.
The append-only `yai.policy_lifecycle_event.v1` stream derives:

```text
candidate → validated → published → superseded | retired
```

Publishing P@2 atomically appends superseded(P@1, related=P@2) and
published(P@2); it does not rewrite P@1. `runtime_consumable` is derived only
when validation is qualified and current lifecycle is published. It does not
mean attached, effective or authoritative.

The storage verdict is an independent canonical governance stream inside the
existing LMDB environment. It has immutable source/artifact DBs plus lifecycle
event identity/order DBs. It is neither CaseState nor a synthetic system Case.
Artifacts can exist with zero Cases and can later be referenced by multiple
Cases.

## Failure, retention and trust semantics

- malformed/future input: rejected before artifact;
- unknown kind: candidate retained unresolved, validation/publication blocked;
- known malformed rule: rejected;
- conflicts: typed and blocking;
- exact duplicate: idempotent source/artifact/event identity;
- source edit/version: new immutable identities;
- source payload loss: retained artifact keeps source digest, parsed facts, IR
  and provenance, but exact original bytes are explicitly unavailable;
- inspect/list: pure, no lifecycle or Case Transition append;
- local `--as`: provenance claim only, not OS/SSO/cryptographic authentication.

Wave 8 deliberately emits no Case PolicyBinding, EffectivePolicy, Decision,
ExecutionGrant, provider Invocation or filesystem effect.

## Tests and validation

- 69 Rust engine tests pass, including deterministic compiler, malformed/
  future input, unresolved/conflict blocking, lifecycle integrity, idempotent
  Case-independent intake, P@1/P@2 supersession/restart, retirement, source-loss
  behavior and query purity.
- `make smoke-governance-intake` passes the real CLI lifecycle and proves zero
  Case Transitions/materializations.
- `make check`: pass, including all Wave 2–7 smoke proofs and the new
  governance characterization;
- `make characterization`: pass. The managed filesystem sandbox twice denied
  the legacy daemon test's Unix-socket creation in the combined target; the
  isolated target passed, and the complete suite passed when rerun with the
  required socket permission;
- `make smoke-semantic-continuity`, `make smoke-agentless-case-runtime`, and
  `make smoke-human-review-runtime`: pass as constituents of `make check`;
- 26-turn agentless proof and 128-iteration endurance proof: pass;
- docs/layout checks: pass;
- `cargo fmt --check`: pass for engine and CLI;
- `git diff --check`: pass.

The tracked repository count moves from 785 to 789 after staging the four new
files. C/header/Rust source owners move from 150 to 152; no C or public header
is added. `main.rs` moves from 1,910 to 1,926 lines: the 16-line change is
module import, dispatch, usage and one info line only; policy semantics remain
outside it.

## Direct rediff

Recovered or strengthened:

- staged source→facts→IR→candidate supply chain;
- deterministic non-model parser;
- typed fact families with source location;
- unresolved/conflict visibility;
- candidate not runtime-consumable;
- version lifecycle and historical inspectability;
- source/artifact cryptographic identity;
- immutable artifact + append-only lifecycle;
- canonical persistence and restart;
- fail-closed validation and executable negative tests.

Rejected:

- giant governance/compliance/overlay/authority trees;
- registry without a demonstrated candidate promotion path;
- mutable file lifecycle;
- Markdown/YAML breadth without a current consumer;
- confidence/metadata fields without invariants;
- Agent, supervisor, Workflow, Space and C/Rust semantic duplication.

## Exact Wave-9 semantic delta

Wave 9 must freshly reinspect `yai-dev` Case qualification, policy attach,
loader/resolution, precedence/overlays and missingness. It must implement exact
immutable Case PolicyBinding plus deterministic EffectivePolicy materialization
and normative readiness. It must not yet turn that materialization into
Decision authority or policy-bound Grants (Wave 10), and it must not begin
cancellation/closure (Wave 11).
