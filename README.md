<!--
YAI

Copyright (c) 2026 Francesco Maiomascio.
All rights reserved.

This file is part of the source-available YAI repository. Use, copying,
modification, distribution, and production operation are governed by the
repository licensing documents, including LICENSE.md and docs/legal.md.
-->

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="docs/assets/brand/yai-readme-dark.png">
    <source media="(prefers-color-scheme: light)" srcset="docs/assets/brand/yai-readme-light.png">
    <img src="docs/assets/brand/yai-readme-light.png" alt="YAI logo" width="280">
  </picture>
</p>

YAI is a Case-centered system for governed work and durable semantic continuity.
Participants, policies, resources, conversations and Workflows operate over one
durable Case. Models contribute computation and proposals; they do not own its
history or acquire authority by producing an answer.

The current product combines a Rust application engine, canonical
Transition/CaseState persistence, governed cognitive execution and a native
REPLAI Case workbench. Its Golden Case exercises a bounded software-change
lifecycle with real resource adapters and a deterministic model provider.
External provider qualification and human acceptance remain separate gates.

Start with the [quick start](#quick-start), the
[architecture and ownership](#architecture-and-ownership) overview, or
[development and validation](#development-and-validation).
The [ZERO-TO-CURRENT runbook](docs/zero-to-current.md) owns complete operator
acceptance; [ROADMAP.md](ROADMAP.md) owns live maturity, direction and selection.

## Why YAI

An interaction can end, a provider can restart, and a model can be replaced
without making any of them the owner of the work. YAI's primitive is the
governed transformation of durable semantic and operational state:

```text
admitted Case state + current intent
  → scoped cognitive planning and exact target
  → provider result / proposed operation
  → current policy, authority and review where required
  → admitted Transition / governed external effect
  → new CaseState and derived views
```

A proposal is not permission. A successful model answer is not proof that an
external effect occurred. An uncertain delivery is not permission to repeat the
work against another target. The Case retains exact identities, provenance and
admitted consequences independently of the model's context or session.

## What is actually implemented

| Boundary | Current evidence | Important limit |
| --- | --- | --- |
| Case continuity | Typed Transition Ledger and replayable CaseState, committed together in LMDB; immutable content with exact provenance | Historical compatibility journals and derived stores are not alternate authority |
| Identity and governance | Principal/Tenant/Participant isolation; policy intake, publication, binding and materialization; Decisions, human review and Grants | Attaching a resource or model does not grant permission to use it |
| Cognitive execution | Capability suitability, pinned/arbitrated exact targets, plans/lanes, typed realization and bounded auxiliary-to-primary composition | Semantic suitability and mechanical support are independent; no model-name inference |
| Conversation and product host | Immutable multipart Turns and execution intent; guided setup and cognitive routing through the REPLAI-backed controller | Native content and function support depend on exact adapter/target qualification |
| Operational world | Bounded filesystem, process, SQLite, HTTP and MCP access; discovery/admission and governed effects | Not ambient shell, filesystem, database or internet access; adapters have explicit supported subsets |
| Workflow and derived access | Definition/progression/amendment, model-proposed PlanPatch and explicit adoption; scoped Handoff; graph, memory, indexes and analytics | Workflow does not own Case continuity; derived assertions and retrieval do not create authority |

These are bounded implemented contracts, not a claim of general production
readiness. [Current architecture](docs/architecture.md) records their source
owners and limitations; the [Golden evidence](refoundation/validation/golden-case-lifecycle/REPORT.md)
records the integrated lifecycle.

## Quick start

### 1. Build

Use a Linux development host with Rust/Cargo, a native C toolchain, GNU Make,
Python 3 and SQLite development libraries. The confined Golden process runner
has additional Linux requirements listed in the
[operator runbook](docs/zero-to-current.md#infrastructure-not-the-case-workflow).
Model weights are not part of YAI; a provider is a separate prerequisite for
cognitive execution.

From the repository root:

```sh
make build-rust
./yai help
```

Use the repository's `./yai` launcher. It executes the local build rather than
an unrelated installed binary. Building does not qualify runtime behavior.

### 2. Initialize and open a Case

For a disposable evaluation, select a fresh home, then follow the guided setup:

```sh
YAI_DEMO_DIR="$(mktemp -d /tmp/yai-demo.XXXXXX)"
export YAI_HOME="$YAI_DEMO_DIR/home"
./yai init
./yai open demo
```

`init` asks for the Tenant and organization, then explicit `create` consent.
`open` guides Case creation and Participant admission. Read the proposed roles
and type the requested confirmation word; Enter alone is not approval.
The model Participant does not inherit the operator's Principal identity.

Keep the same `YAI_HOME` to reopen this Case. For durable work, use a private
persistent directory instead of `/tmp`; creating a fresh home is not a retry.
Policy readiness, review authority and provider trust remain separate from
Participant setup.

### 3. Connect a provider

Inside the Case:

```text
/connect
```

Supply the public OpenAI-compatible endpoint of an already reachable provider.
YAI discovers its model catalog: one exact entry is selected automatically;
several entries require a choice. Credential references are requested when
needed, never secret tokens in ordinary input or URLs.

Review the exact target before typing `approve`. Approval permits synthetic
mechanical probes, explicit trust and an operator-attested pinned
PrimaryConversation binding. It does not fabricate semantic qualification or
grant resource authority. Text is required; native functions and JSON are
independently tested and reported. Probes execute real inference and can take
time on the selected provider.

Then submit ordinary text. SEND commits the Turn before provider execution.
Model replies and YAI system notices remain distinct; `/details` exposes
execution lineage on request. `/retry` keeps the same Turn and intent and
refuses unsafe redispatch after indeterminate delivery.

YAI does not launch YVEX, load a model or administer its runtime. A reachable
catalog or a passing small probe does not qualify a full Case workload. See the
[provider contract](docs/provider-governance.md) and
[operator runbook](docs/zero-to-current.md) for capability and deployment limits.

## Working inside a Case

The workbench is a native REPLAI consumer over the same application seams used
by one-shot commands. It does not shell out to another YAI process or own a
second chat history.

| Action | Purpose |
| --- | --- |
| `/help` / `/help all` | Compact actions or the complete interactive vocabulary |
| `/case` / `/participants` | Current Case and execution identities |
| `/attach` / `/resources` / `/artifacts` | Guided attachment and inspection of the admitted world |
| `/policy publish` / `/policy` | Governed policy setup and current readiness |
| `/work TASK` | Bounded model-requested capability work under Case authority |
| `/review` | Inspect an exact pending operation and record an eligible human action |
| `/workflow` | Inspect the bound Workflow; `/help all` exposes its actions |
| `/history` / `/effects` / `/details` | Canonical history, external outcomes and execution lineage |
| `/memory` / `/graph` / `/verify` | Derived views and replay verification |
| `/retry` / `/cancel` / `/exit` | Delivery-safe retry, truthful cancellation and leaving the interaction |

Resources, policy sources and Workflow definitions still need exact admitted
configuration. A short command is not ambient authority or a policy-authoring
engine. The [ZERO-TO-CURRENT runbook](docs/zero-to-current.md) supplies one
continuous reference lifecycle, including discovery, review, real test
execution, Workflow, isolation and restart. It also documents the operator-owned
continuity canary; automated qualification does not reset that long-lived Case.

## Architecture and ownership

| Plane | Owns | Does not own |
| --- | --- | --- |
| Transition Ledger and CaseState | Canonical Case history and current rebuildable authoritative materialization | Provider sessions or a global mutable memory bag |
| Immutable Content / Artifacts | Payload bytes and exact identity/provenance relations | Automatic admission of external or model claims |
| Policy and authority | Current scoped permission, Decision, review and Grant semantics | Permission inferred from a tool catalog or provider answer |
| Resources and effects | Admitted operational relations, bounded access and prepared/reconciled outcomes | Ambient access to a shared endpoint or filesystem |
| Workflow and Handoff | Explicit progression, amendments and scoped transfers | Replacement of Case continuity or cloning of source authority |
| Graph, memory, indexes and analytics | Rebuildable semantic access and derivation | Canonical history or promotion through similarity/repetition |
| Projection / ContextFrame | Current bounded working-set selection and provider preparation | A general State Compiler or durable model memory |
| REPLAI / product presentation | Terminal mechanics and application interaction | Cognitive routing, policy or semantic ownership |

`yai` is the primary Rust product/application boundary. `yaid` retains a bounded
C status/lifecycle and fixture surface; component-only C tests are not evidence
of a second general product engine. Source membership and cross-language limits
are described in [Architecture](docs/architecture.md).

### YAI and YVEX

The adopted direction is **YAI semantic cognitive state, YVEX computational
cognitive state**. Today's provider path is context-compatible. The target is
to compile only the task's governed working state, then let an execution
substrate lower it toward an exact model's computational representation.

This is a [target architecture](docs/semantic-state-execution-target.md), not
implemented SemanticStateFrame/Delta, a general State Compiler or native State
Read/Update. Models, chats and future Agent compositions remain consumers of
Case continuity, not canonical owners. Public capability contracts must carry
any future YAI/YVEX state boundary; YAI does not acquire KV, tensors or residency.

## Development and validation

Read [AGENTS.md](AGENTS.md) and [CONTRIBUTING.md](CONTRIBUTING.md) before editing.
Preserve unrelated work, identify the changed semantic owner and validate the
property actually affected. The [test topology](tests/README.md) separates
proof class from provider environment:

| Command | Evidence surface |
| --- | --- |
| `make test-fast` | Documentation/layout guards, unit and bounded component proof; no provider |
| `make test-local` | Deterministic local contract, product and recovery proof, including real loopback transport |
| `make check` | Deterministic publication union (`test-release`); no external provider required |
| `make test-golden-local` | Separate integrated Golden lifecycle; real resources, loopback model |
| `make test-external-yvex` | Explicit live-provider interoperability; no fixture fallback |
| `make test-golden-external-yvex` | Complete reference product lifecycle against an operator-supplied real target |

`characterization` is a behavior-freezing evidence posture, not another test
level. `make check characterization` shares identical Make leaves within the
same invocation. Recovery and endurance have independent diagnostic lanes;
[classification.tsv](tests/classification.tsv) owns exact reachability.

The cumulative human runbook is not a test driver. Automated PTY success is not
human acceptance, and Golden loopback success is not live YVEX qualification.
Use the [validation guide](docs/test-cases.md) for dependencies, scope and
evidence reporting.

## Documentation and source map

- [Documentation index](docs/index.md) — canonical navigation and authority map.
- [Constitution](docs/constitution.md) — long-lived invariants and ownership.
- [Architecture](docs/architecture.md) — current executable truth and limits.
- [Roadmap](ROADMAP.md) — sole live project control and target direction.
- [ZERO-TO-CURRENT](docs/zero-to-current.md) — cumulative operator acceptance.
- [Reference contracts](docs/reference/semantics.md) and
  [state/transitions](docs/reference/state-transitions.md) — semantic boundaries.
- [Provider governance](docs/provider-governance.md) and
  [REPLAI integration](docs/replai-terminal.md) — execution and interaction.

```text
cmd/           product CLI/application and bounded daemon entrypoints
engine/        Rust semantic contracts, transitions, stores and algorithms
system/        production C mechanics and separate component characterization
include/       C interfaces; directory names do not establish product reachability
tests/         property-classified tests, protocol peers and Golden reference world
docs/          canonical documentation, target doctrine and public brand assets
tools/         build, documentation, layout and validation tooling
refoundation/  bounded historical implementation and qualification evidence
labs/, work/   explicitly scoped experiments and historical records
```

## Current limits

YAI does not claim production readiness, general autonomous tool access,
arbitrary SQL/shell execution, complete MCP support, Studio or a canonical Agent.
The current Projection/ContextFrame and derived-memory foundations do not yet
prove a general state compiler, Case-age-independent working sets or complete
cold-model substitution.

Real YVEX synthetic text/function/JSON qualification has been observed, but a
subsequent Case SEND encountered an HTTP 413 capacity refusal. The complete
external Golden lifecycle is not qualified by those probes. Human Golden
acceptance remains operator-owned and unclaimed; the
[Roadmap](ROADMAP.md) and [runbook](docs/zero-to-current.md) carry the current
posture and remaining blockers.

## License

YAI is [source-available](LICENSE.md) for technical evaluation and review, not
offered under an OSI-approved open-source license by default. See
[legal posture](docs/legal.md),
[third-party notices](LICENSE.md#third-party-notices), [security](SECURITY.md) and
[contribution policy](CONTRIBUTING.md). [Brand assets](docs/assets/brand/README.md)
retain the supplied identity and do not change the repository's licensing terms.
