# I06 executable evidence

All runs below used baseline HEAD `e9002b7a452856a7b67eb602d66360f83330ec2d`
on the same master worktree. Changed source was not yet committed. Each block is
an independent run; excerpts from different runs are not one causal transcript.
Local fixture evidence is not external YVEX qualification.

## Proof/mode conclusions

| Proof | Mode | Observation |
|---|---|---|
| Unit/component and bounded fast gates | no_provider | PASS |
| Canonical intent, security, atomicity, replay | no_provider | PASS; second append failure rolls back the Turn |
| Shared adapter and cognitive composition | loopback_fixture | PASS; actual HTTP, unchanged I03/I04 seam |
| Typed application host and process recovery | loopback_fixture | PASS; native audio/repeated images and explicit prerequisite, no auxiliary redispatch |
| REPLAI product PTY | loopback_fixture | PASS; cognitive order differs from provider order; pinned and uncertain retry proofs |
| Independent operator replay | loopback_fixture | PASS through ./yai and a separate reopened PTY; not a human-acceptance claim |
| External YVEX | external_yvex | NOT RUN / DEPLOYMENT_LIMITATION; no configured endpoint/model or external request |

[HOST-RECOVERY-RESULT.json](HOST-RECOVERY-RESULT.json) is the parsed, exact typed
result emitted by the fresh child process in host-02. It retains original and
derived source closure, canonical intent, exact binding/target/qualification,
executed and validation plans, lanes and ProviderResults.
[PTY-EXECUTION.json](PTY-EXECUTION.json) retains unmodified selected observations
from `yai-r4-i5sthy7f`, including independent canonical CLI inspection and /retry.
[OPERATOR-REPLAY.json](OPERATOR-REPLAY.json) records every separate operator
command, cwd, environment, exit and exact PTY output/input. Its run used a fresh
home, the same Case after restart and one ProviderResult, then stopped only its
fixture and removed only its disposable state.

## Baseline-host positive control

Run ID: `i06-baseline-host`; order: 1 within this independent run.
Cwd: `/home/mothx/computer-science/projects/YAI/yai`.
Pre-state: Clean baseline implementation before source edits; isolated fixture Homes; loops use HTTP, not YVEX.
Provider mode: `loopback_fixture`. Exit: `0`.

```sh
CARGO_NET_OFFLINE=true make smoke-conversation-interaction-host smoke-cognitive-execution-composition
```

Bounded unedited stdout/stderr excerpt:

```text
i04_composed: auxiliary_dispatch_once=true restart_resume=true primary_dispatch=true turn_immutable=true audio_and_image=true
i04_direct: primary_native_audio=true auxiliary_bypassed=true explicit_intent_preserved=true
i04_failure: delivery_indeterminate_blocks_primary=true malformed_blocks_primary=true normalization_blocks_primary=true unresolved_blocks_primary=true derived_duplicates=0
i04_revalidation: binding_evidence_and_qualification_change_replans=true historical_derivation_preserved=true
```

## Fast-01

Run ID: `i06-fast-01`; order: 1 within this independent run.
Cwd: `/home/mothx/computer-science/projects/YAI/yai`.
Pre-state: I06 implementation under development; installed offline dependencies; no provider deployment.
Provider mode: `no_provider`. Exit: `0`.

```sh
CARGO_NET_OFFLINE=true make test-fast
```

Bounded unedited stdout/stderr excerpt:

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.01s
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.02s
{"entry": "test-rust-unit-characterization", "suite": "engine", "proof_class": "unit", "provider_modes": ["no_provider"], "cwd": "/home/mothx/computer-science/projects/YAI/yai", "command": ["/home/mothx/computer-science/projects/YAI/yai/target/debug/deps/yai_core_engine-f8dbfbb014cc1ad9", "--exact", "--include-ignored", "--color=never", "compatibility::tests::corpus_freezes_all_c_legacy_kinds_and_drift", "compatibility::tests::corpus_reads_all_rust_legacy_kinds", "compatibility::tests::corpus_report_classifies_without_collapsing_repeated_ids"], "selected_tests": 3}

running 3 tests
test compatibility::tests::corpus_freezes_all_c_legacy_kinds_and_drift ... ok
test compatibility::tests::corpus_report_classifies_without_collapsing_repeated_ids ... ok
test compatibility::tests::corpus_reads_all_rust_legacy_kinds ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 292 filtered out; finished in 0.00s

