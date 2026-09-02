# Wave 18 execution evidence

All blocks below are real, bounded excerpts. The final publication SHA is not
known to the commit that contains this file; publication equality belongs in
the post-commit handoff.

## E18-BASELINE

- run_id: `W18-BASELINE-20260902`
- order: 1
- pre-state: published H17 master, 13 preserved dirty entries
- cwd: repository root
- environment: ordinary local shell; no provider endpoint variables
- commands: `git branch --show-current`; `git rev-parse HEAD`;
  `git rev-parse origin/master`; `git ls-remote origin refs/heads/master`
- exits: 0
- stdout:

```text
master
8d6d9fe79f450e42f25324d4e80987e2873a1ae2
8d6d9fe79f450e42f25324d4e80987e2873a1ae2
8d6d9fe79f450e42f25324d4e80987e2873a1ae2 refs/heads/master
```

- invariant: W18 began from the exact published H17 final baseline.

## E18-YVEX-REFERENCE

- run_id: `W18-YVEX-READONLY-20260902`
- order: 2
- pre-state: no YVEX worktree in the writable YAI repository
- cwd: `/tmp/yvex-w18-readonly`
- environment: read-only reference clone, `models1`
- command: `git rev-parse HEAD; git status --short`
- exit: 0
- stdout:

```text
5b3aa34be8999ad8240403e884074833d80c301d
```

- invariant: exact current reference SHA, clean and unmodified.

## E18-ENGINE-CONTRACT

- run_id: `W18-CHAR-20260902-1204c5`
- order: 3
- pre-state: isolated temporary LMDB stores per test
- cwd: repository root
- environment: no live external provider; local test principals/Tenants/Cases
- command: `tests/characterization/provider-governance/test_provider_governance.sh`
- exit: 0
- stdout excerpt:

```text
w18_governed_selection: target=provider-target:8495f14e3724783c1744fd7bf6e286ad qualification=provider-qualification:14c7d9a15e2d5c354f355648928ff097 binding=case-provider-binding:58fe8a27fba89af88f2c6cab46583a68 selection=provider-selection:993ca36f34a1fdfcf5fe75916737f501 generation=4 replay=true exclusions=0
w18_selection_filter: selected=provider-target:faa4fac12d9ee7c56e54afc29194ebe6 text_only=required_capability_missing denied=trust_not_approved cross_tenant=cross_tenant_provider_target_binding_rejected
w18_delivery: primary_circuit=open selected_secondary=provider-selection:52cc91f94d597531e9c90ff8b7d4e51f delivery=indeterminate retry_safe=false automatic_failover=false outcome=provider-attempt-outcome:deccaaa2ac5cf5e035db2d2c0b7ecaf5
w18_concurrency: contenders=16 selections=1 outcomes=1 generation=5 replay=true
w18_qualification_order: newer=provider-qualification:9311a30a3947576604af7fcf4d864b6e stale_late=provider-qualification:e2c0d9f3240ac9e8f8d320325e9b169e current=provider-qualification:9311a30a3947576604af7fcf4d864b6e rollback=false
w18_revoke_start_race: invocation_committed=false final_trust=denied serializable=true
```

- invariant: exact Case selection replay, mechanical exclusions, no unsafe
  retry after indeterminate delivery, concurrent one-truth admission,
  monotonic qualification projection and serializable revoke/start.

## E18-PRODUCT-FIXTURE

- run_id: `W18-PRODUCT-20260902-1204c5`
- order: 4
- pre-state: fresh `mktemp` YAI_HOME, synthetic loopback providers, Tenant
  `tenant:w18-smoke`, Cases `case:w18-smoke`,
  `case:w18-safe-failover`, `case:w18-indeterminate`
- Principal: kernel-authenticated local Tenant Owner
- Participant: `participant:model`
- command: `tests/characterization/provider-governance/test_provider_governance.sh`
- exit: 0
- stdout excerpt:

```text
provider_governance_characterization: pass
synthetic_case_context_items_sent: 0
qualified_capabilities: chat_text,structured_json_object,model_exact_addressing,usage_accounting
provider_dimensions_collapsed: false
indeterminate_automatic_failover: false
provider_selection_case_canonical: true
case_provider_binding_product_path: true
governed_provider_modelwork_completed: true
safe_connect_failover_completed: true
indeterminate_delivery_stopped_without_failover: true
provider_health_operational_shared: true
```

- generated identities: exact target/qualification/selection/run identities
  are emitted by the same executable script; target IDs vary with the
  isolated loopback ports.
- invariants: qualification sends only synthetic probes; an ordinary governed
  Case completes via the target; a stopped primary fails over after zero-byte
  connect refusal; an accept-and-drop primary records
  `DeliveryIndeterminate` and invokes no alternate.

