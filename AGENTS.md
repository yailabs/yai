# YAI repository agent contract

Work as a senior engineering agent: infer a safe implementation strategy from
the requested outcome, inspect the evidence needed for that task, and carry
authorized work through validation and publication. Ordinary scoped edits,
archaeology, refactoring and disposable local tests do not need a separate
approval. Ask when a missing choice would materially change scope or authority.
Future task prompts should state **outcome → authority → freedom → invariants →
completion**; they need not prescribe an edit-by-edit plan.

## Repository invariants

- Executable source, tests and observed behavior outrank documentation claims.
  Committed Transition history owns canonical Case history; CaseState is its
  rebuildable current materialization. Graph, memory, retrieval, analytics,
  Projection, ContextFrame and runtime/model state do not acquire that authority.
  Immutable owned content retains its separate exact-backing obligation.
- Model/provider output is candidate material, never operational authority.
  External effects fail closed through the current typed admission chain.
- Add a semantic owner only for an independent lifecycle, canonical resource,
  transition, execution boundary or stable multi-consumer contract. Avoid
  module-per-noun architecture, speculative registries and duplicate C/Rust
  owners. A runtime loop does not own Case continuity. Consult
  [semantic dispositions](docs/reference/semantics.md) for rejected owners.
- Preserve unrelated dirty and concurrent work. Characterize protected behavior
  before destructive redesign or removal; stage only owned paths or hunks.
- Do not edit the root `README.md` unless Francesco explicitly requests that
  exact edit in the current turn. Do not introduce `.claude/` as an instruction
  or compatibility surface.

## Context and completion

[Architecture](docs/architecture.md) records current executable truth;
[ROADMAP](ROADMAP.md) owns live project control; [Constitution](docs/constitution.md)
and [references](docs/index.md) own stable semantics. Read the parts relevant to
the change, not the whole stack for every edit. Architecture must describe what
works, not be made true by weakening runtime behavior.

For a named implementation or hardening wave, use the
[wave-delivery skill](skills/yai-wave-delivery/SKILL.md). When materially changing
an executable capability, also use the
[capability-surface skill](skills/yai-capability-surface/SKILL.md): every such
capability needs an explicit engine, Application, CLI and Studio disposition,
with behavioral product evidence or an exact deferred/internal reason. For a
load-bearing historical redesign or recovery, use the
[legacy-archaeology skill](skills/yai-legacy-archaeology/SKILL.md). Load the
[external-YVEX skill](skills/yai-external-yvex/SKILL.md) only when that
qualification is explicitly selected; YVEX is a black-box provider to YAI.

Completion means affected validation, complete staged-diff review, an isolated
commit, push and verification against remote master. A rejected push is a
publication blocker, not permission to pull, rebase, merge or force-push.
Evidence must come from actual commands and observations, never reconstructed
transcripts. Keep automated, Golden-local, external-provider, continuity-canary
and human-acceptance verdicts distinct; only the operator can grant human
acceptance. Preserve the exact limitations in the final handoff.
