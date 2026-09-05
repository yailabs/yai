# Execution evidence

This file retains bounded, materially complete outputs from real runs. The
post-commit handoff records the final SHA and publication equality so this
pre-publication report does not attempt to name the commit that contains it.

## E-POST-I01-01 — baseline reconciliation

- cwd: repository root
- command: `git branch --show-current; git rev-parse HEAD; git rev-parse origin/master; git status --short`
- exit: `0`
- observed output:

```text
master
82287cf92b8a95b70d387ca759b56c593848983c
82287cf92b8a95b70d387ca759b56c593848983c
```

- invariant: implementation began from the clean published I01 anchor.

## E-POST-I01-02 — remote YVEX reference

- cwd: repository root
- command: `git ls-remote https://github.com/yailabs/yvex.git refs/heads/models1`
- exit: `0`
- observed output:

```text
cb336ad60c12d6fa841dc0715bba9d44aa721846	refs/heads/models1
```

- invariant: interlock classification uses the current remote reference, not
  I01's older `c3f675d...` checkpoint.

## E-POST-I01-03 — commit/thread/media host semantics

- cwd: repository root
- command: `cargo test --manifest-path cmd/yai/Cargo.toml command_adapters::conversation_controller::tests::post_i01_host_commits_before_provider_and_derives_threads_only_from_turns -- --ignored --exact --nocapture --test-threads=1`
- exit: `0`
- bounded output:

```text
running 1 test
test command_adapters::conversation_controller::tests::post_i01_host_commits_before_provider_and_derives_threads_only_from_turns ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 38 filtered out
```

- invariant: SEND commits before provider availability; threads remain
  history-derived; repeated media parts preserve order; missing I02 media
  delivery cannot erase the Turn.

## E-POST-I01-04 — shared governed execution seam

- cwd: repository root
- environment: isolated test `YAI_HOME`, loopback socket permission
- command: `cargo test --manifest-path cmd/yai/Cargo.toml command_adapters::conversation_controller::tests::post_i01_host_reuses_governed_semantic_execution_without_operational_runtime -- --ignored --exact --nocapture --test-threads=1`
- exit: `0`
- bounded output:

```text
provider_target: registered
qualification: recorded
qualified_capabilities: ChatText,StructuredJsonObject,ModelExactAddressing,UsageAccounting
provider_trust: recorded
case_provider_binding: recorded
failover_policy: SafeOnly
max_attempts_per_turn: 1
test command_adapters::conversation_controller::tests::post_i01_host_reuses_governed_semantic_execution_without_operational_runtime ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 38 filtered out
```

- invariant: a text conversation uses generic provider governance and semantic
  context with zero ResourceAttachments; retry produces a second invocation
  causally linked to the same one committed Turn.

## E-POST-I01-05 — complete regression

- cwd: repository root
- environment: local loopback and multi-process fixtures admitted
- exact command: `make check`
- exit: `0`
- bounded raw output excerpt:

```text
check-no-old-roots: ok
check-required-layout: ok
check-source-placement: ok
check-source-surface-clean: ok
check-doc-links: ok (30 files)
test result: ok. 274 passed; 0 failed; 4 ignored
test result: ok. 35 passed; 0 failed; 4 ignored
memory_index_hardening: pass
episodic_semantic_memory: pass
multipart_conversation: pass
turn_commit_before_provider: pass
original_derived_provenance: pass
provider_failure_preserves_turn: pass
test command_adapters::conversation_controller::tests::post_i01_host_commits_before_provider_and_derives_threads_only_from_turns ... ok
test command_adapters::conversation_controller::tests::post_i01_host_reuses_governed_semantic_execution_without_operational_runtime ... ok
```

- invariant: lower-wave, H19, W20, I01, docs/layout, engine, CLI, and the new
  conversation-host smoke all remain green.

## E-POST-I01-06 — characterization

- cwd: repository root
- environment: local loopback and multi-process fixtures admitted
- exact command: `make characterization`
- exit: `0`
- bounded raw output excerpt:

```text
provider_model_vertical:real_http_invocation ok
semantic_continuity:memory_inspect_drop_rebuild ok
provider_governance_hardening_characterization: pass
memory_representation_characterization: pass
memory_index_hardening: pass
episodic_semantic_memory: pass
multipart_conversation: pass
turn_commit_before_provider: pass
original_derived_provenance: pass
provider_failure_preserves_turn: pass
test command_adapters::conversation_controller::tests::post_i01_host_commits_before_provider_and_derives_threads_only_from_turns ... ok
test command_adapters::conversation_controller::tests::post_i01_host_reuses_governed_semantic_execution_without_operational_runtime ... ok
```

