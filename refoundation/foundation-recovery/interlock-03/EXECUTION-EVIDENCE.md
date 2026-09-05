# I03 execution evidence

All runs used `/home/mothx/computer-science/projects/YAI/yai` on `master`.
Output below is bounded and copied from the named actual run. Separate runs are
not combined into one causal proof.

## I03-E01 — baseline reconciliation

- Run ID: `i03-baseline-20260905`
- Order: 1
- Environment: repository worktree; no YAI_HOME
- Command: `git status --short; git rev-parse HEAD; git rev-parse origin/master; git branch --show-current`
- Exit: 0
- Raw output:

```text
09030ab2064ab943ae2cf22b0258125253259c54
09030ab2064ab943ae2cf22b0258125253259c54
master
```

- Invariant: local and tracked published I02 baseline were equal and the
  worktree was clean before mutation. A separate remote query resolved the same
  `refs/heads/master` SHA.

## I03-E02 — focused exact-target and pre-dispatch qualification

- Run ID: `i03-focused-20260905`
- Order: 2
- Environment: isolated per-test LMDB stores; no provider or network
- Command: `cargo test --manifest-path engine/Cargo.toml store::lmdb::tests::i03_tests:: -- --nocapture`
- Exit: 0
- Raw output:

```text
running 3 tests
i03_schema_upgrade: transition_v14_to_v15=pass
test store::lmdb::tests::i03_tests::i03_transition_v14_store_marker_upgrades_to_v15 ... ok
i03_mechanical_refusal: semantic_suitability=present wire_shape=absent provider_selection=absent
test store::lmdb::tests::i03_tests::i03_semantic_suitability_without_wire_shape_refuses_before_selection ... ok
i03_exact_selection: plan=cognitive-plan:41834b24cfb9c852ef0d2334d5937a07 selected=provider-target:f0776389b8419b5554dbb3005bca9f1a fallback=provider-target:d189e708e1c8432fe58b6f311b179467 substituted=false stale_plan=rejected
test store::lmdb::tests::i03_tests::i03_exact_realization_selection_never_substitutes_provider_target ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 285 filtered out
```

- Invariant: suitability without exact v4 wire evidence creates no selection;
  exact selection never substitutes another eligible envelope target, and stale
  planning refuses.

## I03-E03 — actual typed-provider realization smoke

- Run ID: `i03-realization-smoke-20260905`
- Order: 3
- Environment: fresh temporary YAI_HOME; six deterministic loopback
  OpenAI-compatible providers; misleading provider/model names; no credentials
- Command: `make smoke-typed-provider-realization`
- Exit: 0
- Raw output excerpt:

```text
test result: ok. 283 passed; 0 failed; 4 ignored
test result: ok. 36 passed; 0 failed; 4 ignored
running 2 tests
i03_mechanical_refusal: semantic_suitability=present wire_shape=absent provider_selection=absent
i03_exact_selection: plan=cognitive-plan:d9bd05268110647e6b0cdd5324b80970 selected=provider-target:f0776389b8419b5554dbb3005bca9f1a fallback=provider-target:d189e708e1c8432fe58b6f311b179467 substituted=false stale_plan=rejected
test result: ok. 2 passed; 0 failed
i03_provider_dispatch: native=1 auxiliary=2 exact_target=true
i03_crash_recovery: result_recorded=true reinvocation=false turn_immutable=true
i03_fail_closed: missing_wire_evidence=true malformed=true normalization=true delivery_indeterminate_no_retry=true participant_isolation=true derived_count=2
```

- Produced identities (bounded excerpt):

```text
native plan_id=cognitive-plan:f293e17a5c1d1e92fb832143a470484e
native provider_result_id=provider-result:case:i03-cli:model-output-3
speech provider_result_id=provider-result:case:i03-cli:model-output-6
speech derived_content_id=conversation-derived:sha256:61c58a7ef21f92a0302e7ddfcbfc9cfdb8f5956784c7b2e26bbdc29cd49a5db5
image provider_result_id=provider-result:case:i03-cli:model-output-9
image derived_content_id=conversation-derived:sha256:c3d5aad229caa8194e6651dc3d6bedd8babb112b3b8b6f62ed4c09fc232649f3
```

