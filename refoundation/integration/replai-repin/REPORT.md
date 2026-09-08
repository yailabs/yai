# REPLAI consumer repin — Linux qualification

Baseline master: `35705ff1ee979e12e3c56d15275b43a0212f248a`, tree
`72e641c3c38fba69cb18705b39ed441c108bfaec`. Intended commit:
`build: repin native REPLAI to qualified POSIX editor revision`.
Pre-publication state: dependency, build, real-product and regression gates
passed; awaiting this isolated commit/push and remote equality verification.
The final handoff records the containing commit identity.

## Dependency and scope

The live native Rust dependency is now
`mothx9/replai@6365f84e12865871bf26ecf0d984b48213d81ebc`, Git tree
`2ff0954473fc642f571bf78424278d9019e29338`. Cargo fetched that exact commit into
an initially empty Cargo home; both `cargo metadata` and the actual fetched
checkout agree. No adjacent checkout, system library, C binding or linenoise
is used. Cargo's normal update added only the new producer's `nix` and
`cfg_aliases` resolution, alongside replacing REPLAI's Git source identity.

Only the manifest, lockfile, current terminal document's active pin and two
artifact assertions change. The symbol assertion follows the producer's actual
`replai::interaction::Interaction` module, replacing its old Linux-only module
identity. Public adapter behavior is unchanged. Historical R3/R4/R5 records
continue to name their original qualified revisions. REPLAI is not modified.

Concurrent, unstaged `/connect`/provider source, test and documentation work was
present before this task. It is preserved and excluded from this commit,
including its other hunks in `docs/replai-terminal.md`. The complete canonical
worktree regression union ran with stable tracked file contents (see
[stability](qualification-stability.json)). A second build from the exact
selected Git index additionally qualifies the publication's own source without
that concurrent implementation. The export is a temporary non-Git build
fixture, not another branch or worktree. [All exported blobs match the index](publication-export.json).

## Executed commands and results

Run group: `consumer-repin-linux-20260908`. Canonical cwd:
`/home/mothx/computer-science/projects/YAI/yai` on the physical Linux Exon host,
accessed over LAN SSH. Rust/Cargo 1.93.1; GCC 16.2.1 20260810.
Every command below exited 0. Each PTY run uses its own fresh disposable home,
authorized Case and real loopback provider processes; no operator state is reset.

1. Before changing the dependency, `make build-rust`, then
   `PATH="$PWD/build/r4-venv/bin:$PATH" python3 tests/characterization/replai-terminal/test_replai_terminal.py`.
   Run `yai-r4-bxxc0qmo`: old pin, current concurrent application source.
2. After editing only the manifest pin, with
   `CARGO_HOME=/tmp/yai-consumer-repin/cargo-home` initially empty:
   `cargo update --manifest-path cmd/yai/Cargo.toml -p replai`, then
   `CARGO_TARGET_DIR=/tmp/yai-consumer-repin/target cargo build --locked --manifest-path cmd/yai/Cargo.toml`,
   then `cargo metadata --locked --manifest-path cmd/yai/Cargo.toml --format-version 1`.
   The resolved source is exactly
   `git+https://github.com/mothx9/replai?rev=6365f84e12865871bf26ecf0d984b48213d81ebc#6365f84e12865871bf26ecf0d984b48213d81ebc`.
3. With the same Cargo home and PTY venv PATH,
   `YAI_REPLAI_TEST_ARTIFACT=/tmp/yai-consumer-repin/target/debug/yai python3 tests/characterization/replai-terminal/test_replai_terminal.py`.
   Run `yai-r4-zmy5aqk5`: new pin, otherwise identical production source to step 1.
4. `cargo fetch --locked --manifest-path engine/Cargo.toml`, `make build`,
   then `make check characterization`, with the same Cargo home and venv PATH.
   The publication union completed all 85 classified entries (2 unit,
   14 component, 20 recovery, 48 product, 1 contract), plus current structural
   guards and Rust partitions; none failed. [Machine summary](union-summary.json).
5. `cargo fmt --manifest-path cmd/yai/Cargo.toml --check` and
   `cargo fmt --manifest-path engine/Cargo.toml --all --check` pass.
   With `CARGO_TARGET_DIR=/tmp/yai-consumer-repin/target`, both
   `cargo clippy --locked --manifest-path cmd/yai/Cargo.toml --all-targets` and
   `cargo clippy --locked --manifest-path engine/Cargo.toml --workspace --all-targets`
   exit 0. The CLI reports no warnings; engine Clippy reports 8 warnings in
   unchanged admission/effect/journal/memory/record code. That engine-only
   workspace does not acquire REPLAI. This is not a warning-clean engine claim.
