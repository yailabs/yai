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
    <img src="docs/assets/brand/yai-readme-light.png" alt="YAI logo" width="240">
  </picture>
</p>

<p align="center">
  The work should outlive the model.
</p>

<p align="center">
  <a href="docs/architecture.md"><img src="https://img.shields.io/badge/runtime-local-64748b?style=flat&amp;labelColor=334155" alt="Runtime: local"></a>
  <a href="#architecture-and-ownership"><img src="https://img.shields.io/badge/core-semantic%20state-64748b?style=flat&amp;labelColor=334155" alt="Core: semantic state"></a>
  <a href="LICENSE.md"><img src="https://img.shields.io/badge/license-source--available-64748b?style=flat&amp;labelColor=334155" alt="License: source-available"></a>
</p>

<p align="center">
  <a href="#quick-start">Quick start</a> ·
  <a href="#working-inside-a-case">Inside a Case</a> ·
  <a href="#yai--yvex">YAI + YVEX</a> ·
  <a href="docs/index.md">Documentation</a> ·
  <a href="ROADMAP.md">Roadmap</a>
</p>

AI work often carries its past forward by adding more text to the next context
window. As the work grows, the application must keep deciding what to replay,
what to summarize and what it can afford to forget.

**YAI is building around a different unit: the Case.** A Case holds the history,
evidence, decisions, resources and unresolved work of an ongoing matter.
Conversations and model executions happen within it. Its continuity belongs to
the work, not to the chat, the model or the provider session.

The goal is to reconstruct the experience that matters now, combine it with
what currently holds, and compile a bounded working state. Together, YAI and
YVEX are designed to let models carry that state computationally, rather than
always reconstructing the past from text. Replacing the model should change
the computation, not erase the Case.