- Invariant: the production adapter/transport performed one native and two
  auxiliary HTTP dispatches; ordered duplicate PNG parts remained distinct;
  result-first crash recovery did not reinvoke; malformed, empty,
  delivery-indeterminate, missing-wire and wrong-Participant paths published no
  false content.

## I03-E04 — zero-to-use-case product acceptance

- Run ID: `i03-manual-acceptance-20260905`
- Order: 4
- Environment: fresh `/tmp/yai-i03-manual.*` YAI_HOME; three loopback providers;
  repository launcher `./yai`
- Command: `awk '/^```bash$/{inside=1;next} /^```$/{inside=0;next} inside{print}' refoundation/foundation-recovery/interlock-03/MANUAL-ACCEPTANCE.md | bash`
- Exit: 0
- Raw output excerpt:

```text
Transition schema yai.transition.v15
provider_execution_started: no
"route":"native"
"posture":"recovered_without_dispatch"
"recovered_from_recorded_result":true
"content_integrity":"verified"
"realization_shape":"ordered_png_text_to_text"
"message":"cognitive_realization_shape_not_qualified"
I03 manual acceptance completed; disposable state removed
```

- Produced identities (bounded excerpt):

```text
speech provider_result_id=provider-result:case:i03-manual:model-output-6
speech derived_content_id=conversation-derived:sha256:756a8aff769f05e27e9b5924f1ead84daa599fba8b962b7072398f0c9f78ff41
image provider_result_id=provider-result:case:i03-manual:model-output-9
image derived_content_id=conversation-derived:sha256:9ffae8637f3d37f6b3778be46a3d85c6fb42d15742003fbb90efa4ead8b62ed7
```

- Invariant: natural `./yai` commands start from zero, commit Turns before
  realization, perform native/audio/image dispatch, recover after a durable
  result, verify immutable bytes and exact provenance after restart, exercise a
  no-dispatch negative, and clean only disposable state.

## I03-E05 — manual script syntax

- Run ID: `i03-manual-syntax-20260905`
- Order: 5
- Environment: repository workspace
- Command: `awk '/^```bash$/{inside=1;next} /^```$/{inside=0;next} inside{print}' refoundation/foundation-recovery/interlock-03/MANUAL-ACCEPTANCE.md | bash -n`
- Exit: 0
- Raw output: empty
- Invariant: every published manual command block is valid Bash without a
  hidden `set -e`, `set -u`, or `pipefail` contract.

## I03-E06 — complete repository check

- Run ID: `i03-make-check-final-20260905`
- Order: 6
- Environment: repository test environment with loopback socket permission
- Command: `make check`
- Exit: 0
- Raw output excerpt:

```text
check-no-old-roots: ok
check-required-layout: ok
check-doc-links: ok (30 files)
test result: ok. 285 passed; 0 failed; 4 ignored
test result: ok. 36 passed; 0 failed; 4 ignored
controlled_effect:allow_deny_second_turn ok
memory_index_hardening: pass
episodic_semantic_memory: pass
multipart_conversation: pass
i03_provider_dispatch: native=1 auxiliary=2 exact_target=true
i03_crash_recovery: result_recorded=true reinvocation=false turn_immutable=true
i03_fail_closed: missing_wire_evidence=true malformed=true normalization=true delivery_indeterminate_no_retry=true participant_isolation=true derived_count=2
```

- Invariant: layout, docs, build, the full lower-wave smoke matrix, persisted
  v14 store upgrade, I01/I02 compatibility and I03 qualification pass in one
  repository contract.

## I03-E07 — complete characterization

- Run ID: `i03-characterization-final-20260905`
- Order: 7
- Environment: repository characterization environment; local deterministic
  providers and isolated temporary stores
- Command: `make characterization`
- Exit: 0
- Raw output excerpt:

```text
test result: ok. 285 passed; 0 failed; 4 ignored
test result: ok. 36 passed; 0 failed; 4 ignored
provider_model_vertical:real_http_invocation ok
semantic_continuity:provider_replacement ok
controlled_effect:allow_deny_second_turn ok
memory_index_hardening: pass
episodic_semantic_memory: pass
multipart_conversation: pass
turn_commit_before_provider: pass
provider_failure_preserves_turn: pass
```

