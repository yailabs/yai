# I01 execution evidence

All excerpts below are bounded slices of the retained raw transcripts. They are
not reconstructed outputs. The working directory for every run was
`/home/mothx/computer-science/projects/YAI/yai` on 2026-09-04. The source
pre-state was clean `master` at reconciled baseline
`398def1e7391b736dd58280cff6f29e96248635b`; the implementation was
uncommitted during qualification.

## I01-E001 — focused content and context contracts

- run ID: `i01-focused-final-20260904`
- order: 1
- environment: repository-local Cargo defaults; no provider endpoint
- command:

```text
cargo test --manifest-path engine/yai-engine/Cargo.toml conversation -- --nocapture
cargo test --manifest-path engine/yai-engine/Cargo.toml i01_conversation_turn_tenant_must_match_canonical_case_security_domain -- --nocapture
cargo test --manifest-path engine/yai-engine/Cargo.toml committed_multipart_turn_projects_in_exact_order_without_provider_lineage -- --nocapture
```

- exit: `0`, then `0`, then `0`
- raw excerpt:

```text
running 8 tests
test conversation::tests::identical_bytes_remain_case_and_tenant_scoped ... ok
test conversation::tests::incomplete_object_publication_is_never_accepted_as_adopted_content ... ok
test conversation::tests::identical_draft_labels_are_namespaced_by_case_and_preview_does_not_adopt_bytes ... ok
test conversation::tests::corruption_and_internal_symlink_substitution_fail_closed ... ok
test conversation::tests::rejects_cross_case_derivation_and_oversized_metadata ... ok
test conversation::tests::multipart_order_and_duplicate_content_survive_publication ... ok
test store::lmdb::tests::i01_conversation_turn_tenant_must_match_canonical_case_security_domain ... ok
test conversation::tests::original_machine_transcript_and_human_edit_are_distinct ... ok
test result: ok. 8 passed; 0 failed; 0 ignored

running 1 test
test store::lmdb::tests::i01_conversation_turn_tenant_must_match_canonical_case_security_domain ... ok
test result: ok. 1 passed; 0 failed; 0 ignored

running 1 test
test context::tests::committed_multipart_turn_projects_in_exact_order_without_provider_lineage ... ok
test result: ok. 1 passed; 0 failed; 0 ignored
```

- invariant: content scope/integrity/order/provenance, authenticated canonical
  Case/Tenant binding, and provider-independent Projection/Context semantics
  pass mechanically.

## I01-E002 — literal zero-to-use-case product acceptance

- run ID: `i01-manual-acceptance-20260904`
- order: 2
- environment: fresh
  `YAI_HOME=/tmp/yai-i01-manual-acceptance`, `NO_COLOR=1`; Tenant
  `tenant:i01-acceptance`; Case `case:i01-acceptance`; Participant
  `participant:operator`; deliberately unavailable loopback provider
  `http://127.0.0.1:9/v1`; no YVEX dependency
- command:

```text
awk '/^```bash$/{inside=1; next} /^```$/{inside=0; next} inside{print}' refoundation/foundation-recovery/interlock-01/MANUAL-ACCEPTANCE.md > /tmp/i01-manual-acceptance-final.sh
bash -n /tmp/i01-manual-acceptance-final.sh
bash /tmp/i01-manual-acceptance-final.sh > /tmp/i01-manual-acceptance-final.log 2>&1
```

- exit: `0`
- produced identifiers: multipart Turn
  `conversation-turn:sha256:444e8675648e146f60d9d2a223f5eff97c62f62e5ab30e63cd4dfb02b80c1344`;
  failure-path Turn
  `conversation-turn:sha256:1549a8f4a64497f4a384584a9df8a3e464c86fd44908f5be066eac0157cc1c3f`
- raw excerpt:

```text
ordered_parts: 6
provider_execution_started: no
content_integrity: verified
part: 3 id=content-part:sha256:231af590548d8539484b437938a9c04114563f897a87768f82297e5a6dda8846 type=audio mime=audio/x-yai-fixture bytes=120 digest=sha256:c8cb615447702e40e50b6103aeab06b90f8116ef4225e81962bdf4b245c14a24 object=content-object:f4769612f21dc47730574be276ecedf45cde3d4d24d0606e7b3b3f3bc7435354 storage=yai-content/v1/objects/f4769612f21dc47730574be276ecedf45cde3d4d24d0606e7b3b3f3bc7435354/payload provenance=original
derivation: content-derivation:sha256:d94dceb8add3adca5053865331053e57133bd87f54e068630f0c011f067d810e kind=SpeechTranscription sources=content-part:sha256:231af590548d8539484b437938a9c04114563f897a87768f82297e5a6dda8846 actor=Deterministic:fixture:i01-speech-transcriber provider_result=none
derivation: content-derivation:sha256:818c278082b9517693d865ccc9b5918787fa099931f0ceca3fe18adb116e277e kind=HumanEdit sources=content-part:sha256:859fea0348339db41bd7923d74bac882ebedef1f76903b794716ea7e1954d228 actor=Human:principal:72cc156b82060120eac8f7e234dbfcef provider_result=none
input_conversation_turn_id: conversation-turn:sha256:1549a8f4a64497f4a384584a9df8a3e464c86fd44908f5be066eac0157cc1c3f
runtime_status: ProviderFailureBudgetExhausted
multipart_turns: 3
legacy_text_turns: 0
multipart_turns: 0
legacy_text_turns: 0
I01 manual acceptance completed; disposable state removed
```

