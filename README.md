# yai

`yai` is the runtime implementation repository and the primary runtime consumer of platform law.

## What this repository is

- runtime host and internal runtime modules (`core`, `exec`, `brain`)
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

## Transitional dependency note

`yai` now adopts an embedded-law-surface model:
- canonical normativity remains in `law`
- runtime-facing export is consumed from `embedded/law`
- legacy mirror fallback is retired from active runtime and tooling paths

Primary local commands:
- `tools/bin/yai-law-embed-sync`
- `tools/bin/yai-law-compat-check`

## Scope note

Transitional refactor and mapping documents are quarantined under:
- `archive_tmp/`

## Start here

- `docs/README.md`
- `docs/architecture/repository-scope.md`
