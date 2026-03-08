# tools/bin (Hard-Cut Wrappers)

`yai` keeps stable command entrypoints here, but canonical implementation is externalized.

- Primary target: `infra/tools/bin/*`
- Extended targets: `infra/tools/release/*`, `infra/tools/bundle/*`

Behavior:
- Wrappers are hard-delegated.
- If canonical target is missing, wrapper exits with `2` and prints missing path.

Runtime wrappers kept in this repo:
- `yai-law-embed-sync`
- `yai-law-compat-check`
- `law-sync` (legacy alias)
- `yai-specs-sync` (deprecated alias)
- `yai-version`
- `yai-bundle`
- `yai-changelog-check`
- `yai-check-pins`
- `yai-docs-trace-check`
- `yai-gate`
- `yai-proof-check`
- `yai-suite`
- `yai-verify`


Notes:
- Wrappers are infra-first.
- `yai-changelog-check` keeps a CI fallback to local validator when `infra` is not checked out by the runner.
- Law consumption model is embedded-surface runtime (`embedded/law`) with no active legacy bridge fallback.
