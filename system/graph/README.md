# graph

sys/graph/ is the canonical L2 graph plane.

## Canonical service

- yai-graphd/: executable graph service entrypoint (`yai-graphd`)

## Canonical module layout

- materialization/: graph projection/materialization flows
- query/: graph query surfaces
- lineage/: lineage reconstruction/exposure surfaces
- summary/: summary facets for activation/policy/authority/episodic/semantic views
- internal/: backend, counters, and graph state internals
- include/yai/graph/: canonical public headers

## DR-3 hard-cut status

- active graph implementation relocated from runtime/compatibility/lib/graph/
- compatibility graph subtree is drained and non-canonical
- no parallel compatibility-centered graph plane is kept active
