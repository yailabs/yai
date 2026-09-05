# I04 execution evidence

All commands ran from
`/home/mothx/computer-science/projects/YAI/yai` on `master`. Output below is a
bounded unedited excerpt from the named run. Runs remain separate causal
proofs; no provider output has been reconstructed.

## I04-E01 — baseline reconciliation

- Run ID: `i04-baseline-20260905`
- Order: 1
- Material pre-state: published I03; no I04 changes; clean worktree
- Environment: repository worktree; no YAI_HOME
- Commands:
  - `git branch --show-current`
  - `git rev-parse HEAD`
  - `git rev-parse origin/master`
  - `git ls-remote origin refs/heads/master`
- Exit: 0
- Raw output:

```text
master
e1787b7950ec3ac8cbdff97d04224023483120ee
e1787b7950ec3ac8cbdff97d04224023483120ee
e1787b7950ec3ac8cbdff97d04224023483120ee refs/heads/master
```

- Invariant: local, tracked and published remote master were the exact I03
  baseline before mutation.

## I04-E02 — direct legacy archaeology

- Run ID: `i04-legacy-archaeology-20260905`
- Order: 2
- Material pre-state: current I01-I03 owners mapped; `yai-dev` read-only
- Commands:
  - `git -C ../yai-dev log --all --oneline -- src/orchestrator/planning/conversation.c src/orchestrator/composition src/models/audio src/models/multimodal`
  - bounded `sed` reads of the matching implementations, headers and consumers
- Exit: 0
- Raw evidence excerpt:

```text
17e566b93 Implement local execution core: agent reasoning loop, planning pipeline, response generation, conversation engine, production agent registry
c95acdb3d Implement media processing pipeline: audio transcription, image processing, video extraction, PDF parsing, content classification, model requirements
e9ad7f498 Implement production composition system: validation, templates, error handling, metrics, and CLI commands
```

- Invariant: legacy composition/validation/multimodal planes were stubs or
  Agent-shaped string/name inference. Only explicit source/provider/result
  lineage survived the owner test and is already represented by current Case,
  I02/I03 and `ConversationDerivedContent` owners.

## I04-E03 — focused composition qualification

- Run ID: `i04-composition-focused-final-20260906`
- Order: 3
- Material pre-state: fresh temporary YAI_HOME; six deterministic loopback
  OpenAI-compatible provider targets; misleading model/provider names; no
  credentials or external network
- Command: `tests/characterization/cognitive-execution-composition/test_cognitive_execution_composition.sh`
- Exit: 0
- Raw output excerpt:

```text
i04_composed: auxiliary_dispatch_once=true restart_resume=true primary_dispatch=true turn_immutable=true audio_and_image=true
i04_direct: primary_native_audio=true auxiliary_bypassed=true explicit_intent_preserved=true
i04_failure: delivery_indeterminate_blocks_primary=true malformed_blocks_primary=true normalization_blocks_primary=true unresolved_blocks_primary=true derived_duplicates=0
i04_revalidation: binding_evidence_and_qualification_change_replans=true historical_derivation_preserved=true
```

- Produced identities (bounded excerpt):

```text
request_id=cognitive-composition:sha256:32a8dcc310199ddb6628a21d7a8c7ccb3fb42b5c9305b0b5d285a2c5ec9946ec
prerequisite_plan_id=cognitive-plan:bec0d858f25eb7a09309b78ca25e954e
prerequisite_result_id=provider-result:case:i04-composed:model-output-3
derived_content_id=conversation-derived:sha256:b077f1ce662bd255969d4e97cd6d4ca00b4a2130635949857c249d69454e62b2
source_closure_id=cognitive-source-closure:sha256:ef8b5d3e2586558046cda5e72ce4a6b8ec49ac4bcede9ae2c4c283d22011d657
primary_plan_id=cognitive-plan:18417c7955376156226584ba51fa302e
primary_result_id=provider-result:case:i04-composed:model-output-6
```

- Invariant: real I02 planning and I03 exact-target transport executed the
  direct and composed paths. Restart reused the exact canonical prerequisite;
  ordered audio and repeated-image closure semantics survived; target evidence
  changes forced replanning; all dependency failures stopped primary dispatch.

## I04-E04 — zero-to-use-case manual acceptance

- Run ID: `i04-manual-acceptance-20260906`
- Order: 4
- Material pre-state: fresh `/tmp/yai-i04-manual.*` YAI_HOME, two disposable
  loopback providers, repository `./yai` launcher
- Command: `awk '/^```bash$/{inside=1;next} /^```$/{inside=0;next} inside{print}' refoundation/foundation-recovery/interlock-04/MANUAL-ACCEPTANCE.md | bash`
- Exit: 0
- Raw output excerpt:

```text
case_id: case:i04-manual
provider_mode: governed_pool
candidate_count: 2
last_attempt_posture: ResultReceived
delivery_indeterminate: false
I04 manual acceptance completed; disposable state removed
```

- Invariant: ordinary `./yai` commands built a Case from zero, committed an
  ordered text/audio/text Turn, stopped after prerequisite publication,
  resumed through a fresh process without auxiliary redispatch, completed the
  primary, re-inspected canonical identities, exercised missing-intent refusal
  and cleaned only disposable state.

