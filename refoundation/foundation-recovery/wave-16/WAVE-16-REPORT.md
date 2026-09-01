# Wave 16 — CLI Product Refoundation

State: implementation and local qualification complete; publication pending.

- Baseline: `67fbf2a96b924f596caf0bc74976e402eda34bc6` on `master`, equal to
  `origin/master` and the remote branch before implementation.
- Intended semantic commit: `refactor(cli): establish YAI product command architecture`.
- Historical dirty work: all 13 entries remain preserved and excluded from the
  Wave whitelist; tracked checksum remains
  `3fdb219654405e6fd40b5c0d1b02b94c04fadef5aa57a139aa5fb8fd6db7777e`.
- YVEX provider qualification: `blocked_external_dependency`; neither
  `YAI_EXTERNAL_PROVIDER_BASE_URL` nor `YAI_EXTERNAL_PROVIDER_MODEL` was
  supplied.

## Archaeology and CLI-family reference

Direct `yai-dev` inspection covered the command-registry epoch at
`73e87be6e9c32539c5b902ac49548da583c1f5cb`, its pinned `yai-cli` source at
`e9ae6a`, and its pinned `yai-law` source at `44bad9`. The useful recovered
mechanism is one deterministic command metadata source for path, syntax, help
and discovery. The rejected design is the 2,800-command/14-group taxonomy,
where the catalog projected internal nouns and accumulated authority-like law
metadata. W16 recovers interface coherence, not the historical command
universe or shell runtime.

The current read-only YVEX `models1` checkpoint inspected for CLI architecture
was `e6f8ac71ac862945b9dd500fb9d6043e21147064`. Inspection covered
`config/operator/registry.json`, decision 0002, command architecture, CLI
input/I/O/render/schema code, registry generation and QA. Recovered family
properties are stable operation identity distinct from spelling, a compiled
versioned registry, one execution lane, explicit visibility, centralized
syntax admission, registry-derived help/discovery/completion, typed output,
explicit aliases/removals and `NO_COLOR`/non-TTY discipline. YAI intentionally
keeps different domain nouns, registry representation and JSON schemas.

The permanent family contract is recorded in
`YAILABS-CLI-FAMILY-CONTRACT.md`. The YVEX source study is an architecture
reference only and does not change YAI's black-box provider contract.

## Before and corrected product boundary

The H15 binary exposed roughly 31 roots through a handwritten source-tree
usage projection. `cmd/yai/src/main.rs` was 2,024 lines, with 121 usage-block
lines, 109 hard-coded usage print lines, approximately 100 top-level dispatch
branches and mixed rendering/diagnostic utilities.

The preserved pre-fix reproduction demonstrated:

- bare `yai` exited 2 instead of presenting a product map;
- `doctor` said `ok` while every required local layout directory was missing;
- `case status` and `case stop` leaked a missing checkpoint pathname for a
  valid never-started Case;
- `tenant status` help presented an optional selector while its handler
  required `--tenant`;
- `case enter` exposed Participant admission plumbing without a discoverable
  normal onboarding path;
- `process observe` and `observe process` duplicated one operation;
- direct process/effect diagnostics appeared beside ordinary governed work;
- direct effect helpers repeated endpoint/model configuration even though Case
  provider attachment already existed.

The final default projection has 12 memorable roots: `init`, `doctor`, `case`,
`workflow`, `review`, `policy`, `tenant`, `identity`, `runtime`, `help`,
`version`, and `completion`. Store, journal, projection, graph, facts, carrier,
process and direct effect tooling remain reachable through advanced help.

## Registry, parser, lanes and visibility

`yai.cli.registry.v1` is one compiled typed Rust descriptor registry. The
candidate registry digest is
`sha256:8f40be2da9ab2d94f0ab832d6d687f206c2755d9134c70adcb5d015a889163d9`.
It contains 121 unique canonical operations, 282 flags and 9 explicit aliases:
50 Product, 9 Advanced, 45 Plumbing, 16 Compatibility and 1 Removed.

Every descriptor carries stable operation ID, canonical path, syntax,
visibility, exactly one lane, mutation posture, output capability, handler ID,
aliases and removal metadata. Registry self-checks reject duplicate IDs/paths,
ambiguous aliases, invalid removal successors, malformed argument contracts and
unresolvable handlers. The exhaustive generated audit proves all 121 canonical
help paths parse successfully and no dispatch path exists outside the registry.

