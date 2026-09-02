# H18 execution evidence

Evidence excerpts below are copied from the identified runs. They are bounded;
different run IDs are not combined into one causal proof.

## H18-RUN-001 — repository pre-state

- order: 1
- cwd: repository root
- Principal/Tenant/Case/target: not applicable; read-only Git inspection
- command: `git branch --show-current`, `git rev-parse HEAD`, `git rev-parse origin/master`, `git status --short`, `git diff --stat`, `git diff --cached --stat`
- exit: 0
- pre-state: published W18 final authored as `4b41b814`; actual clean master inspected before edits

```text
master
47986c56c6cd71b91a97efe29b31905ebf93ad41
47986c56c6cd71b91a97efe29b31905ebf93ad41
```

Invariant: H18 began from a clean, published, runtime-semantically W18 baseline.

## H18-RUN-002 — full Rust engine and CLI

- order: 2
- cwd: repository root
- environment: local fixtures only; no external provider variables
- command: `make smoke-provider-governance-hardening`
- exit: first sandbox run 2 (`EPERM` opening loopback); authorized loopback run reached a test-harness assertion, after which the corrected characterization script passed independently

```text
test result: ok. 210 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 84.06s
test result: ok. 26 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.05s
h18_dns_rebinding: host=localhost declared=remote result=not_dispatched request_bytes=0
h18_http_boundary: redirect_followed=false credential_forwarded=false duplicate_content_length=response_invalid
h18_tls: valid_ca_hostname=accepted wrong_hostname=not_dispatched unknown_ca=not_dispatched insecure_downgrade=false
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 25 filtered out; finished in 0.03s
```

Invariant: complete engine, CLI and TLS/DNS unit contracts were green; ignored
socket delivery tests were executed separately in H18-RUN-004.

## H18-RUN-003 — hardening product characterization

- order: 3
- cwd: repository root
- environment: isolated `YAI_HOME`; synthetic loopback provider; credential in process environment only
- Principal: local bootstrap Tenant Owner
- Tenant: `tenant:h18-smoke`
- ProviderTarget: `provider-target:df3dab2edbc6a948bbcc66ece14afc6a`
- command: `tests/characterization/provider-governance-hardening/test_provider_governance_hardening.sh`
- exit: 0

```text
h18_projection_rebuild: qualification=provider-qualification:b47e0094e46380f27b494f082b633a8a trust=provider-trust-event:aa4c6b50ae01fa5f2457f72177f160eb derived_copies_corrupt=true replay=exact
h18_governance_corruption: qualification_capability_forgery=fail_closed missing_trust_sequence=fail_closed restore_exact=true
h18_qualification_time: expires=1788364405000 boundary=exclusive effective_floor=1788364405001 rollback_resurrection=false
h18_credential_rotation: old_qualification=provider-qualification:bd253d14cde5c276a40e0e1fa02b702f revision=1 current_after_rotation=none requalified=provider-qualification:d0e62141284741d5528a511664ba2f4d secret_persisted=false
h18_half_open: contenders=64 admitted=1 epoch=1 success_closed=true worker_held=false
h18_selector_compatibility: historical_version=yai.provider_selector.v1 historical_choice=provider-target:a70add1494e724eb7378192568e3901a future_unknown=fail_closed generic_429_retry_safe=false
provider_governance_hardening_characterization: pass
target_id: provider-target:df3dab2edbc6a948bbcc66ece14afc6a
credential_revision: 1
old_qualification_invalidated: true
requalification_required: true
secret_persisted_or_rendered: false
qualification_before: provider-qualification:e571a436ff0e4310c6b1c650aae6cd3f
qualification_after: provider-qualification:04f8de1d9a351ede3d973a35cf60757c
```

Invariant: real registry-backed CLI rotation invalidated qualification without
persisting/rendering the secret, and requalification created a new exact result.

## H18-RUN-004 — W18 delivery regression

- order: 4
- cwd: `cmd/yai`
- environment: loopback fixtures
- command: `cargo test --locked wave18_ -- --ignored --nocapture`
- exit: 0

```text
running 2 tests
test command_adapters::provider::tests::wave18_connect_refused_is_provably_not_dispatched ... ok
test command_adapters::provider::tests::wave18_accepted_request_then_drop_is_delivery_indeterminate ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 26 filtered out; finished in 0.00s
```

Invariant: pre-dispatch and indeterminate-delivery behavior survived the shared
TLS transport change.

## H18-RUN-005 — full-bound scale

