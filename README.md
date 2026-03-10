# yai

`yai` is the runtime implementation repository and the primary runtime consumer of platform law.

## What this repository is

- runtime host and internal runtime modules (`core`, `exec`, `data`, `graph`, `knowledge`)
- controlled ingress/dispatch and runtime enforcement realization
- consumer of canonical law from `law`

## What this repository is not

- not the normative source of law
- not the ops bureau for official qualification/collateral material
- not the dataplane/db/query implementation scope for this tranche

## Cross-repo role

- `law`: canonical normative authority
- `yai`: runtime realization
- `ops`: official/catalog/qualification/evidence bureau

## Surface quick map

### Primary runtime-facing surfaces

- `embedded/law/`
- `include/yai/law/`
- `lib/law/`
- `tools/bin/yai-law-*`

### Canonical external source

- sibling `../law` repository

### Bridge / transitional tolerated

- `embedded/law/transitional/domain-family-seed/` (bridge payload only, non-primary, do-not-extend)

### Historical / reference-only

- historical debug/audit reports marked as superseded in `docs/architecture/*debug-report*`

Primary local commands:
- `tools/bin/yai-law-embed-sync`
- `tools/bin/yai-law-compat-check`

## Scope note

Transitional refactor and mapping documents are quarantined under:
- `archive_tmp/`

## Start here

- `docs/README.md`
- `docs/architecture/repository-scope.md`
