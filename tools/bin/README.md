# tools/bin (Hard-Cut Wrappers)

`yai` keeps stable command entrypoints here, but canonical implementation is externalized.

- Primary target: `infra/tools/bin/*`
- Extended targets: `infra/tools/release/*`, `infra/tools/bundle/*`

Behavior:
- Wrappers are hard-delegated.
- If canonical target is missing, wrapper exits with `2` and prints missing path.

Runtime wrappers kept in this repo:
- `yai-governance-compat-check`
- `yai-govern` (governance control/ingestion/authoring CLI)
- `yai-govern-ingest-parse`
- `yai-govern-ingest-normalize`
- `yai-govern-ingest-build-candidate`
- `yai-govern-ingest-validate`
- `yai-govern-ingest-inspect`
- `yai-version`
- `yai-bundle`
- `yai-changelog-check`
- `yai-check-pins`
- `yai-docs-trace-check`
- `yai-gate`
- `yai-proof-check`
- `yai-suite`
- `yai-verify`
- `yai-ws-token` (workspace prompt token helper, cwd-scoped)


Notes:
- Wrappers are infra-first.
- `yai-changelog-check` keeps a CI fallback to local validator when `infra` is not checked out by the runner.
- Governance/runtime source-of-truth is `governance/`; embedded fallback is removed.
- `tools/dev/resolve-governance-root.sh` is the only root resolver used by canonical wrappers.
- `yai-ws-token` prints only `icon + alias` (no `ws:` prefix), and only when cwd is inside a workspace `root_path`.
- `yai-ws-token` resolves workspace from `~/.yai/run/*/manifest.json` by longest matching `root_path` prefix.
- For zsh session-only integration (no permanent prompt override), use `tools/dev/yai-prompt.zsh`.