{"entry": "test-rust-unit-characterization", "suite": "engine", "exit": 0, "elapsed_seconds": 0.002}
```

## Intent-03 canonical adoption

Run ID: `i06-intent-03`; order: 1 within this independent run.
Cwd: `/home/mothx/computer-science/projects/YAI/yai`.
Pre-state: Fresh temporary canonical store; includes v16 marker reopening, exact idempotence, spoofed actor/foreign scope rejection, forced failure of second append.
Provider mode: `no_provider`. Exit: `0`.

```sh
CARGO_NET_OFFLINE=true CARGO_TARGET_DIR=target cargo test --manifest-path engine/Cargo.toml i06_tests -- --nocapture
```

Bounded unedited stdout/stderr excerpt:

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 7.26s
     Running unittests src/lib.rs (target/debug/deps/yai_core_engine-f8dbfbb014cc1ad9)

running 1 test
i06_intent: atomic_second_append_rollback=true immutable=true spoofed_actor_rejected=true old_turn_readers=true restart_replay=true no_provider=true
test store::lmdb::tests::i06_tests::conversation_intent_atomic_adoption_security_replay_and_compatibility ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 294 filtered out; finished in 0.01s
```

## Host-02 typed/recovery

Run ID: `i06-host-02`; order: 1 within this independent run.
Cwd: `/home/mothx/computer-science/projects/YAI/yai`.
Pre-state: Fresh isolated YAI_HOME; six misleadingly named deterministic peers; semantic evidence explicitly fixture-attested, mechanical shapes actually probed. Child process receives only YAI_HOME and exact Turn ID.
Provider mode: `loopback_fixture`. Exit: `0`.

```sh
CARGO_NET_OFFLINE=true CARGO_TARGET_DIR=target cargo test --manifest-path cmd/yai/Cargo.toml command_adapters::conversation_controller::tests::i06_typed_host_native_composed_restart_and_fail_closed -- --ignored --exact --nocapture --test-threads=1
```

Bounded unedited stdout/stderr excerpt:

```text

i06_host: audio_and_repeated_images_native=true no_mime_inference=true explicit_composition=true process_restart_reuses_auxiliary=true retry_same_turn_and_intent=true failed_prerequisite_blocks_primary=true uncertain_delivery_no_cross_target=true
ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 40 filtered out; finished in 1.05s
```

## PTY-05 product

Run ID: `i06-pty-05`; order: 1 within this independent run.
Cwd: `/home/mothx/computer-science/projects/YAI/yai`.
Pre-state: Existing pinned terminal-test venv; fresh Home /tmp/yai-r4-i5sthy7f/home. Real ./yai, PTY and HTTP; provider barrier withheld until independent Turn+intent inspection.
Provider mode: `loopback_fixture`. Exit: `0`.

```sh
PATH="$PWD/build/r4-venv/bin:$PATH" CARGO_NET_OFFLINE=true make smoke-replai-terminal
```

Bounded unedited stdout/stderr excerpt:

```text
r4_terminal: canonical_editing_unchanged=true commit_before_provider=true success_failure_turn_retained=true exact_termios=true bounded_fds=true native_replai=true
i06_pty: cognitive_arbitration=true provider_order_bypassed=true pinned=true durable_intent=true retry_no_redispatch=true indeterminate_no_cross_target=true
evidence: /tmp/yai-r4-i5sthy7f
```

## Release-02 positive publication union

Run ID: `i06-release-02`; order: 1 within this independent run.
Cwd: `/home/mothx/computer-science/projects/YAI/yai`.
Pre-state: I06 source with v16 physical marker migration fixed; no live provider variables; Linux local IPC/HTTP/PTY admitted.
Provider mode: `loopback_fixture`. Exit: `0`.

```sh
PATH="$PWD/build/r4-venv/bin:$PATH" CARGO_NET_OFFLINE=true make check characterization
```

Bounded unedited stdout/stderr excerpt:

```text

running 3 tests
test compatibility::tests::corpus_freezes_all_c_legacy_kinds_and_drift ... ok
test compatibility::tests::corpus_report_classifies_without_collapsing_repeated_ids ... ok
test compatibility::tests::corpus_reads_all_rust_legacy_kinds ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 292 filtered out; finished in 0.00s

{"entry": "test-rust-unit-characterization", "suite": "engine", "exit": 0, "elapsed_seconds": 0.002}
make: Nothing to be done for 'characterization'.
```

## External-01 dependency limitation

