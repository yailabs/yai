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
    <source media="(prefers-color-scheme: dark)" srcset="docs/assets/brand/yai-readme-horizontal-dark.png">
    <source media="(prefers-color-scheme: light)" srcset="docs/assets/brand/yai-readme-horizontal-light.png">
    <img src="docs/assets/brand/yai-readme-horizontal-light.png" alt="YAI" width="640">
  </picture>
</p>

<p align="center">
  <strong>Durable semantic state. Case-bound AI execution.</strong><br>
  The work should outlive the model.
</p>

<p align="center">
  <a href="#quick-start">Quick start</a> ·
  <a href="#architecture-and-ownership">Architecture</a> ·
  <a href="#security-without-model-compliance">Security</a> ·
  <a href="docs/index.md">Documentation</a> ·
  <a href="ROADMAP.md">Roadmap</a>
</p>

**YAI is a local control plane for governed AI work.** It gives an ongoing
matter—a **Case**—durable history, qualified memory, current authority and
mediated access to resources. Conversations, Workflows and replaceable models
operate inside that boundary; none owns the Case merely by executing.

YAI is not a model, an inference engine or a retrieval layer around a prompt.
It determines what should matter semantically and what may happen operationally.
Today, generic providers execute context-compatible input compiled by YAI.
[YVEX](https://github.com/yailabs/yvex) is the computational substrate counterpart;
a public model-native working-state contract remains a target.

This is actively developed systems software with bounded executable
qualification, not a production-readiness or certification claim.
[ROADMAP.md](ROADMAP.md) owns live maturity and selection.

## Why a Case

An investigation, software change or operational decision outlasts any one chat.
Its useful state includes evidence, rejected approaches, policy, approvals,
resources, consequences and unresolved obligations—not just message history.

The Case keeps those relationships under explicit identity and authority.
A new Turn supplies immediate input; the system reconstructs relevant memory
and compiles the semantic working state for that execution. Closing a terminal,
switching models or rebuilding a derived index does not erase canonical history.

| Question | YAI boundary |
|---|---|
| What persists? | Committed Transition history and exact owned content/artifacts; CaseState materializes current Case meaning. |
| What is rebuildable? | CaseState, knowledge derivations, graph/indexes, Recall, working state and compatibility projections. |
| Who has authority? | Authenticated Principals and admitted Participants, current Policy/admission, review and finite Grants—not model output. |
| What can a model cause? | Proposals evaluated through existing Case, Resource and effect owners. Knowing a tool name or Resource ID grants nothing. |
| What follows a restart? | Canonical lifecycle and exact provenance; uncertain external delivery requires reconciliation, not blind replay. |

## Architecture and ownership

One semantic compiler sits between qualified memory and model execution.
The authority plane remains outside model computation:

```text
SOURCES / OPERATIONAL WORLD                 CASE AUTHORITY
        │                                  identity · Policy · review
        ▼                                  Grants · Resource scope
governed source frontier / admitted events          │
        │                                          │
   D: source statements   H: experience   S: current meaning/control
        └─────────────────┬──────────────────┘     │
                          ▼                        │
                 qualified Recall                  │
                          │                        │
              S + Recall + task + constraints ◄────┘
                          ▼
                bounded working state W
                          │
         ┌────────────────┴───────────────────┐
         ▼                                    ▼
context-compatible lowering          future public W → YVEX E
Projection / ContextFrame            NOT IMPLEMENTED
         │
         ▼
generic provider / model  ◄── immediate input X
         │
         ▼
output / tool request / semantic proposal
         │
current YAI admission + Case-bound Resource mediation
         ▼
governed consequences / canonical evidence
```

Ordinary governed Conversation and Workflow execution use the same typed
Recall-aware working-state operation before compatibility lowering. Manual
Recall, context compilation, paging and refresh commands remain inspection and
administration surfaces; they are not prerequisites for an ordinary Turn.

A new invocation requalifies current sources, disclosure and control. Projection
does not retrieve, expand pages or reinterpret authority. Failure to construct
the required working state does not silently downgrade that governed execution
to S-only context. Historical compatibility paths retain their explicit
[contracts and limits](docs/recall.md).

## Qualified memory, not a second truth store

| Semantic material | Meaning |
|---|---|
| **D — domain knowledge** | What admitted exact source revisions state, with source coordinates and extraction provenance. |
| **H — experience** | What was recorded in the Case, including observations, Decisions and consequences. |
| **S — current state** | What currently holds through established semantic and authority owners. |
| **R — Recall** | Task-conditioned reconstruction across D/H/S, with qualified relations, missingness and current disclosure. |
| **W — working state** | The bounded semantic material selected for this execution: mandatory current constraints plus relevant evidence. |

Bounded D/H/S Recall, Recall-aware W, exact-group semantic paging and explicit
same-task refresh are implemented. Search discovers candidates; deterministic
resolution establishes identity, scope, historical applicability and backing.
Ranking does not decide truth, authority or causality.

A document can state “retention is 90” while an observation records 30.
Recall and W preserve both epistemic classes and unresolved disagreements.
Historical evidence never rewinds present execution authority.

Paging makes exact, non-resident groups available for explicit demand expansion.
A reference is a locator, not a lasting permission. Refresh reuses the stored
task and reconstructs current Recall/W without another prompt. Neither implies
a background reasoning loop, model-directed paging or constant cost as a Case
grows. [Memory and working-state contracts](docs/recall.md).

## One source frontier, several consumers

Sources retain logical identity, exact revision/backing, declared roles and
current Case applicability. Acquisition and interpretation are separate:

```text
one exact source revision
  ├── knowledge → documentary units / claims → D / Recall
  ├── governance candidate → validation → publication → binding → EffectivePolicy
  └── operational relation → admitted Resource / world identity
```

**Policy may govern; knowledge may inform. Source role ≠ content route ≠ authority.**

Bounded mixed-source routing recognizes explicit policy regions in supported
Markdown and text-PDF profiles while preserving surrounding documentary content.
One clause can feed both consumers with the same provenance. A governance role,
“must” in prose, a filename or a model classification does not publish Policy.

Current deterministic knowledge profiles cover structured text/Markdown, JSON,
text-bearing PDF, admitted filesystem trees and SQLite schema/metadata.
Exact historical backing is required; unavailable bytes stay unavailable.
This is not arbitrary web crawling, Office interpretation, OCR or continuous
synchronization. [Source bootstrap](docs/case-source-bootstrap.md) ·
[Knowledge](docs/source-grounded-knowledge.md) ·
[Mixed-source governance](docs/reference/governance.md#mixed-source-explicit-region-routing).

## Security without model compliance

**Model compliance is not a security prerequisite.**

Model/provider computation and source content are untrusted with respect to
Case authority. Policy rendered to a model is information, not enforcement.
A model may follow an injected instruction and request a prohibited action;
YAI's current authorization path must still deny it or require review.

The bounded mediated surface includes Resource reads as well as writes and
effects: exact filesystem targets, named process execution, SQLite queries,
HTTP endpoints, MCP operations and immutable content access. Protected native
filesystem/process observations are qualified before touching host state;
later PREPARE, dispatch fences and reconciliation keep their distinct roles.
Cross-Case identity substitution and stale authority do not grant access.

| Security ring | Owner / current claim |
|---|---|
| Semantic truth, visibility and authority | YAI; bounded qualified contracts. |
| Case-bound Resource/effect mediation | YAI; current admission, exact bindings and controlled adapters. |
| Computational isolation | Provider/runtime; not supplied by Case policy alone. |
| Host and infrastructure isolation | Platform/kernel/filesystem/network/secrets; independently required. |

YAI does **not** contain ambient filesystem, network or credential access that
a model process has outside YAI mediation. It does not claim universal
prompt-injection immunity, complete sandboxing or automatic declassification.
Credentials and privileged handles belong outside model-visible state where
brokered adapters can exercise them.
[Security contract](docs/reference/governance.md#case-bound-model-security) ·
[Reporting vulnerabilities](SECURITY.md).

## YAI + YVEX

| Owner | Responsibility |
|---|---|
| **YAI** | Case continuity, D/H/S, Recall, W, authority, admission, Resources, Workflow and provenance. |
| **YVEX** | Exact model/composition/runtime realization; model-native state, physical residency and checkpoint mechanics. StateProfile and experiential State Read/Update remain research targets. |
| **Model** | Learned computation. Outputs and future state updates do not create Case authority. |

YAI does not require YVEX: current execution uses a generic OpenAI-compatible
provider contract. It does not load models or administer the inference runtime.

Future **E** denotes experiential computational state, not another semantic
memory owner or necessarily KV. **L** denotes unfinished deliberation, distinct
from reusable E. YVEX's N.B1 slow-update dual-stream realization is producer-owned
research—not implemented YAI behavior. Public W→E, StateProfile transport and
cross-model latent portability are not implemented.
[Semantic/computational boundary](docs/semantic-state-execution-target.md).

## Product surfaces

| Surface | What runs today |
|---|---|
| **YAI Studio** | Native Tauri Case Workbench with bounded in-process live local Case consumption through `yai-application`, real owner-backed projections, generation invalidation and a transient desktop PTY. Browser mode supports explicit fixtures but cannot host live YAI. No resident Local Host, remote transport, mutation-complete API or YVEX management is qualified. |
| **REPLAI Case workbench** | Native terminal interaction for real Cases: guided setup, conversations, capability work, review, Workflow and exact inspection. REPLAI owns reusable editor/terminal mechanics; YAI owns semantics. |
| **`./yai` CLI** | Administration, automation and exact source, policy, resource, history and semantic-state inspection. The CLI is a frontend, not the application API itself. |
| **Typed application boundary** | Existing Rust/controller operations beneath presentation. Not a complete stable public SDK or generated interface package. |

Studio and the native CLI belong in this repository. A future generic interfaces
toolchain is not their runtime dependency.
[Studio preview and build](studio/README.md) ·
[Studio product architecture](docs/studio.md#yai-product-topology) ·
[REPLAI integration](docs/replai-terminal.md) ·
[Application/client boundary](docs/architecture.md).

## Quick start

Use a Linux development host with Rust/Cargo, a C toolchain, GNU Make,
Python 3 and SQLite development libraries. Model weights are not included.

```sh
git clone https://github.com/yailabs/yai.git
cd yai
make build-rust
./yai init
./yai open demo
```

Use a private persistent YAI home; `YAI_HOME` selects an alternate location.
Guided setup asks for explicit consent to create the Tenant, Case and
Participants. Participant admission alone grants no Resource or provider access.

Inside the terminal workbench:

```text
/connect
```

Supply an already reachable public provider endpoint and review the exact model
before approving. Connection probes perform real inference. Use credential
references, never secret tokens in ordinary input or URLs. Then submit a Turn;
`/details` exposes execution lineage. `/work TASK` requests bounded capability
work; `/review` handles eligible human review. `/retry` preserves delivery
safety, and `/cancel` does not promise to abort an already delivered request.

Resources and effects need explicit bindings and ready policy. Follow the
[cumulative operator runbook](docs/zero-to-current.md) for the complete governed
free-work/Workflow lifecycle, provider requirements and recovery.
Studio starts independently through its [preview instructions](studio/README.md).

## Engineering and validation

Committed Transitions are canonical; derived failure cannot rewrite Case truth.
Executable source and tests outrank documentation claims. New semantic owners
require a real lifecycle or contract, not a new UI noun.

```sh
make check characterization
make test-golden-local
```

Component checks, deterministic local qualification and Golden local establish
different evidence. External YVEX is a separate characterization axis—not a
default implementation gate. Human acceptance and an operator-owned continuity
canary remain separate from automated results.
[Validation rules](docs/test-cases.md) · [Contributing](CONTRIBUTING.md).

Current nonclaims include general memory sufficiency, all-source understanding,
autonomous continuous thinking, universal runtime isolation and production
readiness. Exact limitations and research progression belong in the
[Roadmap](ROADMAP.md).

## Documentation

- [Documentation index](docs/index.md) — navigation and authority map.
- [Architecture](docs/architecture.md) · [Constitution](docs/constitution.md) — executable boundaries and durable invariants.
- [Governance](docs/reference/governance.md) · [Provider contracts](docs/provider-governance.md) — authority and execution.
- [Recall / working state](docs/recall.md) · [Semantic-state target](docs/semantic-state-execution-target.md) — implemented memory and future computation.
- [ZERO-TO-CURRENT](docs/zero-to-current.md) — complete operator acceptance.
- [Roadmap](ROADMAP.md) — current maturity and selected engineering work.

## License

YAI is [source-available](LICENSE.md) for technical evaluation and review,
not an OSI-approved open-source offering by default. See
[legal terms](docs/legal.md), [third-party notices](LICENSE.md#third-party-notices)
and the [contribution policy](CONTRIBUTING.md).
[Brand assets](docs/assets/brand/README.md) retain their supplied identity.