## E18-TRANSPORT-BOUNDARY

- run_id: `W18-TRANSPORT-20260902`
- order: 5
- pre-state: local loopback listeners created by ignored focused tests
- cwd: repository root
- command: `cargo test --manifest-path cmd/yai/Cargo.toml wave18_ -- --ignored --nocapture`
- exit: 0
- stdout:

```text
running 2 tests
test command_adapters::provider::tests::wave18_connect_refused_is_provably_not_dispatched ... ok
test command_adapters::provider::tests::wave18_accepted_request_then_drop_is_delivery_indeterminate ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 23 filtered out
```

- invariant: byte-write boundary, not HTTP convenience, owns retry safety.

## E18-EXTERNAL

- run_id: `W18-EXTERNAL-CHECK-20260902`
- order: 6
- environment:

```text
YAI_EXTERNAL_PROVIDER_BASE_URL=missing
YAI_EXTERNAL_PROVIDER_MODEL=missing
```

- result: `blocked_external_dependency`
- invariant: no endpoint/model was invented and no YVEX administration was
  attempted.

## E18-FULL-CHECK

- run_id: `W18-MAKE-CHECK-20260902-3a6778`
- order: 7
- pre-state: all reproduced W18 defects corrected
- cwd: repository root
- command: `make check`
- exit: 0
- bounded stdout:

```text
check-doc-links: ok (30 files)
test result: ok. 195 passed; 0 failed; 0 ignored
test result: ok. 23 passed; 0 failed; 2 ignored
case_runtime:agentless_26_turn_provider_model_replacement ok
case_runtime:budget_stops_before_extra_invocation ok
human_review:crash_r1_r6_recovery ok
policy_authority:allow_chain ok
policy_authority:unconfigured_pre_provider_stop ok
```

- invariant: engine, CLI, repository layout and all make-check lower-wave
  authority/runtime/product contracts are green.

## E18-FULL-CHARACTERIZATION

- run_id: `W18-MAKE-CHARACTERIZATION-20260902-30c196`
- order: 8
- pre-state: clean temporary stores/fixtures created by each characterization
- cwd: repository root
- command: `make characterization`
- exit: 0
- exact provider identities from this run:
  - target `provider-target:4d418d25c57b90904c05262035f0c330`
  - qualification `provider-qualification:6fbe622d7838805e68e2a2816db3a0c2`
  - qualification run `qualification-run:3277b3c7a026dfa0:1788359099640`
- bounded raw stdout:

```text
provider_governance_characterization: pass
governed_provider_modelwork_completed: true
safe_connect_failover_completed: true
indeterminate_delivery_stopped_without_failover: true
run_id: case-run:6017298f17094c98
case_id: case:w18-smoke
runtime_status: Completed
invocations: 1
provider_failures: 0
last_provider_result_id: provider-result:case:w18-smoke:model-output-3
provider_safe_failover: attempt=2 reason:provider_not_dispatched:connect:Connection refused (os error 111)
run_id: case-run:b4f98fcff0251592
case_id: case:w18-safe-failover
runtime_status: Completed
provider_failures: 1
last_provider_result_id: provider-result:case:w18-safe-failover:model-output-4
run_id: case-run:268763703f84c2bb
case_id: case:w18-indeterminate
runtime_status: DeliveryIndeterminate
stop_detail: provider_delivery_indeterminate:invalid_http_response:bytes=3532
invocations: 0
provider_failures: 1
last_provider_result_id: none
```

- invariant: actual HTTP fixture behavior proves the normal governed path,
  safe zero-byte failover, and no failover after possible delivery; all W17/H17
  characterization remains green in the same target.

## E18-STATIC-CONFORMANCE

- run_id: `W18-STATIC-20260902`
- order: 9
- cwd: repository root
- commands: Rust fmt checks; engine and CLI Clippy under the repository warning
  contract; `make check-docs`; registry audit; `git diff --check`
- exits: 0
- stdout:

```text
check-doc-links: ok (30 files)
{"handler_failures": 0, "help_failures": 0, "operation_count": 143, "registry_digest": "sha256:d2b6c5d2987fb18eb283033fb624e98b588afa970e0d4dc9b3994acb7d4f7da2", "schema": "yai.cli.command_discovery.v1", "visibility_counts": {"advanced": 9, "compatibility": 16, "plumbing": 45, "product": 72, "removed": 1}}
```

- stderr posture: Clippy exited 0 with only the repository's admitted existing
  warning classes; W18 introduced no new warning class.
- historical tracked dirty checksum:
  `3fdb219654405e6fd40b5c0d1b02b94c04fadef5aa57a139aa5fb8fd6db7777e`.
- invariant: formatting, docs, registry/help/handler conformance and diff
  whitespace are closed.
