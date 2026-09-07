# YAI R5 executable evidence

Canonical cwd: `/home/mothx/computer-science/projects/YAI/yai` on Exon via LAN SSH.
Run baseline: `fec2f17b0e71e6e12e8f40a37c5f44d874f4b7d5`. R5 changes only obsolete
source/docs/guards and test artifact selection; concurrent constitutional docs
are outside this delivery. Raw outputs below are separate identified runs.

## 1. Clean producer and product interaction

Run `yai-r5-clean-9ws6q5q6`: a fresh non-Git temporary source copy of tracked
current files, excluding deleted files and all untracked build state. There is
no vendor directory or `.git`, no previous target or Cargo home. The copy is a
build fixture, not an implementation branch/worktree. All 184 production files
were SHA-256 compared with the canonical source after execution and matched;
[the source hashes](logs/clean-source-hashes.json.gz) retain that identity.

Exact producer environment and commands:

```sh
export CARGO_HOME=/tmp/yai-r5-clean-9ws6q5q6/cargo-home
export CARGO_TARGET_DIR=/tmp/yai-r5-clean-9ws6q5q6/target
cargo fetch --locked --manifest-path /tmp/yai-r5-clean-9ws6q5q6/source/cmd/yai/Cargo.toml
cargo build --locked --offline --manifest-path /tmp/yai-r5-clean-9ws6q5q6/source/cmd/yai/Cargo.toml
PATH="$PWD/build/r4-venv/bin:$PATH" \
YAI_REPLAI_TEST_ARTIFACT=/tmp/yai-r5-clean-9ws6q5q6/target/debug/yai \
python3 tests/characterization/replai-terminal/test_replai_terminal.py
```

All three exited 0. Fresh Cargo acquisition resolves the public exact Git pin;
no adjacent checkout, old linenoise archive or system REPLAI library is used.
The suite inspects the actual new binary with `nm`, observes native REPLAI
Interaction and no linenoise symbols, and rejects the deleted build script or
vendor tree. Cargo metadata verifies the native Git revision and no C binding.
See [fetch](logs/clean-fetch.log.gz), [build](logs/clean-build.log.gz) and
[PTY stdout](logs/clean-pty.log.gz).

The PTY child run is `yai-r4-n_mbzm24`, with a fresh isolated YAI_HOME and an
explicitly authorized Case. The test name retains its R4 lineage; this is a new
execution of the current I06-qualified test against the clean-built artifact.
[Ordered CLI commands, input bytes, canonical state and terminal observations](logs/terminal-evidence.json.gz)
retain real output without joining records from different runs.

Observed properties: Unicode/grapheme editing, non-end cursor/history draft
return, CRLF multiline paste, host command completion, resize/Ctrl-L and editing
interrupt/EOF retain the intended draft and terminal-native cells. Editing does
not mutate canonical Case, Turn or content state. SEND makes a Turn visible
before the fixture releases its downstream response. Success and indeterminate
failure retain it; I06 intent/arbitration and no-redispatch retry remain intact.
All seven terminal closures restore exact captured termios, balance paste mode
and exit 0. Twenty-four editing reopen cycles and twelve submissions retain
exactly five descriptors at the active prompt. NO_COLOR and TERM=dumb contain no
color residue; terminal cells retain the default background.

Bounded unedited stdout:

```text
r4_terminal: canonical_editing_unchanged=true commit_before_provider=true success_failure_turn_retained=true exact_termios=true bounded_fds=true native_replai=true
i06_pty: cognitive_arbitration=true provider_order_bypassed=true pinned=true durable_intent=true retry_no_redispatch=true indeterminate_no_cross_target=true
evidence: /tmp/yai-r4-n_mbzm24
```

## 2. Current publication union

The first `PATH="$PWD/build/r4-venv/bin:$PATH" make check characterization`
stopped at the document guard. The complete current union was then executed as:

```sh
PATH="$PWD/build/r4-venv/bin:$PATH" make -k check characterization
```

Actual exit: 2. The `-k` flag preserves execution of independent leaves after a
failure; it does not hide or convert the failing gate. The sole Make failure is
`check-docs`. Its guard still requires the exact old constitutional sentence,
while concurrently edited `docs/constitution.md` changes that sentence to durable
semantic and operational state. Neither file was changed by R5. No bypass,
weakened assertion or rollback was introduced. This is an unrelated concurrent
document/guard mismatch, not an editor or cognitive regression.

All reported Rust partitions exited 0; CLI, canonical multipart/controller,
provider, I01–I06, recovery and bounded endurance leaves completed. The current
PTY gate also passed through the ordinary canonical launcher. The complete
[unedited union log](logs/publication.log.gz) and
[machine result summary](run-summary.json) retain actual results. This is not a
repository-wide green claim or a completed cross-consumer R5 claim.

Bounded unedited stdout/stderr from this run:

```text
check-doc-root-canon: constitutional primitive missing
make: *** [Makefile:239: check-docs] Error 1
r4_terminal: canonical_editing_unchanged=true commit_before_provider=true success_failure_turn_retained=true exact_termios=true bounded_fds=true native_replai=true
i06_pty: cognitive_arbitration=true provider_order_bypassed=true pinned=true durable_intent=true retry_no_redispatch=true indeterminate_no_cross_target=true
make: Target 'check' not remade because of errors.
make: Nothing to be done for 'characterization'.
```


## 3. Exact staged source guard control

Run `yai-r5-index-check-xx_zhnx3`: `git checkout-index --all --prefix` exported
only the staged commit contents into a fresh non-Git temporary fixture. No
implementation branch/worktree was created. Concurrent unstaged constitutional
and architecture edits were excluded; the R5 hunks in shared documents were
staged individually.

In `/tmp/yai-r5-index-check-xx_zhnx3`:

```sh
make check-docs check-layout check-foundation-freeze
```

Exit 0. This isolates the failed current-worktree guard causally: the R5 commit
contents pass that same guard without altering the other delivery. The executable
regression controls above used unchanged production source; this additional run
qualifies the precise document/layout contents selected for publication.
[Raw guard output](logs/staged-docs.log.gz):

```text
doc_root_canon: ok
check-doc-canonical-location: ok
check-doc-required-files: ok
check-doc-links: ok (31 files)
check-repository-identity: ok
check-no-old-roots: ok
check-required-layout: ok
check-source-placement: ok
check-source-surface-clean: ok
check-foundation-freeze: ok
```
