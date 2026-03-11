# Transitional Migration Spine (A19)

`transitional/` is a controlled, non-canonical migration area.

Purpose:

- isolate temporary migration artifacts from canonical roots
- keep old-to-new cutover evidence visible and reviewable
- track remaining compatibility paths until final removal

Canonical roots remain authoritative:

- `governance/`
- `docs/`
- `include/`
- `lib/`
- `tests/`
- `tools/`

## Structure

- `embedded-law/`: embedded-law migration markers and decommission notes
- `legacy-docs/`: references to non-canonical/historical docs kept for traceability
- `legacy-maps/`: old-to-new path and naming crosswalks
- `migration-markers/`: tranche markers and closure checklists

## Rules

- no new canonical feature development under `transitional/`
- every file must have an owner tranche and a removal target
- prefer links/maps over duplicated content
- shrink this area incrementally as Block B progresses

## Exit Criteria

This area is considered converged when:

- runtime/tooling no longer require transitional compatibility paths
- legacy references in active docs are eliminated or archived
- migration maps are either consumed by final docs or retired