Run ID: `i06-external-01`; order: 1 within this independent run.
Cwd: `/home/mothx/computer-science/projects/YAI/yai`.
Pre-state: YAI_EXTERNAL_PROVIDER_BASE_URL/MODEL and legacy YVEX_BASE_URL/MODEL absent. No deployment launched or repository accessed.
Provider mode: `external_yvex`. Exit: `2`.

```sh
CARGO_NET_OFFLINE=true make test-external-yvex
```

Bounded unedited stdout/stderr excerpt:

```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.03s
{"validation_entry": {"id": "external-yvex", "kind": "make", "path": "tests/integration/yvex/qualification_yvex_provider.sh", "selector": "qualification-yvex-provider", "proof_class": "external", "evidence_posture": "qualification", "provider_mode": "external_yvex", "network": "external", "mutation": "temporary_case_and_provider_inference", "cadence": "external", "entrypoint": "qualification-yvex-provider", "reachability": "product", "property": "Exact exposed target text invocation; not I03/I04 typed media or model-quality qualification"}}
run_id: yvex-black-box-20260907T115728Z-67
yai_sha: e9002b7a452856a7b67eb602d66360f83330ec2d
provider_mode: external_yvex
proof_class: external
evidence_posture: qualification
exercised_shape: text_chat_completion
typed_media_interlock_qualification: not_exercised
qualification_mode: black_box_openai_compatible_provider
yvex_repository_accessed: false
yvex_cli_used: false
external_request_attempted: false
provider_invocation_attempted: false
yvex_external_qualification_state: blocked_external_dependency
finding_class: DEPLOYMENT_LIMITATION
reason: operator endpoint and exact exposed model are required; no default target or fixture fallback
make: *** [Makefile:770: qualification-yvex-provider] Error 3
```

## Real failures found and resolved

1. The initial sandbox disallowed the local fixture listener (Operation not
permitted). The same baseline controls passed with explicit loopback permission.
This was test-environment availability, not provider execution failure.

2. The first full publication attempt found a real incomplete v16→v17 metadata
migration. Historical Transition JSON had been admitted but the LMDB marker
allowlist omitted v16. The fix updates only the physical schema guard; a
reopening test proves metadata migration without rewriting any Transition.

```text
yai: record store import failed after journal write remained at build/tmp/new12/daemon-2239933/filesystem/journal.jsonl: unsupported_persisted_schema: meta:canonical_transition_schema expected=yai.transition.v17 actual=yai.transition.v16
make: *** [Makefile:640: smoke-agentless-case-runtime] Error 2
```

3. PTY fixture revisions initially had an escaped target regex, a false
assumption that CaseView exposes the full CaseState, a literal backslash-r
instead of Enter, and a reset of the byte transcript that broke the retained
terminal restoration proof. Those tests were corrected to use public cognitive/
provider inspection, actual Enter and a suffix cursor without discarding bytes.
Existing R4 editing/restoration/descriptor assertions were not weakened.

Independent run `pty-01`, command `PATH="$PWD/build/r4-venv/bin:$PATH" CARGO_NET_OFFLINE=true make smoke-replai-terminal`, loopback_fixture, source/test revisions before corrections; bounded excerpt:

```text
TypeError: 'NoneType' object is not subscriptable
make: *** [Makefile:834: smoke-replai-terminal] Error 1
```

Independent run `pty-02`, command `PATH="$PWD/build/r4-venv/bin:$PATH" CARGO_NET_OFFLINE=true make smoke-replai-terminal`, loopback_fixture, source/test revisions before corrections; bounded excerpt:

```text
KeyError: 'cognitive_bindings'
make: *** [Makefile:834: smoke-replai-terminal] Error 1
```

Independent run `pty-04`, command `PATH="$PWD/build/r4-venv/bin:$PATH" CARGO_NET_OFFLINE=true make smoke-replai-terminal`, loopback_fixture, source/test revisions before corrections; bounded excerpt:

```text
AssertionError
make: *** [Makefile:839: smoke-replai-terminal] Error 1
```

4. Independent operator inspection exposed pre-existing hard-coded v8 labels in
`store status`. The operator replay preserves that original output and the
corrected v17/v14 reinspection. Rendering now uses the source-owned constants;
no extra schema or storage mutation was introduced.

Recorded failures were followed by passing focused proofs and the complete
publication union, not reclassified as successful provider evidence.

## Final publication run

