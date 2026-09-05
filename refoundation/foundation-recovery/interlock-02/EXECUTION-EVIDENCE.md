# I02 execution evidence

All runs used `/home/mothx/computer-science/projects/YAI/yai` on branch
`master`. Output below is bounded and copied from the actual run named in each
block; different runs are not combined as one causal proof.

## I02-E01 — baseline reconciliation

- Run ID: `i02-baseline-20260905`
- Order: 1
- Environment: repository worktree; no YAI_HOME required
- Command: `git branch --show-current; git rev-parse HEAD; git rev-parse origin/master; git status --short`
- Exit: 0
- Raw output:

```text
master
5e75a803a610d6d6b1deb33693bc794f315294d8
5e75a803a610d6d6b1deb33693bc794f315294d8
```

- Invariant: local and tracked published baseline were equal and the worktree
  was clean. Direct history inspection showed the requested `89e44cd...`
  semantic baseline plus one compatible launcher-documentation commit.

## I02-E02 — focused store/replay/adversarial qualification

- Run ID: `i02-focused-store-20260905`
- Order: 2
- Environment: isolated per-test LMDB stores; deterministic provider evidence;
  no network
- Command: `cargo test --manifest-path engine/Cargo.toml store::lmdb::tests::i02_tests:: -- --nocapture`
- Exit: 0
- Raw output excerpt:

```text
running 3 tests
i02_adversarial: misleading_name=unresolved envelope_mismatch=rejected cross_tenant_evidence=semantic_suitability_evidence_binding_mismatch provider_dispatches=0
test store::lmdb::tests::i02_tests::i02_binding_composition_and_name_inference_fail_closed ... ok
i02_plan: native=cognitive-plan:c12dc491c0b8503c40da7b97130310bc derived=cognitive-plan:dd792017b5130d513dad07b1ee07ae2e primary_lane=cognitive-lane:bee8c938d061b0e857dd07c4c778de92 auxiliary_lane=cognitive-lane:0ced1a6a9251ae82b0531b9bff797373 provider_dispatches=0 replay=true
test store::lmdb::tests::i02_native_derived_replay_and_zero_execution_contract ... ok
i02_replacement: explicit=true envelope_invalidation=true lane_changed=true unbound=true replay=true unauthorized_principal=rejected
test store::lmdb::tests::i02_replacement_envelope_invalidation_unbind_and_replay_are_explicit ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 282 filtered out
```

- Invariant: deterministic native/derived planning, explicit replacement,
  provider-envelope invalidation, Tenant/Principal isolation, replay and zero
  dispatch all hold.

## I02-E03 — dedicated CLI smoke

- Run ID: `i02-cli-smoke-20260905`
- Order: 3
- Environment: fresh temporary YAI_HOME; two loopback fixture providers stopped
  before planning
- Command: `make smoke-cognitive-capability-bindings`
- Exit: 0
- Raw output excerpt:

```text
test result: ok. 280 passed; 0 failed; 4 ignored
test result: ok. 35 passed; 0 failed; 4 ignored
running 4 tests
test result: ok. 4 passed; 0 failed
running 3 tests
test result: ok. 3 passed; 0 failed
test cli::registry::tests::registry_is_self_consistent ... ok
i02_cli: bindings=2 native=true derived=true unresolved=true provider_endpoints=stopped provider_dispatches=0
```

- Invariant: the actual registry-backed CLI composes provider governance,
  evidence, bindings and planning while stopped endpoints prove no planning
  dispatch.

## I02-E04 — complete CLI suite with loopback permission

- Run ID: `i02-cli-full-20260905`
- Order: 4
- Environment: repository test environment with loopback sockets permitted
- Command: `cargo test --manifest-path cmd/yai/Cargo.toml --no-fail-fast`
- Exit: 0
- Raw output:

```text
test result: ok. 35 passed; 0 failed; 4 ignored; 0 measured; 0 filtered out
```

- Invariant: complete CLI unit suite, including transport security, remains
  green. Ignored tests remain owned by their named smoke targets.

## I02-E05 — zero-to-use-case product acceptance

- Run ID: `i02-manual-acceptance-20260905`
- Order: 5
- Environment: fresh `/tmp/yai-i02-manual.*` YAI_HOME; two loopback fixture
  providers stopped before every planning request
- Command: `awk '/^```bash$/{inside=1;next} /^```$/{inside=0} inside{print}' refoundation/foundation-recovery/interlock-02/MANUAL-ACCEPTANCE.md | bash`
- Exit: 0
- Raw output excerpt:

```text
evidence_id: semantic-suitability:c6e525082e8edb6e40079fdaed61ebe6
evidence_id: semantic-suitability:e9e6be40e9c765cf23dff69b1cf3422a
binding_id: case-cognitive-binding:6f910cf95330447c34ab56039092b189
binding_id: case-cognitive-binding:2e6a14556e755784c2f83417b37c4a2d
execution_lane_id: cognitive-lane:99595d371694720e75d9aa8b296c2e06
provider_execution: not_performed
execution_lane_id: cognitive-lane:d7cfbc2352092d180216e18e66d0debf
provider_execution: not_performed
unresolved_reason: auxiliary_binding_missing
continuation_posture: rejected_cross_lane
I02 manual acceptance completed; disposable state removed
```

