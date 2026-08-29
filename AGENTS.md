# YAI repository agent contract

This file is the canonical repository contract for Codex when developing YAI.
It applies to the entire repository. Claude-specific project settings are not
supported; `.claude/` must not be introduced or treated as a compatibility
surface.

## Evidence and authority

- Executable source, tests, persisted fixtures, and observed behavior outrank
  documentation claims. Reconcile contradictions before changing a
  load-bearing property.
- Committed Transition history is canonical authority. CaseState is its
  rebuildable materialization. Graph, memory, retrieval, analytics, Projection,
  ContextFrame, residency decisions, and provider/runtime state remain derived
  or non-canonical.
- Model and provider output is candidate material, never operational authority.
  External effects fail closed and require the current typed admission chain;
  no text, derived view, or historical compatibility object may bypass it.

## Source ownership

- Do not create module-per-noun architecture.
- Do not add a registry before a demonstrated multi-consumer or external
  contract requires one.
- Do not create a semantic owner without an independent lifecycle, canonical
  state or resource, transition algorithm, execution boundary, or stable
  multi-consumer contract.
- Do not introduce an Agent runtime owner. A Case owns durable continuity; a
  runtime loop may advance it but may not own its history, memory, authority, or
  resources.
- `Space` remains rejected unless a new independent lifecycle and ownership
  boundary is demonstrated that Case cannot represent without semantic
  overload.
- Do not maintain independent C and Rust owners for the same semantic type or
  transition. Cross-language boundaries must be coarse, versioned, and owned in
  one place.

## Legacy archaeology gate

Before materially redesigning or deleting a load-bearing property, inspect
`yai-dev` and its relevant Git history. Determine:

- whether YAI implemented the property before, and in which epoch or commit;
- the strongest executable mechanism and tests that existed;
- its invariants and failure behavior;
- why it was later removed or replaced;
- which property belongs in the current owner and which historical ownership
  must remain rejected.

Recover semantics, contracts, small algorithms, tests, fixtures, validation
rules, and failure behavior when they remain valid. Never copy historical
planes, directory trees, registries, or runtime ownership wholesale. Record the
historical source, recovered property, target owner, and reason for reuse.

No rediscovery without archaeology.

For Foundation Recovery work, `FOUNDATION-RECOVERY-LEDGER.tsv` is navigation,
never authority. Reinspect `yai-dev` source, history, tests, consumers, and
adjacent semantic families in every recovery wave, even when a ledger row says
`refounded_proven`; do not search only the directory named by the current noun.
Repository evidence wins over the ledger, which must be corrected when they
conflict. Adjacent-family evidence may reopen any prior recovery verdict.

## Safe change discipline

- Inspect Git status before editing. Preserve unrelated dirty work and stage
  only an explicit task or wave whitelist.
- Characterize behavior and identify unique protected properties before
  destructive deletion, collapse, or compatibility removal.
- Do not change runtime semantics merely to make implementation claims match
  the Constitution. Architecture documents current executable truth; Roadmap
  owns the remaining gap.
- Validate in proportion to risk and inspect the complete staged diff before
  committing.

## Wave discipline

A wave is not complete when its code merely works locally. Completion requires:

```text
implementation
  -> validation
  -> staged-diff inspection
  -> isolated wave commit
  -> push
  -> origin/master verification
```

- Never include unrelated dirty work in a wave commit.
- Do not begin the next wave from uncommitted or unpublished architectural
  work.
- A versioned wave report records the baseline SHA, intended commit message,
  implementation/test state, and pre-publication closure state. It must not try
  to record the SHA of the commit that contains itself. The post-commit final
  response records the actual final SHA, push result, and equality of `HEAD`,
  `origin/master`, and the remote branch reference.
- Every implementation or hardening wave must retain actual executable
  evidence. Reports must identify the exact command, working directory and
  relevant environment, real exit status, a bounded unedited stdout/stderr
  excerpt, produced identifiers, and the invariant demonstrated. Each retained
  block also records a run ID, execution order, and material pre-state; outputs
  from different runs must not be mixed into one causal proof. Product
  commands and qualification suites are distinct evidence; use both when a
  product surface exists. Never reconstruct, paraphrase as raw output, or
  fabricate a transcript after the fact.
- A failed or rejected push leaves the wave blocked at publication; do not
  declare it complete and do not automatically pull, merge, rebase, or force
  push through an unexpected remote divergence.
