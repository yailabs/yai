# I05 execution and failure evidence

All commands below ran in `/home/mothx/computer-science/projects/YAI/yai` on
`master`, baseline `80d9077f24990711f9879b71c317fc2711d2d7fa`, with this I05
worktree delta. The baseline is not a claim that the uncommitted implementation
was already published. Final SHA/equality belongs in the post-commit handoff.

`logs/*.log.gz` are `gzip -n` copies of actual captured stdout/stderr, not
reconstructed transcripts. Each filename is a separate run ID (outer command
order 1); overlapping runs must not be spliced into a causal execution. Product
JSONL records have their own run ID, command order, cwd, isolated YAI_HOME, exact
argv, exit, stdout and stderr. They retain Tenant/Case/Participant, target/model,
generation, binding/evidence, plan/lane, Turn/part, closure, invocation and result
identities as emitted, rather than borrowing identifiers from another run.

No operator YVEX endpoint/model variables were configured. Local fixtures run
only on loopback, own temporary homes/files/processes and stop them afterwards.
`no_provider` tests construct deterministic evidence directly; that is not an
HTTP qualification or semantic model evaluator. Provider suitability recorded
by CLI is expressly `operator_attested`.

## Executed barriers

Environment shorthand: `OFFLINE` means `CARGO_NET_OFFLINE=true`;
`TARGET` means `CARGO_TARGET_DIR="$PWD/target"`; `TERMINAL` means
`PATH="$PWD/build/r4-venv/bin:$PATH"`. These are command-local settings, not
changes to user configuration. The pre-existing R4 venv supplies pyte 0.8.2
and wcwidth 0.8.3. No packages were installed. Logs were redirected into
`/tmp/yai-i05-evidence.4tdHcS/` and then copied into this package.

| Run ID / raw log | Exact command (environment above) | Exit | Property / provider mode / material pre-state |
|---|---|---:|---|
| fast-01 | `OFFLINE make test-fast` | 0 | unit/component/topology; no_provider; new classified tests compiled |
| arbitration-03 | `TARGET OFFLINE cargo test --manifest-path engine/Cargo.toml --lib arbitration -- --nocapture` | 0 | four pure/LMDB unit/recovery proofs; no_provider; fresh isolated stores |
| i05-loopback-02 | `python3 tests/characterization/cognitive-target-arbitration/test_cognitive_target_arbitration.py` | 0 | product/recovery; loopback_fixture; fresh homes and five HTTP peers |
| i03-03 | `TERMINAL OFFLINE make smoke-typed-provider-realization` | 0 | I03 product/recovery; loopback_fixture; corrected diagnostic and independent negative-work fixtures |
| release-02 | `TERMINAL OFFLINE make check characterization` | 0 | complete publication union; no_provider + loopback_fixture; final semantic source and reconciled I03 test |
| clippy-engine-01 | `TARGET OFFLINE cargo clippy --manifest-path engine/Cargo.toml --workspace --all-targets` | 0 | static analysis; no_provider; 12 existing warnings retained |
| clippy-cli-01 | `TARGET OFFLINE cargo clippy --manifest-path cmd/yai/Cargo.toml --all-targets` | 0 | static analysis; no_provider; 13 existing warnings retained |
| manual-replay-02 | `python3 /tmp/yai-i05-manual-replay.py` | 0 | independent product-command replay; loopback_fixture; two peers on 18351/18352, new disposable home |
| external-01 | `TERMINAL make test-external-yvex` | 2 | external_yvex; dependency unavailable; inner qualifier exit 3, no dispatch |

The shell wrapper retained each command's real exit (`status_code=$?`, tail,
then `exit "$status_code"`). Macro shorthand in the table describes the
environment; it is not an operator command. Reproduction commands are below.
`verification.jsonl` independently records exact baseline/remote, fmt, registry,
Bash/manual syntax, root README preservation, diff and topology commands with
their unedited outputs and real exit codes.

```sh
make test-fast
make smoke-cognitive-target-arbitration
PATH="$PWD/build/r4-venv/bin:$PATH" CARGO_NET_OFFLINE=true make check characterization
```

