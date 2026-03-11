# Governance Ingestion

`governance/ingestion/` is the canonical governance authoring supply chain.

Stages:

- `sources/`: source intake manifests.
- `parsed/`: deterministic parsed governance facts.
- `normalized/`: normalized governance IR.
- `candidates/`: candidate enterprise governance objects.
- `review/`: governance review lifecycle state.
- `templates/`: source authoring templates.
- `examples/`: non-authoritative example payloads.

Canonical tooling entrypoints:

- `tools/bin/yai-govern`
- `tools/bin/yai-govern-ingest-parse`
- `tools/bin/yai-govern-ingest-normalize`
- `tools/bin/yai-govern-ingest-build-candidate`
- `tools/bin/yai-govern-ingest-validate`
- `tools/bin/yai-govern-ingest-inspect`

Ingestion artifacts are controlled inputs to canonical governance domains
(`manifests`, `schema`, `registry`, `compliance`, `overlays`) and are not
runtime source-of-truth by themselves.
