# AI Grounding Governed Case State Baseline (QG-3)

## Objective

Validate that AI/agent execution contexts are built from governed owner-side
state rather than raw unscoped runtime exhaust.

## Baseline checks

1. Verify grounding context includes operational summary and unified graph summary.
2. Verify context carries adjudication boundary markers.
3. Verify degraded grounding is explicit and reason-coded.
4. Verify distributed peer/transport signals appear only through owner-side
   recomposed summaries.
5. Verify AI output path remains non-sovereign and owner-adjudicated.

## Expected outcomes

- `exec` consumes task-scoped governed grounding context.
- Context is provenance/freshness/validity aware.
- AI assistance remains bounded by workspace authority semantics.

## Anti-drift assertions

- Prompt context != canonical truth authority.
- Peer-local signal != direct grounding truth.
- AI proposal != final adjudication.
