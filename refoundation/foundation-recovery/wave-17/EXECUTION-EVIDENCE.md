# Wave 17 execution evidence

Evidence is copied from real command output from one causal run. The full
temporary store is `/tmp/yai-w17-product.HmKKzr/home`.

## W17-PRODUCT-01 — same-Tenant handoff

- run_id: `w17-product-HmKKzr`
- order: 1
- pre-state: fresh temporary YAI_HOME
- cwd: `/home/mothx/computer-science/projects/YAI/yai`
- environment: `YAI_HOME=/tmp/yai-w17-product.HmKKzr/home`; no external
  provider variables
- Principal: `principal:72cc156b82060120eac8f7e234dbfcef`
- Tenant: `tenant:w17-product`
- Cases: `case:w17-source`, `case:w17-target`
- exact command: `./yai case handoff offer case:w17-source --target
  case:w17-target --kind json --value '{"task":"inspect bounded material"}'
  --role operation-proposer`
- exit: 0
- raw stdout:

```text
handoff: offered
handoff_id: handoff:6e4d3ab6046cfa2f1acfef17b83c3eb4
source_case: case:w17-source
target_case: case:w17-target
request_digest: sha256:044856f060d19ac56b2f87dbfa72235284c09b935b657ee459b2fe901fba5099
```

- subsequent exact commands: `case handoff pending`, `accept`, `result`, then
  `reconcile --json`
- produced IDs:
  `handoff-acceptance:51e922e2a05f33815e0e8b5f44d67f26`,
  `handoff-result:1daa050a28edf995a2ad13abeb556e5a`,
  `handoff-reconciliation:8fc877acc2212e3d167181948207982a`
- bounded raw reconciliation output:

```json
{"schema":"yai.cli.result.v1","operation_id":"yai.case.handoff.reconcile","status":"ok","data":{"kind":"native_json","value":{"handoff_id":"handoff:6e4d3ab6046cfa2f1acfef17b83c3eb4","outcome":"succeeded","reconciliation_id":"handoff-reconciliation:8fc877acc2212e3d167181948207982a","result":{"kind":"json","value":"{\"finding\":\"bounded result\"}"},"target_acceptance_id":"handoff-acceptance:51e922e2a05f33815e0e8b5f44d67f26","target_result_id":"handoff-result:1daa050a28edf995a2ad13abeb556e5a"}}}
```

- invariant: bounded work data crossed one Tenant boundary between two separate
  Case histories; target authority was not imported.

## W17-PRODUCT-02 — human patch proposal and adoption

- run_id: `w17-product-HmKKzr`
- order: 2
- pre-state: source Case generation 4, exact Definition v2 bound, revision 0
- Definition: `workflow-definition:a032abd40d22c11229d443ba63c6c008`
- binding: `case-workflow-binding:ab0449a0bcfa69ddede93d383d0190df`
- exact command: `./yai workflow patch propose case:w17-source --file
  /tmp/yai-w17-product.HmKKzr/patch.json`
- exit: 0
- raw stdout:

```text
workflow_plan_patch: proposed
patch_id: workflow-plan-patch:7d95153c39a27ab6ed7fe4722233140e
base_revision: 0
base_topology_digest: sha256:fedfa279db2928926865504bf04c08297f4764016f28dd9497658a0478048f20
operations: 1
```

- validation command: `./yai workflow patch validate case:w17-source --patch
  workflow-plan-patch:7d95153c39a27ab6ed7fe4722233140e`
- validation exit: 0
- raw validation stdout:

```text
workflow_plan_patch: valid
patch_id: workflow-plan-patch:7d95153c39a27ab6ed7fe4722233140e
resulting_revision: 1
resulting_topology_digest: sha256:44d2ea55c1ea3452fe360a1c411f775f5a9d6042f7bba1b7116c1d06cbfa8a56
```

- adoption command: `./yai workflow patch adopt case:w17-source --patch
  workflow-plan-patch:7d95153c39a27ab6ed7fe4722233140e --json`
- adoption exit: 0
- produced amendment:
  `workflow-amendment:9e7e51beaa64b6f095b1c13475453812`
- post-state: Case generation 6, revision 1, amendment count 1, topology digest
  exactly `sha256:44d2ea55c1ea3452fe360a1c411f775f5a9d6042f7bba1b7116c1d06cbfa8a56`
- invariant: proposal did not alter topology; explicit owner adoption did, and
  status remained a read-only derived projection.

## W17-FOCUSED-01 — executable semantic characterization