- Invariant: ordinary `./yai` commands start from zero, materialize exact
  evidence and canonical bindings, plan while both provider endpoints are
  stopped, preserve identities after a fresh process invocation, exercise
  unresolved and cross-lane failures, and clean all disposable state.

## I02-E06 — complete repository check

- Run ID: `i02-make-check-20260905`
- Order: 6
- Environment: repository test environment with loopback sockets permitted
- Command: `make check`
- Exit: 0
- Raw output excerpt:

```text
test result: ok. 281 passed; 0 failed; 4 ignored
test result: ok. 35 passed; 0 failed; 4 ignored
registry_audit: handler_failures=0 help_failures=0
i02_cli: bindings=2 native=true derived=true unresolved=true provider_endpoints=stopped provider_dispatches=0
```

- Invariant: layout, documentation, compilation, the complete lower-wave smoke
  matrix and I02 dedicated qualification are green together.

## I02-E07 — complete characterization

- Run ID: `i02-characterization-final-20260905`
- Order: 7
- Environment: repository characterization environment with local provider
  fixtures and isolated temporary stores
- Command: `make characterization`
- Exit: 0
- Raw output excerpt:

```text
test result: ok. 281 passed; 0 failed; 4 ignored; 0 measured; 0 filtered out
test result: ok. 35 passed; 0 failed; 4 ignored; 0 measured; 0 filtered out
provider_governance_characterization: pass
memory_representation_characterization: pass
test command_adapters::conversation_controller::tests::post_i01_host_commits_before_provider_and_derives_threads_only_from_turns ... ok
test command_adapters::conversation_controller::tests::post_i01_host_reuses_governed_semantic_execution_without_operational_runtime ... ok
```

- Invariant: historical provider, memory, workflow and conversation-host
  characterization remains green after the canonical binding/version delta.

## I02-E08 — format, lint, registry and documentation contracts

- Run ID: `i02-static-closure-20260905`
- Order: 8
- Environment: repository workspace; shared root Cargo target directory
- Commands:
  - `cargo fmt --manifest-path engine/Cargo.toml --all -- --check`
  - `cargo fmt --manifest-path cmd/yai/Cargo.toml --all -- --check`
  - `CARGO_TARGET_DIR=target cargo clippy --manifest-path engine/Cargo.toml --workspace --all-targets --all-features`
  - `CARGO_TARGET_DIR=target cargo clippy --manifest-path cmd/yai/Cargo.toml --all-targets --all-features`
  - `make check-layout check-docs`
  - `python3 tests/characterization/cli-product-surface/audit_registry.py --binary ./yai`
  - `git diff --check`
  - `awk '/^```bash$/{inside=1;next} /^```$/{inside=0} inside{print}' refoundation/foundation-recovery/interlock-02/MANUAL-ACCEPTANCE.md | bash -n`
- Exit: 0 for every command
- Raw output excerpt:

```text
check-no-old-roots: ok
check-required-layout: ok
check-source-placement: ok
check-source-surface-clean: ok
check-doc-links: ok (30 files)
{"handler_failures": 0, "help_failures": 0, "operation_count": 177, "registry_digest": "sha256:f238dd5b3edd416e0eefad8b33291d3898e300f95f6f631171d02ad8ca553760", "schema": "yai.cli.command_discovery.v1", "visibility_counts": {"advanced": 25, "compatibility": 16, "plumbing": 45, "product": 90, "removed": 1}}
warning: `yai-engine` (lib) generated 12 warnings
warning: `yai` (bin "yai") generated 13 warnings
Finished `dev` profile [unoptimized + debuginfo]
```

- Invariant: code formatting, the repository Clippy contract, docs/layout,
  compiled registry coverage, whitespace and the copy/paste acceptance script
  satisfy their existing contracts. Publication equality is recorded only in
  the post-commit handoff.

## I02-E09 — remote reference freshness before publication

- Run ID: `i02-remote-freshness-20260905`
- Order: 9
- Environment: read-only Git remote queries; temporary YVEX source checkout
- Commands:
  - `git ls-remote https://github.com/yailabs/yai.git refs/heads/master`
  - `git ls-remote --heads https://github.com/yailabs/yvex.git`
  - `git -C /tmp/yvex-i02-readonly diff --name-status 3a6520945a5c103365178f48104f0ccdb5154624..origin/models2 -- include/yvex/content.h include/yvex/provider.h docs/openai-compatibility.md`
- Exit: 0 for every command
- Raw output excerpt:

```text
5e75a803a610d6d6b1deb33693bc794f315294d8 refs/heads/master
5b95ee82eee394581521d106c7b1ec479d472448 refs/heads/main
5b95ee82eee394581521d106c7b1ec479d472448 refs/heads/models2
```

The exact `refs/heads/models1` query returned no row. The path-limited diff
also returned no row, meaning the three inspected public contract files are
unchanged between the start-time `models1` commit and current `models2`.

- Invariant: YAI remote truth remained at the reconciled baseline before
  publication; YVEX branch retirement/drift was observed rather than hidden,
  and no changing YVEX ABI was imported into I02.