- invariant: the semantic host is additive to existing provider, memory,
  workflow, authority, and multipart-content behavior.

## E-POST-I01-07 — registry and static contracts

- cwd: repository root
- exact commands:

```text
CARGO_TARGET_DIR="$PWD/target" cargo clippy --manifest-path engine/Cargo.toml --workspace --all-targets --all-features
CARGO_TARGET_DIR="$PWD/target" cargo clippy --manifest-path cmd/yai/Cargo.toml --all-targets --all-features
cargo fmt --manifest-path cmd/yai/Cargo.toml -- --check
python3 tests/characterization/cli-product-surface/audit_registry.py --binary ./yai
make check-layout
make check-docs
git diff --check
```

- exit: all `0`
- bounded raw output excerpt:

```text
Finished `dev` profile [unoptimized + debuginfo] target(s)
{"handler_failures": 0, "help_failures": 0, "operation_count": 171, "registry_digest": "sha256:7cc7150b051b8fbcdbbbb8b596902b241218a350b4a986ac4923a18add9d3907", "schema": "yai.cli.command_discovery.v1", "visibility_counts": {"advanced": 19, "compatibility": 16, "plumbing": 45, "product": 90, "removed": 1}}
check-required-layout: ok
check-source-placement: ok
check-doc-links: ok (30 files)
```

- note: normal repository Clippy retains pre-existing warnings in established
  engine/provider code; no warning points to `conversation_controller.rs`.
- invariant: the registry remains closed and executable; seven I01 draft/SEND
  operations move from Product to Advanced without deleting plumbing, while
  `yai prompt` remains Advanced.

## E-POST-I01-08 — zero-state manual product commands

- cwd: repository root
- environment: `YAI_HOME=/tmp/yai-post-i01-acceptance.UaoRgB`
- commands: the same command arguments now retained in `MANUAL-ACCEPTANCE.md`,
  executed individually without shell assertion scaffolding; this historical
  run invoked `./target/debug/yai` directly before the operator-facing launcher
  ambiguity was corrected to `./yai`
- exit: every command `0`
- produced identifiers:
  - Case: `case:post-i01-acceptance`
  - first Turn: `conversation-turn:sha256:86fb893f22c11befda7f6d8bde51a647a21f0172442f15a4bef21746dd4a109c`
  - second Turn: `conversation-turn:sha256:1f09eba3c49b297e8bf6bcd7445db4e71a7a9b67e8f076ea1a3025320329c5ac`
- bounded raw output excerpt:

```text
case_generation: 5
ordered_parts: 1
canonical: yes
provider_execution_started: no
draft_discarded: yes
case_generation: 6
ordered_parts: 1
canonical: yes
provider_execution_started: no
draft_discarded: yes
multipart_turns: 2
legacy_text_turns: 0
turn: conversation-turn:sha256:86fb893f22c11befda7f6d8bde51a647a21f0172442f15a4bef21746dd4a109c parts=1 thread=thread:manual-one
turn: conversation-turn:sha256:1f09eba3c49b297e8bf6bcd7445db4e71a7a9b67e8f076ea1a3025320329c5ac parts=1 thread=thread:manual-two
provider_execution_required_for_identity: no
content_integrity: verified
Visibility: Advanced
```

- cleanup: the exact isolated `/tmp/yai-post-i01-acceptance.UaoRgB` directory
  was removed after inspection.
- invariant: ordinary registry commands prove canonical SEND, restart/process
  reopening, thread derivation, content integrity, and the frozen Advanced
  prompt posture without pretending the future interactive frontend exists.

## E-POST-I01-09 — external qualification posture

- cwd: repository root
- observation: `YAI_EXTERNAL_PROVIDER_BASE_URL` and
  `YAI_EXTERNAL_PROVIDER_MODEL` were unavailable
- external result: `blocked_external_dependency`
- classification: `DEPLOYMENT_LIMITATION`
- invariant: deterministic loopback provider qualification proves the generic
  YAI seam, but no live YVEX/model result is fabricated.

## E-POST-I01-10 — repository-local operator launcher

- cwd: repository root
- exact command: `./yai --help`
- exit: `0`
- invariant: the documented `./yai` operator entry point resolves the freshly
  built local product binary; manual acceptance does not expose Cargo's
  `target/debug/yai` build path.