The last command assumes the existing R4 test venv; on another workstation use
the dependency setup already documented in `tests/README.md`. No YVEX is needed.

## No-provider architectural proof

Bounded raw excerpt from `arbitration-03.log.gz`:

```text
i05_no_provider: policy_bounds=8 duplicate=reject wrong_capability=reject wrong_target=reject spoofed_principal=reject hidden_participant=reject atomic_failure=true v15_v13_markers=upgraded history=unchanged
test store::lmdb::tests::i05_tests::arbitration_policy_security_bounds_and_historical_readers ... ok
i05_no_provider: health_excludes_without_semantic_mutation=true recovery_rearbitrates=true selection_to_invocation_race=blocked zero_invocations=true
test store::lmdb::tests::i05_tests::arbitration_changes_between_selection_and_invocation_fail_closed ... ok
i05_no_provider: replay=exact preferred=provider-target:51558c0b8ed7628c14af1ab1fda4a526 shape_fallback=provider-target:5d718a87f73bed142c6f7fcf360b526f trust_and_envelope=excluded pinned=no_fallback stale_plan=no_selection lane_isolated=true
test store::lmdb::tests::i05_tests::arbitration_replay_shape_trust_envelope_and_pinning ... ok
```

These proofs use the actual LMDB canonical writer/replay and invocation-admission
seam, but no network. A candidate's circuit is closed again between exact
selection and invocation: the now-stale snapshot rejects invocation even though
the selected fallback target itself is still healthy. Current binding history
is unchanged. Other tests prove missing/wrong semantic evidence, mechanical
shape exclusion, exact replacement, pinned behavior, historical readers,
atomic failed admission and lane-continuation isolation.

## Actual adapter/transport and composition proof

`i05-loopback-02` uses product launcher `./yai`, fresh process per command and
real fixture HTTP. Inner run `yai-i05-ec94s6o8` retains 68 exact command records.
Its bounded final raw output is:

```text
{"i05_qualification": "PASS", "proof_class": "product/recovery", "provider_mode": "loopback_fixture", "native_bypass": true, "ordered_shape_exclusions": true, "composed_arbitrated_targets": true, "restart_no_redispatch": true, "indeterminate_no_cross_target_retry": true, "actual_requests": {"preferred": 1, "native": 1, "auxiliary": 1, "drop": 1, "unsuitable": 0}}
```

The preferred primary is named `whisper-vision-best-name` but has only text wire
evidence. `plain-native` wins for admitted audio; auxiliary is bypassed. After
independent trust changes, explicit speech prerequisites use the admitted
auxiliary and the reconstructed text closure uses the exact preferred primary.
Restart after the prerequisite failpoint consumes the committed derivation
without another auxiliary request. Repetition keeps the same closure and both
request counts. A misleading `whisper-vision` target without exact semantic
evidence cannot bind. A different source hits the drop peer: after trust loss,
the next explicit cycle must not dispatch another auxiliary or primary.

Synthetic qualification probes are excluded from the `actual_requests` count.
They still traversed HTTP; this number denotes semantic executions, not total
network traffic. Model intelligence/STT accuracy and live YVEX are not claimed.
The final `release-02` runs this same leaf again, under its own new isolated IDs.

## Independent operator-command replay

The byte-exact final replay driver is retained as `manual-replay-driver.txt`.
It executes the shell blocks in `MANUAL-ACCEPTANCE.md`, supplies the IDs emitted
by prior commands to `read`, and checks the resulting JSON. Its original path
was `/tmp/yai-i05-manual-replay.py`; it can also be run with Python using the
retained `.txt` path. It requires free fixture ports 18351/18352. This is
automated reproduction, **not a claim of human acceptance**.

Raw final line, `manual-replay-02.log.gz`:

```text
{"run_id": "i05-manual-replay-1252331", "manual_replay": "PASS", "human_acceptance": "not_claimed", "proof_class": "product", "provider_mode": "loopback_fixture", "commands": 48, "exact_realizations": 2, "pinned_refusal": true, "immutable_turn": true, "cleanup": "disposable_home_removed"}
```