- Invariant: existing provider, authority, workflow, memory and conversation
  behavior remains characterized after the version advances.

## I03-E08 — static, registry and artifact contracts

- Run ID: `i03-static-final-20260905`
- Order: 8
- Environment: repository workspace; shared root target directory
- Commands:
  - `cargo fmt --manifest-path engine/Cargo.toml --all -- --check`
  - `cargo fmt --manifest-path cmd/yai/Cargo.toml --all -- --check`
  - `env CARGO_TARGET_DIR=target cargo clippy --manifest-path engine/Cargo.toml --workspace --all-targets`
  - `env CARGO_TARGET_DIR=target cargo clippy --manifest-path cmd/yai/Cargo.toml --all-targets`
  - `make check-layout check-docs`
  - `python3 tests/characterization/cli-product-surface/audit_registry.py --binary ./yai`
  - `git diff --check`
  - `awk '/^```bash$/{inside=1;next} /^```$/{inside=0;next} inside{print}' refoundation/foundation-recovery/interlock-03/MANUAL-ACCEPTANCE.md | bash -n`
  - `awk -F '\t' 'NF == 0 { next } FNR==1 { expected=NF } NF != expected { print FILENAME ":" FNR ": expected " expected ", got " NF; bad=1 } END { exit bad }' refoundation/foundation-recovery/interlock-03/*.tsv`
- Exit: 0 for every canonical command
- Raw output excerpt:

```text
check-no-old-roots: ok
check-required-layout: ok
check-source-placement: ok
check-source-surface-clean: ok
check-doc-links: ok (30 files)
{"handler_failures": 0, "help_failures": 0, "operation_count": 179, "registry_digest": "sha256:fdad42993e3ce598e2441739ad40b0919e3718e7d988ccf3cc0198ae489be070", "visibility_counts": {"advanced": 27, "compatibility": 16, "plumbing": 45, "product": 90, "removed": 1}}
```

Clippy completed with the repository's existing advisory warnings in historical
unrelated code; the sole new I03 lint finding was corrected before this run.
Publication equality is recorded only in the post-commit handoff.

## I03-E09 — final boundary audit and requalification

- Run ID: `i03-boundary-audit-final-20260905`
- Order: 9
- Environment: isolated LMDB stores and deterministic loopback providers;
  repository root binary rebuilt from the final source
- Commands:
  - `cargo test --manifest-path engine/Cargo.toml store::lmdb::tests::i03_tests:: -- --nocapture`
  - `cargo test --manifest-path engine/Cargo.toml conversation::tests::provider_derived_content_binds_capability_shape_and_source_modality -- --exact`
  - `cargo test --manifest-path cmd/yai/Cargo.toml command_adapters::provider::tests::i03_typed_input_preflight_is_bounded_and_signature_safe -- --exact`
  - `make smoke-typed-provider-realization`
  - `awk '/^```bash$/{inside=1;next} /^```$/{inside=0;next} inside{print}' refoundation/foundation-recovery/interlock-03/MANUAL-ACCEPTANCE.md | bash`
- Exit: 0 for every command
- Raw output excerpt:

```text
i03_exact_selection: plan=cognitive-plan:d24fc4e9f7271e8aff70dc452fad2dc7 selected=provider-target:f0776389b8419b5554dbb3005bca9f1a fallback=provider-target:d189e708e1c8432fe58b6f311b179467 substituted=false capability_shape_mismatch=rejected stale_plan=rejected stale_binding_dispatch=rejected
i03_provider_dispatch: native=1 auxiliary=2 exact_target=true
i03_crash_recovery: result_recorded=true reinvocation=false turn_immutable=true
i03_fail_closed: missing_wire_evidence=true malformed=true normalization=true delivery_indeterminate_no_retry=true participant_isolation=true derived_count=2
I03 manual acceptance completed; disposable state removed
```

- Invariant: the final audit binds capability to realization shape and source
  modality, emits no proprietary YAI fields on the OpenAI-compatible wire,
  rejects a cognitive-binding race at invocation admission, requires current
  Principal/Participant linkage before source reads and inspection, reads
  immutable source bytes through one verified descriptor-anchored snapshot,
  and preserves the result-first recovery contract.
