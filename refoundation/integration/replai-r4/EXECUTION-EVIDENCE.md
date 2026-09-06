# R4 executable evidence

All canonical work and the publication union ran over LAN SSH on Exon at
`/home/mothx/computer-science/projects/YAI/yai`. Baseline master was
`d3ca2cac5f6575241839c65083d14336e5c1fedc`; the worktree contained only this
migration. No branch reset or consumer-library mutation occurred. Source
fingerprints and observed results are in [run-summary.json](run-summary.json).

## Execution order and material pre-state

1. **Independent acquisition/build**, run `yai-r4-isolation-n93bpa_a`.
   A temporary non-Git source copy excluded `.git`, build/target directories and
   the entire `vendor` directory. It is a build fixture, not an alternate
   implementation worktree. A new Cargo home fetched the public exact Git pin.
   Initial online build exited 0. Cargo fetch then populated test metadata;
   final current Rust source was copied into this isolated source fixture and
   the explicit offline build below exited 0. No adjacent REPLAI source is used.

   ```sh
   CARGO_HOME=/tmp/yai-r4-isolation-n93bpa_a/cargo-home \
   CARGO_TARGET_DIR=/tmp/yai-r4-isolation-n93bpa_a/target \
   cargo build --locked --offline \
     --manifest-path /tmp/yai-r4-isolation-n93bpa_a/source/cmd/yai/Cargo.toml
   ```

   [Acquisition log](logs/isolation-acquisition.log.gz),
   [final build log](logs/isolation-final.log.gz), and
   [resolved source](isolation.json) retain actual output/path identity. The
   initial source build was online; this final offline reuse is not described
   as a second cold fetch. The final build ran alongside publication tests but
   used its own target directory and no shared test state.

2. **CLI lint comparison**, current worktree, existing dependencies available.

   ```sh
   cargo clippy --manifest-path cmd/yai/Cargo.toml --all-targets --locked
   ```

   Exit 0; [actual log](logs/clippy-cli.log.gz). The 13 `(diagnostic, location)`
   pairs are exactly equal to the committed baseline's
   `refoundation/validation/test-topology-0/logs/clippy-cli.log.gz`.
   [Executable comparison](clippy-comparison.json) records both arrays and true
   equality. A separate exploratory `-D warnings` run failed on baseline lint
   debt; warnings-as-errors cleanliness is explicitly not claimed. An introduced
   helper-placement lint was corrected before this retained run.

3. **Final authoritative publication union**, run `publication-final`, with the
   pinned pyte/wcwidth requirements installed in `build/r4-venv` and dependencies
   already fetched. Exact command:

   ```sh
   cargo fmt --manifest-path cmd/yai/Cargo.toml --check
   git diff --check
   PATH="$PWD/build/r4-venv/bin:$PATH" make check characterization
   ```

   Each returned 0. [Unedited combined stdout/stderr](logs/publication.log.gz)
   retains each selected leaf and Rust partition's command/result. This union
   includes the existing CLI, Case, multipart, controller, provider governance,
   typed realization, cognitive composition, recovery and bounded endurance
   surface plus the new real-product terminal gate. It is not an aggregate
   unit-test-count claim. Earlier development runs are not spliced into it.

4. **Product PTY/canonical proof**, child run `yai-r4-51oz2j5q`, executed by
   `smoke-replai-terminal` in the final union. Its actual command is:

   ```sh
   python3 tests/characterization/replai-terminal/test_replai_terminal.py
   ```

   Exit 0. Fresh `YAI_HOME` is `/tmp/yai-r4-51oz2j5q/home`; preparation creates
   `tenant:r4`, `case:r4`, `participant:r4`, authenticated principal linkage,
   role and model-context admission through `./yai`. There are no initial
   Turns or imported conversation bytes. The actual `target/debug/yai prompt`
   process runs with real PTY stdin/stdout and a controlling terminal. Ordered
   [input bytes, CLI commands/output, termios, canonical identities and PTY byte
   captures](logs/terminal-evidence.json.gz) are retained from this one run.
   Records carry explicit monotonically increasing execution order.

## Direct observations

- Initial Case generation 4 and empty Turn/content inventory survive grapheme
  editing, Ctrl-C and EOF unchanged. `café界`, Left, Backspace, `X` displays
  `cafX界`, cursor column 18 (14-cell prompt plus four edited cells).
