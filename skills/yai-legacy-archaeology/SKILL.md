---
name: yai-legacy-archaeology
description: Recover or redesign a load-bearing YAI property after inspecting its yai-dev source, history and tests.
---

# Legacy archaeology

Use before materially redesigning, deleting or recovering a load-bearing
property. Ordinary small edits do not require this workflow.

Inspect `yai-dev` source and relevant Git history, including adjacent semantic
families and consumers. Establish whether the property existed, its strongest
executable implementation and tests, invariants and failure behavior, and why
it was removed or replaced. A recovery ledger is navigation, never authority;
an old `refounded_proven` verdict does not replace source inspection.

Recover still-valid contracts, bounded algorithms, fixtures or failure rules
into the current owner. Do not copy an old plane, tree, registry, runtime or
ownership model wholesale. Record the historical source, recovered property,
target owner and reason in the affected current contract, test or ROADMAP entry
when project-control state changes. Do not revive a separate archaeology ledger.
Use [semantics](../../docs/reference/semantics.md) and
[architecture](../../docs/architecture.md) to distinguish current ownership
from historical vocabulary.
