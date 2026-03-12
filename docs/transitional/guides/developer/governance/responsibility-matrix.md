# Cross-Repo Responsibility Matrix

| repo | primary role | allowed dependencies | forbidden dependencies | authority relationship | compatibility relationship | artifact/export relationship | notes |
|---|---|---|---|---|---|---|---|
| `governance` | normative source of truth | none | all structural deps | authoritative for governance semantics | publishes compatibility surfaces consumed by others | may publish exported governance snapshots/manifests | autonomous repo |
| `yai` | integration/runtime authority | `governance` tight link allowed | reciprocal/multi-satellite pins | consumes governance as integration baseline | validates integrated behavior against governance | may vendor/export integration artifacts | only repo allowed tight governance link |
| `sdk` | public programmatic surface consumer | no structural cross-repo pin | pinning `governance`, pinning `cli` | non-authoritative for governance | declares supported compatibility | may consume exported/generated governance artifacts (optional) | must not be model/registry/governance-live coupled |
| `cli` | operator surface consumer | no structural cross-repo pin | pinning `governance`, pinning `sdk` | non-authoritative for foundation/sdk | declares supported compatibility | may run verify-only compatibility checks | verify hooks are not repo dependency |