- History Up/Down restores `draft` with a non-end cursor at column 18; Ctrl-L
  and resize to 80×20 preserve it. Unique Tab selects `/thread status` through
  the host vocabulary. Ambiguous completion writes a notice through REPLAI
  and restores `/thread ` with cursor column 21.
- Bracketed `é\r\n界` remains one draft; no canonical/content mutation until
  Enter. It commits one ordered part with exact bytes `c3a90ae7958c`, six-byte
  UTF-8 text `é\n界`, base generation 5 and next Case generation 6. The test
  checks exact SHA-256, participant, Case, ordinal and original provenance
  against canonical Turn inspection, not echoed terminal text. A subsequent
  exact comparison of the retained prompt thread identity and that same Turn
  also returned equality; both values are retained in run-summary.json.
- With a bound loopback provider, the test holds the real non-synthetic request
  before releasing its response. Canonical Turns are already visible with
  correct pre-commit base generation. Success prints the fixture's actual
  result and `completed`. Dropped delivery prints `delivery_indeterminate`.
  Real Ctrl-C after dispatch also observed `delivery_indeterminate` from an
  interrupted read. All preserve the same committed Turn; exactly one real
  request occurs per scenario, with no automatic replay.
- While provider output owns the terminal, termios equals the captured original.
  Every clean exit also restores that exact value. Paste enable/disable counts
  are respectively 2/2, 1/1, 1/1, 44/44, 2/2, 2/2 and 2/2 across the seven
  child lifecycles. All child exit statuses are 0.
- Twenty-four editing interrupt/reopen cycles keep `/proc/PID/fd` exactly 5;
  twelve additional submissions also return to 5. Provider success, failure
  and execution interrupt reopen a usable draft with the original FD count.
- Accent palette 81 is observed. NO_COLOR (even empty) and TERM=dumb produce
  no SGR. The independent terminal oracle sees default-background cells and
  no alternate screen. Multiline continuation is `... ` with correct CJK
  cursor geometry.
- Non-TTY output and interactive direct-provider override fail with exit 2,
  unchanged termios and canonical state, and no bracketed-paste enable.
- Cargo metadata resolves the exact native Git revision; the product's
  demangled symbols include `replai::terminal::Interaction` and no linenoise.
  No C binding dependency or linenoise build script exists.

## Bounded unedited stdout excerpts

The following are selected complete lines from the final union, followed by
its contiguous final eight lines. Complete ordering remains in the raw log.

```text
i04_composed: auxiliary_dispatch_once=true restart_resume=true primary_dispatch=true turn_immutable=true audio_and_image=true
i04_direct: primary_native_audio=true auxiliary_bypassed=true explicit_intent_preserved=true
i04_failure: delivery_indeterminate_blocks_primary=true malformed_blocks_primary=true normalization_blocks_primary=true unresolved_blocks_primary=true derived_duplicates=0
i04_revalidation: binding_evidence_and_qualification_change_replans=true historical_derivation_preserved=true
r4_terminal: canonical_editing_unchanged=true commit_before_provider=true success_failure_turn_retained=true exact_termios=true bounded_fds=true native_replai=true
evidence: /tmp/yai-r4-51oz2j5q

test compatibility::tests::corpus_freezes_all_c_legacy_kinds_and_drift ... ok
test compatibility::tests::corpus_report_classifies_without_collapsing_repeated_ids ... ok
test compatibility::tests::corpus_reads_all_rust_legacy_kinds ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 287 filtered out; finished in 0.00s

{"entry": "test-rust-unit-characterization", "suite": "engine", "exit": 0, "elapsed_seconds": 0.002}
make: Nothing to be done for 'characterization'.
```

The independent-build and lint raw logs remain separate runs; their outputs
are not represented as part of the PTY causal sequence. Remote publication
identity and clean-worktree verification are recorded in the final handoff.

## Final documentation/index preparation

After adding this dossier and Roadmap entry, in the same canonical worktree:

```sh
make check-docs check-layout
cargo fmt --manifest-path cmd/yai/Cargo.toml --check
git diff --check
```

All returned 0. [Unedited guard output](logs/final-guards.log.gz) is retained
separately; no product source changed after the final publication union.