- run_id: `w17-smoke-adaptive-workflow`
- order: 3
- pre-state: formatted Wave-17 candidate
- exact command: `make smoke-adaptive-workflow`
- exit: 0
- bounded raw stdout:

```text
w17_planpatch: proposals=2 adopted=1 stale=1 revision=1 topology_digest=sha256:11a6ed023a490c147d0b2d9d3ce544d7a77a5b7ae025bc384d180235d16dad5d
w17_model_patch: provider_results=2 malformed_candidates=0 valid_candidates=1 duplicate_candidates=0 auto_adoptions=0 owner_adoptions=1 patch_id=workflow-plan-patch:e64c69f9b191c7e21ae89f5110845d9b
w17_patch_race: contenders=8 winners=1 stale=7 amendments=1 revision=1
w17_subflow_progress: cases=1 definitions=2 qualified_nodes=2 work_items=0 completed=true digest=sha256:1407a22efca87a961ec2499aefc0d4566b145f0bb995d59a8c667de732cfccb9
w17_scale: amendments=32 amendment_bytes=28028 amendment_nodes=33 amendment_rebuild_us=15471 subflow_instances=128 expanded_nodes=512 expanded_edges=765 expanded_rebuild_us=42154
w17_handoff_chain: cases=4 edges=3 accepts=3 results=3 reconciliations=3 histories_replayed=4 process_owners=0 imported_grants=0
adaptive_workflow_characterization: pass
model_auto_adoptions: 0
handoff_workers_held_while_waiting: 0
cross_tenant_handoff: rejected
workflow_run_owner: 0
multi_case_process_owner: 0
```

- invariant: one-winner amendment, strict/idempotent model candidate,
  same-Case Subflow replay and worker-free authority-isolated Handoff all use
  canonical Case facts.

## W17-QUALIFICATION-01

- run_id: `w17-full-check`
- order: 4
- pre-state: Wave-17 candidate with product evidence complete
- exact command: `make check`
- cwd: `/home/mothx/computer-science/projects/YAI/yai`
- exit: 0
- bounded raw stdout:

```text
check-no-old-roots: ok
check-required-layout: ok
check-source-placement: ok
check-source-surface-clean: ok
check-doc-links: ok (29 files)
test result: ok. 166 passed; 0 failed
test result: ok. 22 passed; 0 failed
case_runtime:agentless_26_turn_provider_model_replacement ok
human_review:crash_r1_r6_recovery ok
policy_authority:allow_chain ok
```

- invariant: all lower-wave source/layout, engine, CLI, authority, runtime,
  review and product smoke contracts remain green.

## W17-QUALIFICATION-02

- run_id: `w17-full-characterization-escalated-local-socket`
- order: 5
- pre-state: identical candidate; sandbox Unix bind limitation identified
- exact command: `make characterization`
- environment: normal repository environment, executed outside the restricted
  socket sandbox only so the legacy local `yaid` tests could bind AF_UNIX
- exit: 0
- bounded raw stdout:

```text
daemon:started
ipc:status ok
daemon:shutdown ok
provider_model_vertical:real_http_invocation ok
case_runtime:agentless_26_turn_provider_model_replacement ok
cli_product_surface: porcelain_governed_workflow=pass
w17_patch_race: contenders=8 winners=1 stale=7 amendments=1 revision=1
w17_model_patch: provider_results=2 malformed_candidates=0 valid_candidates=1 duplicate_candidates=0 auto_adoptions=0 owner_adoptions=1
w17_handoff_terminal: source_cancel_before_accept=rejected target_cancel_after_accept=reconciled_cancelled target_results=0 histories_verified=4
adaptive_workflow_characterization: pass
workflow_run_owner: 0
multi_case_process_owner: 0
```

- invariant: complete real characterization including daemon IPC, provider
  fixture, product CLI and W17 adaptive composition passes.

## W17-QUALIFICATION-03

- run_id: `w17-format-lint-diff`
- order: 6
- pre-state: same fully characterized Wave-17 candidate
- exact commands: `cargo fmt --manifest-path engine/Cargo.toml --all --check`;
  `cargo fmt --manifest-path cmd/yai/Cargo.toml --all --check`;
  `cargo clippy --manifest-path engine/Cargo.toml --workspace --all-targets`;
  `cargo clippy --manifest-path cmd/yai/Cargo.toml --all-targets`;
  `git diff --check`
- exit: 0 for every command
- bounded raw stdout/stderr: both format and diff checks produced no output;
  both Clippy invocations finished the `dev` profile successfully and reported
  only the repository's admitted warning classes
- invariant: the candidate is formatted, lint-buildable under the current
  warning contract, and contains no whitespace errors.
