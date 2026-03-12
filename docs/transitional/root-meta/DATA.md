# Data Policy

## Scope

This policy governs **data assets committed to this repository**, specifically under `data/`, and the rules for handling any runtime-generated artifacts produced while developing or operating `yai`.

This is a **source repository**, not a data store.

## Canonical Locations

Committed datasets and related metadata live under:

- `data/datasets/`
- Example dataset: `data/datasets/global-stress/v1/`

Repository-local shared/reference data (if any) must be explicitly documented and justified:

- `data/global/` (only if non-sensitive, reviewable, and required for deterministic local tests)

## Data Content Rules

Any data committed to this repository must:

- be **non-sensitive** and suitable for public distribution
- contain **no personal data** (PII), special category data, or biometric identifiers
- contain **no credentials or secrets** (tokens, API keys, private keys, certificates, session material)
- be **reviewable** (human-auditable formats preferred: `.md`, `.json`, `.jsonl`, `.csv`)
- include sufficient **provenance** to evaluate legality and quality

Disallowed examples (non-exhaustive):
- real user conversations, emails, phone numbers, addresses
- scraped datasets with unclear licensing
- production logs, telemetry, workspace state, crash dumps containing identifiers
- proprietary customer material

## Provenance and Licensing Requirements

Each dataset version directory (e.g., `data/datasets/<name>/<version>/`) must include:

- a `README.md` describing:
  - source/origin and license (or explicit “synthetic” generation method)
  - intended use (tests, benchmarks, examples)
  - known limitations and any filtering/redaction steps
- a `manifest.json` (or equivalent) listing:
  - files included
  - expected model/schema/columns (if applicable)
  - checksums where useful for integrity

Opaque binaries are discouraged. If a binary is required (e.g., an `.xlsx` used as a compatibility fixture), it must be:
- minimal in size,
- justified in the dataset README,
- paired with a human-readable description of its contents.

## Runtime Artifacts Policy (Do Not Commit)

The following are **runtime artifacts**, not datasets, and must never be committed:

- logs, traces, events output, debug dumps (`*.log`, `dist/logs/`, etc.)
- generated runtime state (`build/`, `dist/`, `states/`, caches)
- workspace runtime directories (`.yai/` and any per-workspace run dirs)
- database files produced by execution unless explicitly approved as a dataset fixture

All runtime artifacts must live outside git-tracked paths and must be covered by `.gitignore`.

## Review and Enforcement

Data changes are treated as governance-relevant changes. Pull requests that add or modify `data/` should:

- explain provenance and intended usage
- confirm no sensitive data is included
- include a minimal validation step (e.g., schema check or file list sanity)

## License

This policy is distributed under Apache-2.0. See `LICENSE` and `NOTICE`.