48 entries include fixture startup, shell setup/ID reads, canonical product
commands and cleanup; they are not 48 independent architectural tests. Two
successful primary results are real loopback executions. Changing trust selects
another exact target/lane on a fresh plan. Replacing the policy with pinned B
then denying B leaves no selected target even when A is eligible. Both Turn
inspections are equal. Only the disposable home was deleted; fixture peers
were stopped. No existing Case, content, provider config or model was removed.

## Publication and compatibility evidence

`release-02.log.gz` is the complete successful `check characterization` run.
It includes I01–I04, provider contract/product flows, H19/W20, runtime, recovery,
the pre-existing REPLAI terminal test and the release-bounded endurance lane.
The final line is:

```text
make: Nothing to be done for 'characterization'.
```

That line alone is not the proof: the full union preceding it exited 0.
Shared Make leaves ran once in this invocation. Separate diagnostic runs
intentionally repeated some proof; there is no persistent PASS cache. The
explicit extended endurance lane was not run, and no I05 performance benchmark
or external compatibility claim is inferred from aggregate test counts.

Raw topology output retained in `verification.jsonl`:

```text
coverage_parity: PASS old_smoke_leaves=71 current_make_leaves=77 combined_duplicate_make_leaves=0
validation_catalog: PASS entries=420 assertion_files=83 rust_tests=334
```

The historical baseline comparison is maintained by TEST.TOPOLOGY.0 itself.
I05 adds four classified Rust tests and one Make product/recovery fixture leaf;
it removes no reachable proof. Registry remains 180 operations with zero help
or handler failures. Clippy warnings are in unchanged legacy logic; there is
no `-D warnings` or warning-free claim. No schema/owner/DB changes beyond the
versioned cognitive contracts documented in the report.

## Real failures, not erased from the record

| Raw run | Real exit / pre-state | Finding and correction |
|---|---|---|
| i05-loopback-01 | 1; initial fixture setup | Fixture asked `--max-attempts 5`; existing envelope bound correctly rejected it (`case_provider_binding_bounds_invalid`, CLI exit 3). Test changed to 3; runtime bound unchanged. |
| arbitration-02 | 101; new Rust assertion compiling | E0505 in test setup moved a borrowed target; cloned fixture target. |
| local-01 | 2; ambient Python without R4 venv | `ModuleNotFoundError: No module named 'pyte'`. Reused the already installed pinned R4 venv; did not skip the terminal test or install dependencies. |
| release-01 / i03-trace-01 | 2 / 1; initial shape-aware CLI | I03 expected mechanical shape refusal, but got `cognitive_realization_plan_unresolved:Some(AuxiliarySuitabilityMissing)`. Preserved the mechanical diagnostic and exposed candidate exclusions. No provider dispatch had slipped through. |
| i03-trace-02 | 1; diagnostic corrected | New canonical uncertain-delivery guard correctly refused changing targets after `ResponseInvalid`; old combined fixture expected another dispatch. Preserved refusal, added an explicit assertion of zero dispatch, and separated the next normalization scenario with a new SEND. |
| manual-replay-01 | 1; all documented commands already exited 0 and cleanup completed | Driver incorrectly expected an explicit null optional target field; the schema omits it for unresolved plans. Driver now checks absence or null; product JSON unchanged. |

Bounded raw excerpt, `i03-trace-02.log.gz`:

```text
+ EMPTY_OUTPUT='{"schema":"yai.cli.error.v1","operation_id":"yai.case.cognitive.realize","status":"error","code":"operation_failed","message":"cognitive_realization_prior_delivery_indeterminate_requires_resolution"}'
```

No failure above authorizes bypassing current evidence or unsafe retry.

## YVEX EXTERNAL FINDINGS

Run `yvex-black-box-20260906T190234Z-1197862`, exact outer command
`PATH="$PWD/build/r4-venv/bin:$PATH" make test-external-yvex`, exit 2 (inner
qualifier 3). Endpoint/model absent. Bounded raw excerpt:

```text
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
```

No new observed YVEX defect or interoperability success. No source/repository
inspection or administration was performed. The operator-supplied YVEX SHA is
not substituted for deployment evidence. Public typed-media realization and
production semantic evidence remain post-I05 integration work.
