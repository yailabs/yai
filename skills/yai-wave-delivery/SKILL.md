---
name: yai-wave-delivery
description: Complete and publish a named YAI implementation or hardening wave with owned validation and evidence.
---

# YAI wave delivery

Use when a named implementation or hardening boundary is selected. The task
prompt owns its desired outcome; [ROADMAP](../../ROADMAP.md) owns selection and
maturity. Choose the implementation strategy from live source and tests.

Reconcile HEAD, origin/master, remote master and index/worktree before changing
shared files. Preserve concurrent work and keep a precise commit whitelist.
Update current contracts in their existing [documentation owners](../../docs/index.md),
not a new wave dossier. Substantive Case/product changes update the cumulative
[operator runbook](../../docs/zero-to-current.md); do not reset its canary.

Use the relevant [validation lanes](../../docs/test-cases.md) in proportion to
the change, including Golden local when affected. Retain executable evidence
for implementation/hardening claims: each observed block needs an exact
command, cwd, relevant environment, exit status, bounded unedited output,
produced identities, run ID, order and material pre-state. The existing
`tools/validation/capture_evidence.py` can record it. Product commands and
qualification suites are separate evidence; never fabricate or combine runs
into one causal proof. Keep tests under `tests/`, reusable checks under
`tools/validation/`, and independently needed external observations under
`labs/`; retain raw evidence for non-reproducible observations or unresolved
qualification. Git owns past wave chronology.

If the wave changes an executable capability, also read
[yai-capability-surface](../yai-capability-surface/SKILL.md). An unrelated dirty
file need not stop the wave, but overlapping unpublished architecture must be
reconciled before a claim of completion. Review the full staged diff, commit
only owned changes, push and verify HEAD, origin/master and remote master.
If push is rejected, stop at publication and report it without an automatic
pull, merge, rebase or force push.

Report the SHA pair, tests and observed evidence, exact blockers, publication
result and separate Golden local, external, canary and human postures. Follow
the [test-case doctrine](../../docs/test-cases.md); an aggregate PASS count or
an unavailable external producer is not a substitute for scoped proof. Every
implementation-wave handoff includes `YVEX EXTERNAL FINDINGS`, including a
`NOT_RUN` reason when the external lane was not selected.