- invariant: normal registry-backed commands prove SEND before execution,
  ordered multipart/repeated media, original→machine/deterministic→human-edit
  lineage, provider failure survival, process reopen, and cross-Case isolation.

## I01-E003 — full repository regression

- run ID: `i01-make-check-final-20260904`
- order: 3
- environment: external loopback sockets admitted for existing provider tests;
  all test state temporary
- exact command:
  `make check > /tmp/i01-make-check-final-source.log 2>&1`
- exit: `0`
- raw excerpt:

```text
test result: ok. 274 passed; 0 failed; 4 ignored; 0 measured; 0 filtered out
test result: ok. 35 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out
{"handler_failures": 0, "help_failures": 0, "operation_count": 171, "registry_digest": "sha256:9448c1082c82a34126e2cc8b88ddb6ce8e3c31b88d8930a11307cc44127695a8", "schema": "yai.cli.command_discovery.v1", "visibility_counts": {"advanced": 12, "compatibility": 16, "plumbing": 45, "product": 97, "removed": 1}}
memory_index_hardening: pass
episodic_semantic_memory: pass
multipart_conversation: pass
turn_commit_before_provider: pass
original_derived_provenance: pass
provider_failure_preserves_turn: pass
```

- invariant: layout/docs/build/full engine/full CLI/registry and all smoke
  families through W20 remain green with I01.

## I01-E004 — independent characterization

- run ID: `i01-characterization-final-20260904`
- order: 4
- environment: external loopback sockets admitted; all test state temporary
- exact command:
  `make characterization > /tmp/i01-make-characterization-final-source.log 2>&1`
- exit: `0`
- raw excerpt:

```text
memory_index_hardening: pass
authority_delta: zero
lmdb_delta: zero
episodic_semantic_memory: pass
hierarchy_rebuild_exact: true
provider_reinference_on_rebuild: zero
cross_case_isolation: true
index_drop_preserved_hierarchy: true
multipart_conversation: pass
turn_commit_before_provider: pass
original_derived_provenance: pass
provider_failure_preserves_turn: pass
```

- invariant: I01 product behavior and lower long-horizon memory properties pass
  in the characterization lane independently of the unit-suite assertions.

## I01-E005 — formatting, lint, registry, dossier

- run ID: `i01-static-final-20260904`
- order: 5
- exact commands:

```text
CARGO_TARGET_DIR="$PWD/target" cargo clippy --manifest-path engine/Cargo.toml --workspace --all-targets --all-features
CARGO_TARGET_DIR="$PWD/target" cargo clippy --manifest-path cmd/yai/Cargo.toml --all-targets --all-features
cargo fmt --manifest-path engine/yai-engine/Cargo.toml -- --check
cargo fmt --manifest-path cmd/yai/Cargo.toml -- --check
python3 tests/characterization/cli-product-surface/audit_registry.py --binary ./yai
git diff --check
```

- exit: all `0`
- raw excerpt:

```text
Finished `dev` profile [unoptimized + debuginfo] target(s)
Finished `dev` profile [unoptimized + debuginfo] target(s)
{"handler_failures": 0, "help_failures": 0, "operation_count": 171, "registry_digest": "sha256:9448c1082c82a34126e2cc8b88ddb6ce8e3c31b88d8930a11307cc44127695a8", "schema": "yai.cli.command_discovery.v1", "visibility_counts": {"advanced": 12, "compatibility": 16, "plumbing": 45, "product": 97, "removed": 1}}
```

- note: normal repository Clippy completed with pre-existing warning classes;
  no warning points at the new conversation module or changed I01 call sites.
- invariant: source is formatted; registry/help/handlers resolve; Markdown/TSV
  package is structurally valid; no whitespace errors exist.

## I01-E006 — remote and parallel-provider reference closure

- run ID: `i01-remote-closure-20260904`
- order: 6
- environment: read-only network access; YVEX clone
  `/tmp/yvex-w20-readonly`; no provider credentials rendered
- exact commands:

```text
git ls-remote origin refs/heads/master
git -C /tmp/yvex-w20-readonly branch --show-current
git -C /tmp/yvex-w20-readonly rev-parse HEAD
git -C /tmp/yvex-w20-readonly rev-parse origin/models1
git -C /tmp/yvex-w20-readonly status --short
git ls-remote https://github.com/yailabs/yvex.git refs/heads/models1
```

- exit: all `0`
- raw excerpt:

```text
398def1e7391b736dd58280cff6f29e96248635b refs/heads/master
models1
c3f675d1213ce3a6d7387179bb22415775e40e37
c3f675d1213ce3a6d7387179bb22415775e40e37
c3f675d1213ce3a6d7387179bb22415775e40e37 refs/heads/models1
```

- invariant: YAI remote still equals the reconciled baseline before
  publication; the clean read-only YVEX reference equals current remote
  `models1`. A final YAI remote-equality check is performed after push and is
  reported in the handoff because a commit cannot contain its own SHA.

## I01-E007 — corrected duplicate-object product assertion

- run ID: `i01-smoke-corrected-20260904`
- order: 7
- environment: fresh temporary YAI_HOME and deliberately unavailable loopback
  provider, as created by the smoke
- exact command:
  `tests/characterization/multipart-conversation/test_multipart_conversation.sh > /tmp/i01-smoke-corrected.log 2>&1`
- exit: `0`
- raw excerpt:

```text
multipart_conversation: pass
turn_commit_before_provider: pass
original_derived_provenance: pass
provider_failure_preserves_turn: pass
```

- invariant: after the staged-review correction, the smoke extracts actual
  object IDs and proves that two repeated image positions share one immutable
  content object while retaining distinct part identities.
