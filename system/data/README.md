# data

sys/data/ is the canonical L2 data plane.

## Canonical service

- yai-datad/: executable data service entrypoint (`yai-datad`)

## Canonical module layout

- records/: append flows for runtime/source records
- evidence/: evidence append and evidence summary surfaces
- retention/: retention pruning logic
- archive/: archive rotation logic
- store/: LMDB and DuckDB-backed data store implementations
- internal/: data query surfaces and store/binding internals
- include/yai/data/: canonical public headers

## DR-4 hard-cut status

- active data implementation relocated from runtime/compatibility/lib/data/
- compatibility data subtree no longer hosts primary implementation
- no parallel compatibility-centered data plane remains active