- order: 5
- cwd: `engine`
- command: full `yai-engine` suite, test `h18_full_provider_bounds_and_thousand_selection_endurance`
- exit: 0

```text
h18_provider_endurance: targets=128 target_129=rejected candidates=32 candidate_33=rejected selections=1000 min_us=32929 max_us=134253 mean_us=83285 db_bytes=15310848 deterministic=true
```

Invariant: exact admitted bounds and deterministic selection held under 1,000
canonical selections.

The same focused run also produced the independent-process proof:

```text
h18_process_concurrency: trust_processes=64 trust_commits=35 trust_sequence_contiguous=true probe_processes=64 probe_winners=1 selection_processes=64 selection_winners=1 duplicate_network_work=false
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 196 filtered out; finished in 85.00s
```

## YVEX external qualification

`YAI_EXTERNAL_PROVIDER_BASE_URL=missing` and
`YAI_EXTERNAL_PROVIDER_MODEL=missing`; state is
`blocked_external_dependency`. No YVEX CLI or administration was attempted.

## H18-RUN-006 — complete repository qualification

- order: 6
- cwd: repository root
- environment: authorized local Unix/TCP/TLS fixtures; external provider variables absent
- command: `make check`
- exit: 0
- pre-state: corrected H18 source and semantic-continuity safety characterization; publication pending

```text
check-no-old-roots: ok
check-required-layout: ok
check-source-placement: ok
check-source-surface-clean: ok
doc_root_canon: ok
check-doc-links: ok (30 files)
test result: ok. 212 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 86.74s
test result: ok. 26 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.06s
semantic_continuity:unsafe_continuation_retry_refused_and_restart ok
h18_process_concurrency: trust_processes=64 trust_commits=42 trust_sequence_contiguous=true probe_processes=64 probe_winners=1 selection_processes=64 selection_winners=1 duplicate_network_work=false
h18_tls: valid_ca_hostname=accepted wrong_hostname=not_dispatched unknown_ca=not_dispatched insecure_downgrade=false
provider_governance_hardening_characterization: pass
credential_revision: 1
old_qualification_invalidated: true
requalification_required: true
secret_persisted_or_rendered: false
```

Invariant: all engine, CLI, registry, lower-wave, adaptive, provider-governance
and H18 product smokes completed in one causal run. The two unit tests ignored
by default are the socket cases retained separately in H18-RUN-004 and the
provider smoke.

## H18-RUN-007 — complete characterization

- order: 7
- cwd: repository root
- environment: authorized local Unix/TCP/TLS fixtures; external provider variables absent
- command: `make characterization`
- exit: 0
- pre-state: H18-RUN-006 green; publication pending

```text
test result: ok. 212 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 87.97s
test result: ok. 26 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.05s
h18_process_concurrency: trust_processes=64 trust_commits=35 trust_sequence_contiguous=true probe_processes=64 probe_winners=1 selection_processes=64 selection_winners=1 duplicate_network_work=false
h18_dns_rebinding: host=localhost declared=remote result=not_dispatched request_bytes=0
h18_http_boundary: redirect_followed=false credential_forwarded=false duplicate_content_length=response_invalid
h18_tls: valid_ca_hostname=accepted wrong_hostname=not_dispatched unknown_ca=not_dispatched insecure_downgrade=false
provider_governance_hardening_characterization: pass
credential_revision: 1
old_qualification_invalidated: true
requalification_required: true
secret_persisted_or_rendered: false
```

Invariant: the complete product/endurance characterization remained green in
an independent run, including lower-wave replay, provider continuity, exact
delivery boundaries and H18 scale/concurrency proofs.

## H18-RUN-008 — formatting, lint and repository integrity

- order: 8
- cwd: repository root
- environment: no network or provider
- commands: `cargo fmt --manifest-path engine/Cargo.toml --all -- --check`; `cargo fmt --manifest-path cmd/yai/Cargo.toml --all -- --check`; Clippy for both manifests under the repository warning contract; `make check-layout check-docs`; `git diff --check`
- exit: 0 for every command

```text
warning: `yai-engine` (lib) generated 12 warnings
warning: `yai` (bin "yai") generated 13 warnings
check-no-old-roots: ok
check-required-layout: ok
check-source-placement: ok
check-source-surface-clean: ok
doc_root_canon: ok
check-doc-canonical-location: ok
check-doc-required-files: ok
check-doc-links: ok (30 files)
check-repository-identity: ok
```

Both format checks and `git diff --check` emitted no output. Clippy completed
successfully with the repository's 12 engine and 13 CLI pre-existing warning
classes; H18 does not claim a warning-free repository.
