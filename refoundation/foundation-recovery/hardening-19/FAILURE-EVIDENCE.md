# H19 real failure evidence

## H19-F01 — W19 query-time full deep validation

- pre-state: published W19 physical v1 at `95730f1a...`
- source: retained W19 run in `wave-19/EXECUTION-EVIDENCE.md` and
  `wave-19/memory-scale.tsv`
- exact historical command: `cargo test --release --manifest-path
  engine/yai-engine/Cargo.toml memory_index::tests::memory_index_scale_characterization
  -- --ignored --nocapture`
- exit: 0
- bounded raw output:

```text
memory_scale entries=50000 representation_ms=7745 lexical_build_ms=13298 fixture_embedding_build_ms=9776 exact_query_us=43163 lexical_query_us=326008 hybrid_query_us=39411645 exact_hits=32 lexical_hits=32 hybrid_hits=32 serialized_bytes=226727138 peak_memory=not_observed ann=deferred
```

The historical loader called `bundle.validate()`, which rebuilt lexical state
and reserialized vector-bearing content after an unbounded `read_to_end`. H19
reproduced the structural source path directly with `git show 95730f1a...` and
replaced it with publish-time deep validation plus bounded sealed load
validation. Current 50k cold hybrid is 1,182,221 microseconds.

## H19-F02 — pathname TOCTOU and unbounded derived reads

- run: direct baseline source audit
- command: `git show
  95730f1a11025491a3f0f7ccec0aba8b1d3f036b:engine/yai-engine/src/memory_index.rs`
- exit: 0
- observed executable sequence:

```text
1627:        match fs::symlink_metadata(&current) {
1699:    let mut file = File::open(path)
1701:    file.read_to_end(&mut bytes)
1724:    fs::rename(&temporary, &target)
1788:    let lock_path = profile_directory.join("build.lock");
```

This was a real check-then-open pathname window and allocation-before-bound
path. H19 moved all derived operations to descriptor-relative Linux primitives
and checks `fstat` size before bounded reads.

## H19-F03 — corrupt index failed the Case runtime

During H19 review, `execute_hybrid_retrieval` propagated
`find_current_memory_index(...)?`; corrupt physical state therefore escaped the
derived plane and failed the full turn. The corrected path classifies load or
source error as `derived_index_unavailable:<reason>`, supplies no fuzzy index,
and runs qualified operational/exact retrieval. `status`/`verify` remain the
diagnostic surfaces.

## H19-F04 — sandbox-only regression attempt failure

- run ID: unified exec session 7050
- cwd: repository root
- exact command: `make build-rust && make smoke-memory-representation`
- exit: 2
- bounded raw stderr/test result:

```text
command_adapters::provider_transport::tests::redirects_and_ambiguous_http_framing_fail_closed ... FAILED
called `Result::unwrap()` on an `Err` value: Os { code: 1, kind: PermissionDenied, message: "Operation not permitted" }
command_adapters::provider_transport::tests::tls_validates_chain_and_hostname_without_downgrade ... FAILED
test result: FAILED. 31 passed; 2 failed; 2 ignored
```

The identical suite was rerun with loopback socket permission and passed. This
is retained as an execution-environment failure, not relabeled as a code defect.

## H19-F05 — live external dependency unavailable

- cwd: repository root
- command: non-secret presence check for all required variables
- exit: 0
- raw output:

```text
YAI_EXTERNAL_PROVIDER_BASE_URL=missing
YAI_EXTERNAL_PROVIDER_MODEL=missing
YAI_MEMORY_ENCODER_BASE_URL=missing
YAI_MEMORY_ENCODER_MODEL=missing
YAI_MEMORY_ENCODER_REVISION=missing
YAI_MEMORY_ENCODER_DIMENSION=missing
```

No live result was fabricated. Final publication posture remains external
acceptance pending.

## H19-F06 — generated target directory tripped layout guard

- run ID: `h19-static-20260903-01`
- cwd: repository root
- exact command: `make check-layout check-docs`
- exit: 2
- bounded raw output:

```text
cmd/yai/target/debug/build/aws-lc-sys-667adff7fa811392/out/flag_check.c
C files are only allowed under system/, cmd/yaid/, tests/ or vendor/
make: *** [Makefile:222: check-layout] Error 1
```

The preceding per-manifest Clippy invocation created disposable
`engine/target` and `cmd/yai/target` directories. They were removed explicitly;
`make check-layout check-docs` then passed. This is retained as qualification
environment evidence, not a product defect or a source-tree exception.
