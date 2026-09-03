# Wave 19 failure evidence

## W19-F01 — sandbox loopback denial

- run ID: `W19-F01-20260903`
- order: 1
- cwd: `/home/mothx/computer-science/projects/YAI/yai`
- environment: default restricted sandbox
- command: `make smoke-memory-representation`
- exit: non-zero
- bounded stderr posture: the two CLI tests documented as requiring loopback
  sockets could not execute under the restricted network namespace
- resolution: rerun with explicit loopback permission; no product semantic change

## W19-F02 — stale index at provider-selection boundary

- run ID: `W19-F02-20260903`
- order: 2
- cwd: `/home/mothx/computer-science/projects/YAI/yai`
- environment: fresh fixture YAI_HOME, governed chat target and loopback encoder
- material pre-state: an index built before the next governed provider selection
- observed result: provider selection advanced Case generation before Projection;
  the exact index correctly reported stale and fuzzy planes were unavailable
- invariant demonstrated: stale generation was never silently accepted
- resolution: runtime refreshes that exact configured profile only through the
  separately qualified loopback encoder, seals/publishes it, then compiles the
  existing Projection path; encoder failure still degrades safely

## W19-F03 — Resource qualification rejected unrelated provider memory

- run ID: `W19-F03-20260903`
- order: 3
- cwd: `/home/mothx/computer-science/projects/YAI/yai`
- environment: fresh fixture YAI_HOME
- material pre-state: only a ProviderClaim existed; runtime query was anchored to
  `resource:w19-memory`
- bounded raw retrieval excerpt:

```text
"qualified_count":0
"resource_qualification":1
"selected":[]
```

- invariant demonstrated: similarity did not bypass Resource qualification
- resolution: the product fixture now performs one real governed write first;
  the later runtime retrieval selects its same-Resource finalized observed
  consequence and Decision, while unrelated ProviderClaims remain rejected

## W19-F04 — live external acceptance unavailable

- run ID: `W19-F04-20260903`
- order: 4
- cwd: `/home/mothx/computer-science/projects/YAI/yai`
- environment: operator environment inspection without rendering secret values
- exact pre-state:

```text
YAI_EXTERNAL_PROVIDER_BASE_URL=missing
YAI_EXTERNAL_PROVIDER_MODEL=missing
YAI_MEMORY_ENCODER_BASE_URL=missing
YAI_MEMORY_ENCODER_MODEL=missing
```

- exit/posture: no live command dispatched; `blocked_external_dependency`
- classification: `DEPLOYMENT_LIMITATION`, not a YAI or YVEX defect
- resolution: exact operator commands remain in `MANUAL-ACCEPTANCE.md`; final
  publication state is external-acceptance-pending

## W19-F05 — non-canonical Cargo target placement

- run ID: `W19-F05-20260903`
- order: 5
- cwd: `/home/mothx/computer-science/projects/YAI/yai`
- environment: Clippy invoked without the repository `CARGO_TARGET_DIR=target`
- material pre-state: no nested crate-local build directories
- exact later detection command: `make check-layout`
- exit: 2
- bounded raw output:

```text
/home/mothx/computer-science/projects/YAI/yai/cmd/yai/target/debug/build/aws-lc-sys-667adff7fa811392/out/flag_check.c
C files are only allowed under system/, cmd/yaid/, tests/ or vendor/
make: *** [Makefile:221: check-layout] Error 1
```

- invariant demonstrated: repository layout rejects generated C source outside
  the canonical build root
- resolution: remove only session-generated `cmd/yai/target` and
  `engine/target`; rerun both Clippy commands with `CARGO_TARGET_DIR=target`;
  `make check-layout` then exits 0

## W19-F06 — final characterization loopback denied in restricted run

- run ID: `W19-F06-20260903`
- order: 6
- cwd: `/home/mothx/computer-science/projects/YAI/yai`
- environment: default restricted sandbox after the definitive `make check`
- command: `make characterization`
- exit: 2
- bounded raw stderr:

```text
called `Result::unwrap()` on an `Err` value: Os { code: 1, kind: PermissionDenied, message: "Operation not permitted" }
failures:
    command_adapters::provider_transport::tests::redirects_and_ambiguous_http_framing_fail_closed
    command_adapters::provider_transport::tests::tls_validates_chain_and_hostname_without_downgrade
test result: FAILED. 26 passed; 2 failed; 2 ignored
make: *** [Makefile:371: build-rust] Error 101
```

- invariant demonstrated: these tests require opening loopback listeners and do
  not silently skip when the execution namespace refuses sockets
- resolution: rerun the exact `make characterization` target with explicit
  loopback permission; engine `227/2 ignored`, CLI `28/2 ignored`, every
  characterization through W19, and `memory_representation_characterization`
  then pass with exit 0; no source change was made between the two runs

## W19-F07 — secure-directory creation race

- run ID: `W19-F07-20260903`
- order: 7
- cwd: `/home/mothx/computer-science/projects/YAI/yai`
- environment: central Cargo target; eight independent test child processes
- command: `CARGO_TARGET_DIR=target cargo test --manifest-path engine/Cargo.toml -p yai-engine memory_index -- --nocapture`
- exit: 101
- material pre-state: read-path symlink hardening had just been added; no
  persisted product index
- bounded raw stderr:

```text
called `Result::unwrap()` on an `Err` value: "memory_index_directory_create_failed: File exists (os error 17)"
test memory_index::tests::concurrent_rebuild_child ... FAILED
test memory_index::tests::eight_process_concurrent_rebuilds_publish_one_equivalent_manifest ... FAILED
test result: FAILED. 12 passed; 1 failed; 2 ignored
```

- invariant demonstrated: concurrent builders can race while creating a shared
  secure directory before reaching the profile lock
- resolution: `AlreadyExists` is accepted only after `symlink_metadata` proves
  the raced path is a real directory; symlinks/non-directories remain rejected;
  the exact rerun passed 13 tests with two characterization tests ignored

## W19-F08 — automatic-profile scope validation asymmetry

- run ID: `W19-F08-20260903`
- order: 8
- cwd: `/home/mothx/computer-science/projects/YAI/yai`
- environment: pre-commit staged-diff inspection; no external state
- material pre-state: explicit-profile loads validated expected Tenant, Case,
  profile and pointer checksum; automatic profile discovery validated only the
  bundle's internal checksum before downstream qualification
- observed posture: no authority bypass was possible, but a valid foreign-scope
  bundle copied into the disposable tree was not rejected at the store boundary
- invariant demonstrated: internal content integrity is not a substitute for
  expected namespace identity
- resolution: publication locks now bind Tenant/Case/profile; list and automatic
  discovery validate pointer, directory hash, Tenant, Case, profile, index and
  checksum together; the focused 14-test pass includes a foreign-scope
  publication and corrupted-pointer regression