The final lanes are `LocalDomain`, `RuntimeHost`, `RuntimeControl`,
`Inspection`, `Compatibility` and `LocalInteractive`. Visibility remains a
separate projection: Product, Advanced, Plumbing, Compatibility or Removed.
The central parser resolves the exact longest path once, parses the descriptor
once, produces one typed invocation and dispatches once. Domain owners retain
authorization and semantic validation.

## Output and first-use contract

Product execution produces typed `CliData`, rendered either as restrained
human tables/sections or deterministic `yai.cli.result.v1`. Errors use
`yai.cli.error.v1`. JSON never carries ANSI, truncation, human hint lines or
credential values. Human success uses stdout; errors/hints use stderr. Exit 0
means success, 2 means syntax/refusal and 3 means a domain/operational failure.
Advanced characterization exit codes remain intact.

Bare `yai`, `yai help`, advanced help, JSON discovery, leaf help, bash/zsh/fish
completion and bounded nearest-command diagnostics all derive from the same
registry. TTY rendering is line-oriented only; non-TTY output is undecorated,
and `NO_COLOR` disables ANSI.

`yai init` delegates to existing layout/security/Tenant owners and is
idempotent. `doctor` reports `NOT_INITIALIZED` with `yai init` remediation on a
fresh home and `OK` only after the owned prerequisites exist. It treats the
legacy daemon as optional.

## Case-first product grammar

Case is the primary work object. The Product surface now supports Case
create/list/show, explicit Participant roles, generic provider attachment,
filesystem/process Resource attachment, Case policy binding without manual
generation copying, optional Workflow binding/status/input, normal Review and
Runtime control. A Case without Workflow remains normal.

`case show` begins with canonical CaseState and then adds derived execution,
Workflow, provider/model, Resource, policy/Review and Effect summaries. A
missing checkpoint is `never_started`, never an OS-path error. Stopping such a
Case returns clean `no_active_execution`. Participant remains distinct from
Principal, no role is granted implicitly, and provider credentials are never
rendered.

The real product characterization initializes a fresh home, creates and shows
a never-run Case, stops it safely, adds deliberate Participant roles, attaches
a generic endpoint/model and fenced Resource, ingests/publishes/binds policy,
defines/binds a Workflow, runs one deterministic governed filesystem Effect,
and inspects the completed result. It uses no store, journal, projection,
graph, facts, carrier, direct effect trigger or raw WorkItem command. Provider
invocations are exactly zero.

## Compatibility and implementation footprint

Safe compatibility spelling resolves to the same operation ID and handler;
for example `case status --case CASE` resolves to `yai.case.show`. The duplicate
`observe process` path is explicitly removed with a replacement hint to
`process observe`. Direct signals and direct effect generators remain visibly
engineering-only and cannot masquerade as governed product work.

`main.rs` is now 12 lines and owns startup plus top-level exit only.
`command_adapters.rs` is the deliberately named operation-ID adapter to stable
existing handlers; it is not a `legacy_cli`, alternate parser or second command
tree. Cohesive `cli/registry.rs`, `parser.rs`, `help.rs`, `output.rs` and
`product.rs` own only the interface boundary.

No semantic owner, LMDB database or canonical schema was introduced. LMDB
remains 35/40, Transition and CaseState remain v10, WorkflowRun remains absent,
and existing Case/authority/resource/runtime owners remain authoritative.

## Qualification and reclassification

`make check`, `make characterization`, the full 154-test engine suite, full
22-test CLI suite, all smoke targets through workflow hardening, the new CLI
product smoke, registry audit, 26-turn runtime, Review crash suite, endurance,
Tenant/runtime/resource/Workflow regression, formatting, repository-contract
Clippy, docs/layout and `git diff --check` pass in the pre-publication state.

Foundation Recovery reclassification supported by this evidence:

- CLI product boundary: `refounded_proven`;
- command registry: `refounded_proven`;
- parser/help/discovery: `refounded_proven`;
- product output: `refounded_proven`;
- legacy CLI compatibility: `bounded_compatibility`.

W17 remains exactly adaptive Workflow: PlanPatch, immutable amendment lineage,
bounded graph mutation, subflow and same-Tenant typed Case handoff. W18 remains
provider governance: qualification, capabilities, trust, health, selection and
an optional YVEX-native extension. Neither begins in W16.

## YVEX EXTERNAL FINDINGS

Live provider qualification was not executed because the operator supplied no
`YAI_EXTERNAL_PROVIDER_BASE_URL` or exact
`YAI_EXTERNAL_PROVIDER_MODEL`. State is
`blocked_external_dependency`. The read-only YVEX CLI architecture study is
not a provider finding. No YVEX source, CLI, profile, artifact, engine or
session entered YAI provider semantics, and W16 establishes no new external
provider finding.