Run ID: `i06-release-final`; order 1. Cwd:
`/home/mothx/computer-science/projects/YAI/yai`. Pre-state: complete I06 modified
master, corrected v16 marker migration and inspection labels, added same-intent
retry after binding replacement, existing offline dependencies and pinned
terminal venv. No live provider configuration. Exit: **0**.

```sh
PATH="$PWD/build/r4-venv/bin:$PATH" CARGO_NET_OFFLINE=true make check characterization
```

Selected **independent, unedited summary lines from this same run**:

```text
validation_catalog: PASS entries=424 assertion_files=83 rust_tests=336
i06_host: audio_and_repeated_images_native=true no_mime_inference=true explicit_composition=true process_restart_reuses_auxiliary=true retry_same_turn_and_intent=true failed_prerequisite_blocks_primary=true uncertain_delivery_no_cross_target=true
r4_terminal: canonical_editing_unchanged=true commit_before_provider=true success_failure_turn_retained=true exact_termios=true bounded_fds=true native_replai=true
i06_pty: cognitive_arbitration=true provider_order_bypassed=true pinned=true durable_intent=true retry_no_redispatch=true indeterminate_no_cross_target=true
make: Nothing to be done for 'characterization'.
```

Final unedited epilogue:

```text

running 3 tests
test compatibility::tests::corpus_freezes_all_c_legacy_kinds_and_drift ... ok
test compatibility::tests::corpus_report_classifies_without_collapsing_repeated_ids ... ok
test compatibility::tests::corpus_reads_all_rust_legacy_kinds ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 292 filtered out; finished in 0.00s

{"entry": "test-rust-unit-characterization", "suite": "engine", "exit": 0, "elapsed_seconds": 0.002}
make: Nothing to be done for 'characterization'.
```

The release union includes no-provider unit/component/security/recovery, actual
loopback contract/product/recovery, I01–I06, H19/W20, retained bounded endurance,
registry, docs/layout and exact test reachability. The catalog count is
coverage metadata, not a claim of 336 external-model executions. Characterization
shared leaves run once in the combined Make invocation. No timing guarantee.

Clippy (order 1 in each independent `i06-clippy-engine-final` /
`i06-clippy-cli-final` run, same cwd/pre-state, no_provider, exit 0):

```sh
CARGO_NET_OFFLINE=true CARGO_TARGET_DIR=target cargo clippy --manifest-path engine/Cargo.toml --workspace --all-targets
CARGO_NET_OFFLINE=true CARGO_TARGET_DIR=target cargo clippy --manifest-path cmd/yai/Cargo.toml --all-targets
```

Existing warnings remain (12 engine, 13 CLI), outside the new execution/intent
logic; this is not a warning-free claim. New needless borrows and large enum
storage introduced during factoring were corrected. Fmt, docs/layout, TSV audit,
manual Bash syntax and diff checks complete before staged publication inspection.


## Final guard run

Run ID: `i06-final-guards`; command order as recorded below, same repository cwd,
complete modified I06 source, no_provider. Empty output is retained as empty,
not reconstructed as a fabricated success message.

```json
[
  {
    "command": "cargo fmt --manifest-path engine/Cargo.toml --all -- --check",
    "exit": 0,
    "output": ""
  },
  {
    "command": "cargo fmt --manifest-path cmd/yai/Cargo.toml --all -- --check",
    "exit": 0,
    "output": ""
  },
  {
    "command": "git diff --check",
    "exit": 0,
    "output": ""
  },
  {
    "command": "git diff --exit-code -- README.md",
    "exit": 0,
    "output": ""
  },
  {
    "command": "awk '/^```sh$/{capture=1;next} /^```$/{capture=0} capture' refoundation/foundation-recovery/interlock-06/MANUAL-ACCEPTANCE.md | bash -n",
    "exit": 0,
    "output": ""
  }
]
```

The separate final docs/layout/catalog audit (same cwd, no_provider, exit 0)
printed:

```text
check-no-old-roots: ok
check-required-layout: ok
check-source-placement: ok
check-source-surface-clean: ok
doc_root_canon: ok
check-doc-canonical-location: ok
check-doc-required-files: ok
check-doc-links: ok (31 files)
check-repository-identity: ok
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.04s
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.07s
coverage_parity: PASS old_smoke_leaves=71 current_make_leaves=78 combined_duplicate_make_leaves=0
validation_catalog: PASS entries=424 assertion_files=83 rust_tests=336
```