YAI is under active implementation. This README describes the product direction
and [what you can run today](#run-yai-today); [ROADMAP.md](ROADMAP.md) tracks
exact maturity and current engineering state.

## The Case outlives the conversation

Consider a software change that takes several weeks. The issue arrives before
the implementation. A release constraint changes. Tests fail, a reviewer asks
for evidence, and an earlier decision turns out to rest on a false assumption.
The useful memory is not just the messages exchanged along the way. It is the
relationship between the request, the evidence, the decisions and what actually
happened.

A Case gives that work a durable home. A conversation explores it. A Workflow
organizes part of it. A human reviews a proposed change. A model investigates
an unresolved question. A terminal session or an automated evaluation accesses
the same Case under its own permissions. Leaving one interaction does not
mean exporting its memory into the next.

The Case retains both consequences and unfinished work: what was observed,
what was decided, which sources supported it, what remains uncertain and which
resources are available. Models contribute computation and proposals over
that state. They are replaceable participants, not its owners.

## Memory beyond context

**What happened is not the same as what holds now.** A rejected approach can
still explain a later decision. A superseded constraint may be essential to
understanding an old failure while being wrong for today's release.

YAI's target is temporal-causal **Recall**: reconstructing the relevant
experience for a particular question, even when it spans Episodes separated
by months of unrelated work. Recall should recover what was recorded then,
what changed, what supported a decision and where the exact evidence lives.
Where causality is not established, it must preserve that uncertainty rather
than turn chronological order into an explanation.

Vector similarity, lexical search, graph traversal and possible learned or
sparse representations can help find the path. They do not decide what is
true. A useful Recall Trace must resolve back to its sources, retain historical
supersession and respect the reader's current access. Missing evidence stays
missing; a plausible summary cannot replace it.

The target is not a larger prompt. It is a working state shaped by the task:
relevant experience plus current facts, unresolved questions and Case-level
constraints, within an explicit budget. A Case should be able to grow for years
without requiring every year to become the next prompt. Difficult work may
need more state; unrelated age should not be the reason.

## YAI + YVEX

YAI preserves model-independent meaning. YVEX determines how an exact model can
carry and compute over it.

This separates two problems that a context window tends to collapse. YAI must
decide which experience and current state matter for this task and Participant.
YVEX must realize that qualified working state in a form the model can use,
with an exact execution, residency and recovery contract.

The architecture we are building is one feedback loop with distinct semantic
and computational ownership:

```text
WORLD / HUMAN
      |
      v
YAI CASE <----------------------------------------------------+
  |                                                           |
  +-- Historical experience H -> Recall R --+                 |
  |                                         |                 |
  +-- Current semantic state S -------------+                 |
                                            v                 |
                                   State Compiler -> W        |
                                 intent / scope / budget      |
                                            |                 |
                             public cognitive-state boundary  |
                                            v                 |
                                     YVEX -> State E          |
                                            |                 |
                    immediate input X ---> MODEL              |
                                            |                 |
                                output / action proposal      |
                                            |                 |
                               YAI validation + admission ----+
```

**W** is Semantic Working State: what must count for this execution, not the
entire Case. **E** is Experiential Computational State: its model-native
realization, reusable across an execution history. The target includes
**State Read** and **State Update**, so a model can consume and evolve
computational state beyond the immediate input stream.

Today's YAI compiles W and lowers it into ordinary provider context. The
research path extends from qualified reusable prefix/KV state to learned or
architecture-native persistent state. Physical representations, checkpoints
and model-side mechanisms belong to YVEX and the model; YAI does not need to
become a tensor runtime to preserve semantic continuity.

Computational state can evolve without changing what the Case knows. **E → E′
does not imply S → S′.** Output remains output; only an explicit proposal or
observed consequence can pass through YAI's admission rules. Replacing a model
may require discarding its computational state and rebuilding from the Case,
not rebuilding the Case from scratch.

## Beyond request and response

### When the world changes, working state should follow

A test finishes. A review resolves. Another Participant supplies evidence.
A Workflow advances or a source changes. None needs to begin as another human
message.

The continuous-feedback target is for relevant admitted changes to refresh
Recall and working state, and eventually reconcile the model's computational
state before its next affected read. Revoked access or invalidated evidence
must prevent stale consumption; useful enrichment can happen asynchronously
within bounds. Continuity does not mean invoking a model endlessly when there
is no authorized work.

### An assignment can outlast the exchange

> Review everything we learned about this subsystem today. Reconsider the
> unresolved assumptions. We'll continue tomorrow.

Persistent deliberation is the research direction behind that request. YAI
would retain the assignment, sources, permissions, compute budget, deadline and
cancellation conditions. A compatible model/runtime could continue bounded
internal computation, revisit qualified Case experience, checkpoint unfinished
deliberation and return with new candidate conclusions or questions.

That is different from keeping a tool loop running overnight. Thinking need not
act on the environment. Testing a hypothesis against a real resource still
requires ordinary governed execution. Reusable experiential state and unfinished
deliberation have different lifecycles: losing a latent line of thought must
not lose the admitted assignment or Case history. YAI needs justified candidate
outputs, not a transcript of hidden reasoning promoted into memory.

The [target doctrine](docs/semantic-state-execution-target.md) develops this
design; the [Roadmap](ROADMAP.md) records the work still needed to realize it.

## Run YAI today

There is already a local Rust application with durable Case persistence, an
explicit semantic-state/working-state compiler and a native **REPLAI Case
workbench**. You can create a Case, connect a provider, converse, attach
resources, inspect policy, review operations and follow their consequences.
Bounded filesystem, process, SQLite, HTTP and MCP capabilities share the same
authority path. Workflow progression, scoped Handoff and rebuildable
operational, episodic and semantic access are part of that foundation.

The **Golden Case — Governed Software Change & Release Qualification** gives it
a concrete workload: investigate a small defect using source, database, service
and MCP evidence; propose a reviewed repair; run the real test; continue through
Workflow, restart and replay. Start with the interactive path below, then use
[ZERO-TO-CURRENT](docs/zero-to-current.md) for the complete operator lifecycle.
[Architecture](docs/architecture.md) documents the exact implemented contracts.

## Quick start

### 1. Build

Use a Linux development host with Rust/Cargo, a native C toolchain, GNU Make,
Python 3 and SQLite development libraries. The confined Golden process runner
has additional Linux requirements listed in the
[operator runbook](docs/zero-to-current.md#infrastructure-not-the-case-workflow).
Model weights are not part of YAI; a provider is a separate prerequisite for
cognitive execution.

Clone and build:

```sh
git clone https://github.com/yailabs/yai.git
cd yai
make build-rust
./yai help
```

Already cloned? Run the last two commands from the repository root. The build
creates the local `./yai` launcher; it executes this checkout's binary.

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
Participant setup does not by itself grant policy, resource or provider access.

### 3. Connect a provider

Inside the Case:

```text
/connect
```

Supply the public OpenAI-compatible endpoint of an already reachable provider.
YAI discovers its model catalog: one exact entry is selected automatically;
several entries require a choice. Credential references are requested when
needed, never secret tokens in ordinary input or URLs.

Review the exact target before typing `approve`. This explicitly approves
mechanical probes, provider trust and your suitability attestation for the
conversation role; it grants no resource authority. Text, native functions and
JSON are tested separately. Probes perform real inference, so setup time depends
on the provider. Answer each guided question before entering the next action.

Then submit ordinary text. YAI records your Turn before execution and keeps
model replies separate from system notices. `/details` exposes execution
lineage; `/retry` preserves the same Turn and refuses unsafe redispatch after
uncertain delivery. `/cancel` gates further work, not a guaranteed abort of an
already dispatched request.

YAI does not launch YVEX or load models. The provider must already be reachable
and have capacity for the intended workload; a catalog entry or small probe
alone does not establish that. See [provider setup and contracts](docs/provider-governance.md)
for exact capability and deployment requirements.

## Working inside a Case

Remain in the Case while inspecting and advancing the work. These are
interactive actions, not shell commands; use one at a time.

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

Resource operations require admitted bindings and ready policy; they are not
ambient tools. The [runbook](docs/zero-to-current.md) supplies the reference
resources and the complete free-work/Workflow procedure. It also explains how
to retain an operator-owned continuity canary across upgrades. One-shot CLI
commands remain available for automation, administration and exact inspection.

## Architecture and ownership

The product thesis rests on boundaries that can be inspected and tested:

- **History and current authority.** The Transition Ledger is canonical Case
  history; CaseState is its rebuildable current materialization, committed
  atomically in LMDB. Immutable Content/Artifacts retain payloads that the ledger
  cannot reconstruct. Historical experience H is a qualified view of existing
  owners, not another ledger.
- **Meaning and selection.** The current `SemanticState` composes replay-qualified
  history and state without a new store. The compiler produces deterministic W
  for an exact generation, Participant, intent, disclosure and budget. Control,
  observations, derived assertions and model claims retain their distinct
  evidence status. Relevance cannot widen access or silently omit required state.
- **Compilation and compatibility.** Projection and ContextFrame lower W into
  provider-compatible input; they do not own semantic selection or memory.
  Working state is disposable; lowering rejects stale working state. Derived
  semantic deltas have checked full-compilation equivalence; their current
  application uses full recompilation, not an incremental speedup.
- **Admission and consequences.** Model output does not create authority.
  Operations require current policy, Decisions, review where required and
  finite Grants. External effects retain PREPARE, terminal evidence and
  reconciliation; ambiguous delivery is not permission to repeat an action.
- **Derived access and product views.** Graph, operational memory, Episodes,
  semantic assertions, indexes and analytics remain rebuildable. Neither
  retrieval rank nor a repeated claim promotes truth. Workflow/Handoff preserve
  scoped continuity without cloning authority. REPLAI owns terminal mechanics;
  YAI owns application actions and content classification. An optional Agent
  composition would own none of this merely by existing.

The H/S/Recall/W/E diagram expresses the target architecture, not a renaming of
every current type. General Recall and a public model-state consumer remain
targets. YAI owns neither tensors, latent state nor GPU placement; YVEX
does not acquire Case, Policy or Workflow authority. Current provider execution
uses generic OpenAI-compatible contracts, not model-name branches or private
YVEX protocols.

For source-level ownership, read [Architecture](docs/architecture.md).
`cmd/yai/` hosts the Rust application; `engine/yai-engine/` holds reusable
semantic contracts and algorithms. `cmd/yaid/` and the production subset of
`system/` provide the bounded C daemon/platform surface, not a second semantic
engine.

## Development and validation

Build it, follow a Case through the [Golden workload](docs/zero-to-current.md),
and inspect where the architecture holds or breaks. For development, start with
[CONTRIBUTING.md](CONTRIBUTING.md) and [AGENTS.md](AGENTS.md). The
[validation guide](docs/test-cases.md) selects evidence for the boundary being
changed:

| Command | Evidence surface |
| --- | --- |
| `make test-fast` | Documentation/layout guards, unit and bounded component proof; no provider |
| `make test-local` | Deterministic local contract, product and recovery proof, including real loopback transport |
| `make check` | Deterministic publication union (`test-release`); no external provider required |
| `make test-golden-local` | Separate integrated Golden lifecycle; real resources, loopback model |
| `make test-golden-external-yvex` | Complete reference product lifecycle against an operator-supplied real target |

Local Golden uses a deterministic model with real persistence, resource
adapters and protocol peers. External YVEX qualification and human acceptance
are separate: neither follows from local success. Their live posture belongs
in the [Roadmap](ROADMAP.md), not a historical test count. This is development
software, not a production-readiness claim.

Golden is also the intended common workload for deeper research: compare
context and persistent-state execution, correct and misleading memories,
source-closed Recall, and deliberation against equal-compute baselines. The
question is whether state improves justified task outcomes, not just whether
more material can be retrieved.

## Documentation

- [Documentation index](docs/index.md) — canonical navigation and authority map.
- [Constitution](docs/constitution.md) — long-lived invariants and ownership.
- [Architecture](docs/architecture.md) — current executable truth and limits.
- [Roadmap](ROADMAP.md) — sole live project control and target direction.
- [Semantic-state target](docs/semantic-state-execution-target.md) — Recall,
  continuous feedback, experiential state and deliberation.
- [ZERO-TO-CURRENT](docs/zero-to-current.md) — cumulative operator acceptance.
- [Reference contracts](docs/reference/semantics.md) and
  [state/transitions](docs/reference/state-transitions.md) — semantic boundaries.
- [Provider governance](docs/provider-governance.md) and
  [REPLAI integration](docs/replai-terminal.md) — execution and interaction.

## License

YAI is [source-available](LICENSE.md) for technical evaluation and review, not
offered under an OSI-approved open-source license by default. See
[legal posture](docs/legal.md),
[third-party notices](LICENSE.md#third-party-notices), [security](SECURITY.md) and
[contribution policy](CONTRIBUTING.md). [Brand assets](docs/assets/brand/README.md)
retain the supplied identity and do not change the repository's licensing terms.
