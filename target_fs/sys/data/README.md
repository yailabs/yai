# data

`target_fs/sys/data/` is the data service surface.

## Scope

This domain keeps only the canonical data service entry shell.

## What stays here

- `cmd/datad/`: canonical data entrypoint
- this README as data surface contract

## What does not stay here

Archive, evidence, records, retention, store and query/runtime
implementation belong to `target_fs/krt/data/`.