## I04-E05 — manual script syntax

- Run ID: `i04-manual-syntax-20260906`
- Order: 5
- Material pre-state: published manual artifact
- Command: `awk '/^```bash$/{inside=1;next} /^```$/{inside=0;next} inside{print}' refoundation/foundation-recovery/interlock-04/MANUAL-ACCEPTANCE.md | bash -n`
- Exit: 0
- Raw output: empty
- Invariant: the copy/paste command blocks are valid Bash and contain no hidden
  `set -e`, `set -u` or `pipefail` contract.

## I04-E06 — complete repository check

- Run ID: `i04-make-check-final-20260906`
- Order: 6
- Material pre-state: final I04 executable source; repository loopback/process
  qualification permission; generated subproject Cargo targets removed
- Command: `make check`
- Exit: 0
- Raw output excerpt:

```text
check-no-old-roots: ok
check-required-layout: ok
check-source-placement: ok
check-doc-links: ok (30 files)
test result: ok. 286 passed; 0 failed; 4 ignored
test result: ok. 36 passed; 0 failed; 4 ignored
i04_composed: auxiliary_dispatch_once=true restart_resume=true primary_dispatch=true turn_immutable=true audio_and_image=true
i04_direct: primary_native_audio=true auxiliary_bypassed=true explicit_intent_preserved=true
i04_failure: delivery_indeterminate_blocks_primary=true malformed_blocks_primary=true normalization_blocks_primary=true unresolved_blocks_primary=true derived_duplicates=0
i04_revalidation: binding_evidence_and_qualification_change_replans=true historical_derivation_preserved=true
```

- Invariant: build, layout, docs, registry and every lower-wave smoke through
  I03 plus the complete I04 composition/adversarial matrix passed against the
  final implementation.

## I04-E07 — complete characterization

- Run ID: `i04-characterization-final-20260906`
- Order: 7
- Material pre-state: final I04 executable source; isolated stores and local
  deterministic providers
- Command: `make characterization`
- Exit: 0
- Raw output excerpt:

```text
test result: ok. 286 passed; 0 failed; 4 ignored
test result: ok. 36 passed; 0 failed; 4 ignored
provider_model_vertical:real_http_invocation ok
semantic_continuity:provider_replacement ok
memory_index_hardening: pass
episodic_semantic_memory: pass
multipart_conversation: pass
turn_commit_before_provider: pass
provider_failure_preserves_turn: pass
```

- Invariant: existing authority, provider, Case runtime, Workflow, H19/W20,
  I01 conversation, I02 planning and I03 realization behavior remained
  characterized after I04.

## I04-E08 — static, registry and artifact contracts

- Run ID: `i04-static-final-20260906`
- Order: 8
- Material pre-state: final executable source and evidence files
- Commands:
  - `cargo fmt --manifest-path engine/Cargo.toml --all -- --check`
  - `cargo fmt --manifest-path cmd/yai/Cargo.toml -- --check`
  - `env CARGO_TARGET_DIR=target cargo clippy --manifest-path engine/Cargo.toml --workspace --all-targets`
  - `env CARGO_TARGET_DIR=target cargo clippy --manifest-path cmd/yai/Cargo.toml --all-targets`
  - `make check-layout check-docs`
  - `python3 tests/characterization/cli-product-surface/audit_registry.py --binary ./yai`
  - `bash -n tests/characterization/cognitive-execution-composition/test_cognitive_execution_composition.sh`
  - `git diff --check`
  - bounded TSV column validation over `interlock-04/*.tsv`
- Exit: 0 for every command
- Raw output excerpt:

```text
check-source-surface-clean: ok
check-doc-links: ok (30 files)
{"handler_failures": 0, "help_failures": 0, "operation_count": 180, "registry_digest": "sha256:293a37e343657797f0051266e3ff32dd75c21bf967687a39207e201a32ba4075", "visibility_counts": {"advanced": 28, "compatibility": 16, "plumbing": 45, "product": 90, "removed": 1}}
```

Clippy exited zero with the repository's existing advisory warnings in
unrelated historical functions. `README.md` remained untouched.

## I04-E09 — external YVEX and pre-publication remote closure

- Run ID: `i04-external-closure-20260906`
- Order: 9
- Material pre-state: implementation/evidence complete; YVEX read-only; no
  operator endpoint/model variables
- Commands:
  - `git ls-remote https://github.com/yailabs/yvex.git refs/heads/main refs/heads/models1`
  - `git ls-remote origin refs/heads/master`
- Exit: 0
- Raw output:

```text
3f4a1c182d35e5a0e163adb81008ae7a366efcc6 refs/heads/main
e1787b7950ec3ac8cbdff97d04224023483120ee refs/heads/master
live_yvex_inputs=not_available
```

- Invariant: public YVEX main remained at the observed prompt SHA and exposed
  no public `models1` ref. Live YVEX composition was unavailable and is not
  claimed; generic loopback realization proves the YAI-owned boundary. Remote
  YAI master had not diverged before publication.
