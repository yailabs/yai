# I-003 — Governance

Governance is non-bypassable and continuous.

## Invariant

Authority, policy, enforcement, and consequence handling MUST remain effective across lifecycle phases and failure modes.

## Required properties

- no external effect without authority-evaluated decision
- no execution path that bypasses `core` governance surfaces
- violation attempts are detectable and consequential
- recovery/restart/upgrade must preserve governance guarantees

## Ontology alignment

- `core` is sovereign governance plane
- `exec` is constrained effect plane
- `brain` is governed cognitive plane

Any architecture where `exec` or `brain` can bypass `core` authority is non-conformant.
