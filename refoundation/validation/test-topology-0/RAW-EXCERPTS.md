# Bounded unedited execution excerpts

Selected lines preserve their original bytes and order within each run. Full
major-gate logs are retained separately; these blocks are not one combined run.

## make check (old gate)

Source: `old-check-02.log` from the corresponding run above.

```text
test result: ok. 286 passed; 0 failed; 4 ignored; 0 measured; 0 filtered out; finished in 88.93s
i04_composed: auxiliary_dispatch_once=true restart_resume=true primary_dispatch=true turn_immutable=true audio_and_image=true
i04_direct: primary_native_audio=true auxiliary_bypassed=true explicit_intent_preserved=true
i04_failure: delivery_indeterminate_blocks_primary=true malformed_blocks_primary=true normalization_blocks_primary=true unresolved_blocks_primary=true derived_duplicates=0
i04_revalidation: binding_evidence_and_qualification_change_replans=true historical_derivation_preserved=true
elapsed_seconds: 211.86s
```

## make characterization (old gate)

Source: `old-characterization.log` from the corresponding run above.

```text
test result: ok. 286 passed; 0 failed; 4 ignored; 0 measured; 0 filtered out; finished in 87.60s
test result: ok. 286 passed; 0 failed; 4 ignored; 0 measured; 0 filtered out; finished in 88.18s
elapsed_seconds: 241.18s
```

## make check characterization (final graph)

Source: `new-final-combined.log` from the corresponding run above.

```text
Ran 8 tests in 0.053s
OK
coverage_parity: PASS old_smoke_leaves=71 current_make_leaves=75 combined_duplicate_make_leaves=0
validation_catalog: PASS entries=414 assertion_files=81 rust_tests=330
i04_composed: auxiliary_dispatch_once=true restart_resume=true primary_dispatch=true turn_immutable=true audio_and_image=true
i04_direct: primary_native_audio=true auxiliary_bypassed=true explicit_intent_preserved=true
i04_failure: delivery_indeterminate_blocks_primary=true malformed_blocks_primary=true normalization_blocks_primary=true unresolved_blocks_primary=true derived_duplicates=0
i04_revalidation: binding_evidence_and_qualification_change_replans=true historical_derivation_preserved=true
make: Nothing to be done for 'characterization'.
elapsed_seconds: 252.91s
```

## make test-external-yvex

Source: `external-absent.log` from the corresponding run above.

```text
CARGO_TARGET_DIR=target cargo build --manifest-path engine/Cargo.toml --workspace
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.01s
CARGO_TARGET_DIR=target cargo build --manifest-path cmd/yai/Cargo.toml
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.02s
{"validation_entry": {"id": "external-yvex", "kind": "make", "path": "tests/integration/yvex/qualification_yvex_provider.sh", "selector": "qualification-yvex-provider", "proof_class": "external", "evidence_posture": "qualification", "provider_mode": "external_yvex", "network": "external", "mutation": "temporary_case_and_provider_inference", "cadence": "external", "entrypoint": "qualification-yvex-provider", "reachability": "product", "property": "Exact exposed target text invocation; not I03/I04 typed media or model-quality qualification"}}
run_id: yvex-black-box-20260906T145052Z-67
yai_sha: 6e332851b5066cbb1da25f816b8db31b74580acb
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

## make check-validation-topology

Source: `topology-04.log` from the corresponding run above.

```text
Ran 8 tests in 0.053s
OK
coverage_parity: PASS old_smoke_leaves=71 current_make_leaves=75 combined_duplicate_make_leaves=0
validation_catalog: PASS entries=414 assertion_files=81 rust_tests=330
```

## make check-docs check-layout

Source: `static-guards.log` from the corresponding run above.

```text
doc_root_canon: ok
check-doc-canonical-location: ok
check-doc-required-files: ok
check-doc-links: ok (30 files)
check-repository-identity: ok
check-no-old-roots: ok
check-required-layout: ok
check-source-placement: ok
check-source-surface-clean: ok
```

## bash -n, 67 scripts and extracted manual blocks

Source: `bash-syntax.log` from the corresponding run above.

```text
bash_syntax: PASS scripts=67 manuals=2
```