6. After staging only this delivery's four live files/hunks,
   `git checkout-index --all --prefix=/tmp/yai-consumer-repin/publish-source/`.
   With the same Cargo home, a new empty target and no `.git` or old build state:
   `CARGO_TARGET_DIR=/tmp/yai-consumer-repin/publish-target cargo build --locked --offline --manifest-path /tmp/yai-consumer-repin/publish-source/cmd/yai/Cargo.toml`.
   From canonical cwd, venv PATH:
   `YAI_REPLAI_TEST_ARTIFACT=/tmp/yai-consumer-repin/publish-target/debug/yai python3 tests/characterization/replai-terminal/test_replai_terminal.py`.
   Run `yai-r4-kuo01qtz`: actual binary built solely from this publication's
   selected source. `make check-docs check-layout` also passes in that export.

## Observed product boundary

All three real-process PTY runs passed. The before/after runs have identical
recorded draft text, cursor coordinates and FD counts. For example, grapheme
editing produces `cafX界` with cursor column 18; history returns `draft` with
cursor column 18, retained through Ctrl-L and resize. Registry completion
produces `/thread status`; ambiguous completion preserves a non-end cursor.
Paste `é\r\n界` becomes one submitted `é\n界` object, exact bytes `c3a90ae7958c`.

Case generation remains 4 with zero Turns/content changes while initially
editing, interrupting and leaving by EOF. Application SEND adds exactly one
Turn through the existing controller. The provider barrier sees the committed
Turn and execution intent before releasing a result. In run `yai-r4-zmy5aqk5`,
provider success has generation `34 -> 38` while pending and 41 after completion;
failure `43 -> 47 -> 49`; cancellation `51 -> 55 -> 56`. These include existing
provider/controller transitions, not edit mutations. The exact committed Turn
survives all three outcomes. Arbitration, pinned selection, result lineage and
no cross-target retry after indeterminate delivery remain qualified.

Each run observes seven exact termios restorations and balanced bracketed-paste
mode. Twenty-four editing reopen cycles and twelve submissions retain five
process FDs at the active prompt. Ctrl-D on nonempty input deletes; empty Ctrl-D
exits. NO_COLOR and TERM=dumb have no color residue, all terminal cells retain
default background, and no alternate screen is used.

The reported Case layout issue is **not reproduced** in the bounded tested
scenarios on either revision. Identical observed drafts/cursors are not evidence
that every reported Case layout problem is fixed. No layout repair was made.

Unedited stdout from publication-source run `yai-r4-kuo01qtz`:

```text
r4_terminal: canonical_editing_unchanged=true commit_before_provider=true success_failure_turn_retained=true exact_termios=true bounded_fds=true native_replai=true
i06_pty: cognitive_arbitration=true provider_order_bypassed=true pinned=true durable_intent=true retry_no_redispatch=true indeterminate_no_cross_target=true
evidence: /home/mothx/.cache/tmp/yai-r4-kuo01qtz
```

Unedited final union output:

```text
{"entry": "test-rust-unit-characterization", "suite": "engine", "exit": 0, "elapsed_seconds": 0.002}
make: Nothing to be done for 'characterization'.
```

Compressed unedited command logs and ordered real CLI/PTY/Case/provider records
are retained under `logs/`, with separate before, after and publication-source
identities. No records from different runs are combined into a causal proof.

## Limits and remaining externally owned work

AUTOMATED REGRESSION: PASS; actual native consumer build and PTY path qualified.
GOLDEN CASE LOCAL: NOT_AFFECTED; this wave changes no Case/product operation,
and retains the current publication union plus explicit canonical/provider PTYs.
GOLDEN CASE EXTERNAL YVEX: NOT_RUN; no operator-supplied endpoint/model was
provided. YVEX's separate consumer qualification does not stand in for this lane.
YVEX EXTERNAL FINDINGS: NO_ISSUE in the changed dependency boundary; no new
live-provider claim, no brand-specific workaround. CONTINUITY CANARY: NOT_RUN;
operator data untouched. HUMAN GOLDEN CASE: PENDING_OPERATOR.
ZERO-TO-CURRENT RUNBOOK: unchanged by this wave; concurrent edits remain owned
by their delivery. No YAI hosted CI workflow exists in the reconciled tree;
local authoritative qualification is reported rather than invented hosted CI.
No memory checker was run on YAI in this wave; real terminal/FD lifecycle was.
