# Execution evidence

Baseline and pre-state: master `6e332851b5066cbb1da25f816b8db31b74580acb`,
HEAD/origin/master/remote master equal; initial worktree clean. All commands
run from `/home/mothx/computer-science/projects/YAI/yai`. No external endpoint
or model variables are supplied. YVEX is not started, queried or administered.
No credentials are included in the retained evidence.

The migration used the unchanged old gates first with
`CARGO_TARGET_DIR=$PWD/target` exported to prevent incidental nested build
trees. Gate outputs are captured at execution time, not reconstructed.
Preparation failures and topology defects found during migration remain
recorded below; final qualification is recorded separately.

## Runs already observed

| Run ID / order | Exact command | Exit | Elapsed | Invariant |
|---|---|---:|---:|---|
| topology-old-check-01 / 1 | `/usr/bin/time -p make check` | 127 | not run | host lacks /usr/bin/time; no gate result claimed |
| topology-old-check-02 / 2 | `make check` | 0 | 211.86 s | historical publication positive control |
| topology-old-characterization / 3 | `make characterization` | 0 | 241.18 s | historical characterization positive control |
| topology-fast-01 / 4 | `make test-fast` | 0 | 4.14 s | pure/bounded local lane, no provider |
| topology-combined-01 / 5 | `make check characterization` | 0 | 351.11 s | first migrated graph, including mistakenly selected H19 scale; not final topology timing |
| topology-combined-final / 6 | `make check characterization` | 0 | 252.91 s | final publication union, bounded endurance retained, extended scale opt-in |
| topology-contract / 7 | `make test-contract` | 0 | per-partition records | actual HTTP/TLS loopback and parser contracts independently reachable |
| topology-external-absent | `make test-external-yvex` | 2 (script 3) | not a benchmark | no request attempted; DEPLOYMENT_LIMITATION |

Exact outer timing/capture form for timed runs used the zsh builtin `time`,
`TIMEFMT='elapsed_seconds: %E'`, redirected stdout/stderr to the per-run log,
captured `$?` immediately, then printed a bounded tail. No test output is
reclassified as another run's output. Shell gate exit and libtest counts are
different layers of evidence.

## Real failures / corrections

- `/usr/bin/time` was unavailable (exit 127); retried with the installed shell's
  builtin timer. No successful test was invented for the failed launch.
- The first topology audit rejected residual `target/debug/yai` commands in
  daemon procedures; fixed launcher references, preserving assertions.
- Generated Make initially emitted a Rust partition for a Make-owned doctest
  entry, causing a recipe-override warning; narrowed generation to Rust rows.
- Review found the ignored H19 realistic-dimension characterization classified
  under recovery/publication. Moved it to explicit endurance and added compiled
  ignored-test auditing to prevent silent future promotion. Its incidental run
  is not used to claim final publication timing.
- A real failing temporary child in the topology tests exits 7; the selector
  reports class `contract`, returns 7 and does not claim a provider ran. This is
  infrastructure falsification, not external-provider qualification.
- Initial manual setup attempted a Principal link before admitting its
  Participant and correctly failed `principal_participant_link_case_contract_invalid`
  (exit 3). The manual now admits the existing model-executor role before
  linking; the failed run remains in `manual-run.json` and the corrected fresh
  run is retained separately. No runtime semantics were changed to accommodate
  the bad procedure.

The absence test also proves external mode cannot silently become fixture mode.
Live external success remains unavailable, not inferred from local passes.

## Retained artifacts and manual result

Compressed logs under `logs/` contain full captured bytes (gzip without filename
or timestamp metadata). Old/migration/final runs remain distinct. `run-summary.json`
provides machine-readable attribution and counts derived from the final raw
log, not invented test results. Bounded unedited excerpts below are selected
from those exact logs. Gate environment: no external endpoint/model variables;
installed dependencies, Linux local filesystem/processes, deterministic peers.

`manual-run.json` preserves the real initial procedure failure.
`manual-success-run.json` is a separate fresh isolated home/run: Tenant created,
Case generation 1, Participant role generation 2, Principal link generation 3;
unknown Case exits 3, valid Case remains generation 3 with provider null and
execution never_started. Repeated CLI invocations reopen the same store.
Both verified disposable home directories were removed after inspection; their
captured evidence remains. No operator/provider/model estate was removed.

Other commands: both `cargo fmt --manifest-path ... --all -- --check` passed;
`CARGO_TARGET_DIR=$PWD/target cargo clippy --manifest-path engine/Cargo.toml --workspace --all-targets`
and the CLI equivalent passed with existing warnings; `make check-docs check-layout`,
`python3 tests/characterization/cli-product-surface/audit_registry.py --binary ./yai`,
Python compilation, Bash syntax for 67 shell files and two manual documents,
and `git diff --check` passed. Runtime source diff against baseline is empty.

Final catalog review expanded leaf property descriptions and restored the
explicit `product_fixture` reachability of the daemon's fixed C loops. It did
not change IDs, class/provider selections, assertions or cadence. Earlier raw
metadata labels remain unedited. The final audit again reports 414 entries,
330 compiled Rust identities, 71 retained old smoke leaves and zero duplicate
Make leaves. The last three inline Make CLI assertions were checked separately
through `./yai` along with the topology/docs/layout guards.
