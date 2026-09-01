# Wave 16 execution evidence

These blocks retain bounded, unedited excerpts from actual pre-publication
runs. Each block identifies its own run and does not combine output from
different executions.

## P16-01/P16-02 — product and advanced help

- evidence_id: `P16-HELP-01`
- run_id: `w16-product-20260901-OtGc1A`
- execution_order: 01–02
- pre-state: fresh temporary `YAI_HOME`; no mutation performed by help
- cwd: `/home/mothx/computer-science/projects/YAI/yai`
- environment: `YAI_HOME=/tmp/yai-cli-product-surface.OtGc1A/home`
- exact commands: `target/debug/yai`; `target/debug/yai help --advanced`
- exit: 0 for both

```text
YAI — governed operational AI runtime

Usage: yai <command> [arguments]

START
  init               Initialize local identity and Tenant state
  doctor             Diagnose whether this environment is ready

WORK
  case               Create, govern, run, and inspect Cases
  workflow           Define and bind deterministic progression
  review             Resolve authenticated human Reviews

GOVERN
  policy             Manage policy artifacts and lifecycle
  tenant             Inspect Tenant membership and scope
  identity           Inspect the authenticated Principal

RUNTIME
  runtime            Host and control bounded RuntimeInstance work

META
  help               Show product or advanced command discovery
  version            Show binary and CLI registry identity
  completion         Generate shell completion from the registry

Use `yai help --advanced` for engineering and compatibility tools.
Use `yai <command> --help` for exact syntax; add `--json` for machine output.
```

The advanced projection appended the following registry-derived groups:

```text
ADVANCED / PLUMBING / COMPATIBILITY
  carrier            plumbing
  case               advanced, product
  context            plumbing
  control            plumbing
  daemon             compatibility
  decision           plumbing
  effect             advanced
  engine             plumbing
  facts              plumbing
  graph              plumbing
  hot                compatibility
  info               compatibility
  journal            compatibility
  memory             compatibility, plumbing
  observe            plumbing, removed
  process            plumbing
  projection         plumbing
  prompt             advanced
  query              plumbing
  receipt            plumbing
  reconcile          plumbing
  runtime            advanced, product
  security           advanced
  store              plumbing
```

Invariant: default help is a compact product map while every engineering tool
remains discoverable; both projections come from the same registry.

## P16-03/P16-04/P16-21 — registry discovery and conformance

- evidence_id: `P16-REGISTRY-01`
- run_id: `w16-registry-audit-20260901-final`
- execution_order: 01–121
- pre-state: built candidate binary; no Case/store mutation
- cwd: `/home/mothx/computer-science/projects/YAI/yai`
- environment: ordinary repository environment
- exact command: `python3 tests/characterization/cli-product-surface/audit_registry.py --binary ./yai --inventory-tsv`
- exit: 0

```text
{"handler_failures": 0, "help_failures": 0, "operation_count": 121, "registry_digest": "sha256:8f40be2da9ab2d94f0ab832d6d687f206c2755d9134c70adcb5d015a889163d9", "schema": "yai.cli.command_discovery.v1", "visibility_counts": {"advanced": 9, "compatibility": 16, "plumbing": 45, "product": 50, "removed": 1}}
```

Invariant: all 121 canonical paths admit `--help`, and all non-removed handler
IDs are source-resolvable; discovery, paths, visibility and the compiled digest
come from one descriptor authority.

The same candidate binary returned the interface identity through the Product
version operation:

```json
{"schema":"yai.cli.result.v1","operation_id":"yai.meta.version","status":"ok","data":{"kind":"object","title":"YAI VERSION","fields":[{"name":"Binary","value":"0.0.0-newcore"},{"name":"CLI registry","value":"yai.cli.registry.v1"},{"name":"CaseState schema","value":"yai.case_state.v10"},{"name":"Registry digest","value":"sha256:8f40be2da9ab2d94f0ab832d6d687f206c2755d9134c70adcb5d015a889163d9"},{"name":"Transition schema","value":"yai.transition.v10"}]}}
```

Normal discovery contained the 12 registry-owned Product roots and 50 Product
operations; advanced discovery contained the same root metadata and all 121
operations.

## P16-05/P16-06 — truthful first use

- evidence_id: `P16-FIRST-USE-01`
- run_id: `w16-product-20260901-OtGc1A`
- execution_order: 03–06
- pre-state: empty `/tmp/yai-cli-product-surface.OtGc1A/home`
- cwd: `/home/mothx/computer-science/projects/YAI/yai`
- environment: same isolated `YAI_HOME`
- exact commands: `target/debug/yai doctor`;
  `target/debug/yai init --tenant tenant:cli-product --organization organization:cli-product`;
  `target/debug/yai doctor`; repeated `init --json`
- exit: 0 for every command

Before init:

```text
YAI DOCTOR
Posture            NOT_INITIALIZED
YAI_HOME           /tmp/yai-cli-product-surface.OtGc1A/home
Runtime layout     missing
Storage            missing
Storage backend    lmdb
Local identity     not_enrolled
RuntimeInstance    not_running
Legacy daemon      optional

Next run `yai init --tenant <TENANT> --organization <ORGANIZATION>`
```

After init:

```text
YAI DOCTOR
Posture            OK
YAI_HOME           /tmp/yai-cli-product-surface.OtGc1A/home
Runtime layout     ready
Storage            ready
Storage backend    lmdb
Local identity     enrolled
RuntimeInstance    not_running
Legacy daemon      optional
```

Invariant: doctor reflects owned prerequisites; init is scriptable and repeat
execution is idempotent rather than destructive.

## P16-07/P16-08 — Case before runtime

- evidence_id: `P16-NEVER-STARTED-01`
- run_id: `w16-product-20260901-OtGc1A`
- execution_order: 07–09
- pre-state: initialized home, no Case/runtime checkpoint
- Tenant: `tenant:cli-product`
- Case: `case:cli-product`
- exact commands: `yai case create case:cli-product --tenant tenant:cli-product`;
  `yai case show case:cli-product`; `yai case stop case:cli-product`
- exit: 0 for all

```text
CASE
ID         case:cli-product
Tenant     tenant:cli-product
Lifecycle  open
Generation 0

EXECUTION
Posture never_started
```

```text
CASE EXECUTION
Case    case:cli-product
Posture no_active_execution
```

Invariant: canonical Case truth is available before operational state; an
absent checkpoint is not an OS error and stop is clean/idempotent.

## P16-09/P16-10 — deliberate Participant/provider/Resource onboarding

- evidence_id: `P16-ONBOARDING-01`
- run_id: `w16-product-20260901-OtGc1A`
- execution_order: 10–15
- pre-state: open Case, no Participant/provider/Resource
- Case: `case:cli-product`
- Participant: `participant:model`
- provider/model: `provider:openai-compatible` / `fixture-model`
- Resource: `resource:workspace`
- exact commands: two `case participant role add` commands; `case participant list`;
  `case provider attach ... --endpoint http://127.0.0.1:9/v1/chat/completions --model fixture-model`;
  `case resource attach filesystem ... --resource resource:workspace`;
  `case resource list`
- exit: 0 for every command

```text
PARTICIPANTS
PARTICIPANT        ROLES
participant:model  model-executor,operation-proposer

RESOURCES
RESOURCE            KIND        POLICY
resource:workspace  filesystem  policy:filesystem-prefix:resource:workspace
```

Invariant: onboarding is discoverable and explicit; Participant is not
Principal, roles are not granted implicitly, and the provider boundary uses
only endpoint plus model identity.

## P16-11/P16-13/P16-14/P16-15 — porcelain-only governed Workflow

- evidence_id: `P16-PORCELAIN-E2E-01`
- run_id: `w16-product-20260901-OtGc1A`
- execution_order: 16–28
- pre-state: open governed Case with Participant/provider/Resource attached
- cwd: `/home/mothx/computer-science/projects/YAI/yai`
- environment: isolated home; one bounded RuntimeInstance worker
- Tenant: `tenant:cli-product`
- Case: `case:cli-product`
- Policy: `policy-artifact:99b991440979d06fb28d6bf04911ca128e582bbd18204935711af93827a186d2`
- WorkflowDefinition: `workflow-definition:35785abefd03599bb8e2e3b8bfd9cfd0`
- WorkflowBinding: `case-workflow-binding:f29c60aa09638bdd6e849823e0353f07`
- execution: `workflow-execution:83922d2af375d38c`
- exact command family: Product `policy ingest/validate/publish`, `case policy bind`,
  `workflow define/bind/status`, `runtime serve/stop`, and `case show --json`
- exit: 0

```text
workflow_definition_id: workflow-definition:35785abefd03599bb8e2e3b8bfd9cfd0
workflow_binding_id: case-workflow-binding:f29c60aa09638bdd6e849823e0353f07
case_id: case:cli-product
case_generation: 15
completed: true
satisfied: 1
active: 0
waiting: 0
skipped: 0
ready: 0
node: apply-change kind=deterministic_work posture=Satisfied reason=canonical_satisfaction_recorded execution=workflow-execution:83922d2af375d38c
```

```json
{"schema":"yai.cli.result.v1","operation_id":"yai.case.show","status":"ok","data":{"kind":"case","case":{"case_id":"case:cli-product","tenant_id":"tenant:cli-product","generation":15,"lifecycle":"open","execution":"completed","participants":[{"participant_id":"participant:model","roles":["model-executor","operation-proposer"]}],"provider":{"participant_id":"participant:model","provider_id":"provider:openai-compatible","provider_kind":"openai_compatible","endpoint":"http://127.0.0.1:9/v1/chat/completions","model_id":"fixture-model"},"resources":[{"resource_id":"resource:workspace","kind":"filesystem","policy_id":"policy:filesystem-prefix:resource:workspace","policy_owner_participant_id":"participant:model","review_requirement":"automatic"}],"workflow":{"definition_id":"workflow-definition:35785abefd03599bb8e2e3b8bfd9cfd0","binding_id":"case-workflow-binding:f29c60aa09638bdd6e849823e0353f07","completed":true,"satisfied":1,"skipped":0,"active":0,"waiting":0,"ready_nodes":[]},"policy_bindings":1,"pending_reviews":0,"unresolved_effects":0,"finalized_effects":1}}}
```

Invariant: the complete governed filesystem Effect used only Product commands;
no store/journal/projection/graph/facts/carrier/direct-effect/raw-WorkItem path
was invoked. The deterministic Workflow made `provider_invocations=0`, wrote
the admitted file and finalized exactly one Effect.

## P16-12 — Product Review park and resume

- evidence_id: `P16-REVIEW-01`
- run_id: `w16-review-20260901-64fad196`
- execution_order: 01–05
- pre-state: deterministic Workflow execution parked on the existing Review
  owner; worker released; no physical Effect yet
- cwd: `/home/mothx/computer-science/projects/YAI/yai`
- environment: `YAI_EXECUTION_EVIDENCE=1`; isolated temporary `YAI_HOME`
- Tenant/Case: `tenant:wave15-review` / `case:wave15-review`
- WorkflowDefinition: `workflow-definition:5f7f1c041ea039b62e1a40abc2058355`
- WorkflowBinding: `case-workflow-binding:1cf21b6445742d5e52b3b636f4d4abc1`
- Review: `review:64fad1963d6d5358fd1577603bd4617a`
- exact command: `YAI_EXECUTION_EVIDENCE=1 tests/characterization/workflow-kernel/test_workflow_review.sh`
- exit: 0

```text
state: WaitingReview
attempt_count: 1
worker_id: none
stop_reason: awaiting_review: workflow deterministic operation awaiting existing Review; run_id=case-run:c6aaaca80864483d; case_id=case:wave15-review
review_action: committed
review_id: review:64fad1963d6d5358fd1577603bd4617a
reviewer_participant: participant:operator
action: approve
execution_grant: none_review_command_never_executes
external_effect: none
completed: true
satisfied: 1
node: reviewed-write kind=deterministic_work posture=Satisfied reason=canonical_satisfaction_recorded execution=workflow-execution:9bb1f0e7a5971582
state: Completed
attempt_count: 2
worker_released_while_waiting_review: true
provider_invocations: 0
review_owner_reused: yai.review
```

Invariant: the Product `review approve` surface authenticates and records only
the existing ReviewAction; it creates no Grant or Effect itself. The same
Workflow execution resumes through the existing runtime and authority path.

## P16-18/P16-19/P16-20 — output and error boundaries

- evidence_id: `P16-OUTPUT-01`
- run_id: `w16-product-20260901-OtGc1A`
- execution_order: 29–33
- pre-state: completed Product Case above
- exact commands: `NO_COLOR=1 yai case show case:cli-product`;
  mistyped `yai case shwo ...`; removed `yai observe process ...`;
  `yai case show case:missing --json`
- exits: 0, 2, 2, 3 respectively

```text
yai: unknown command: case shwo case:cli-product
hint: did you mean `yai case show`?
```

```text
yai: command `yai observe process` was removed
hint: use `yai process observe`
```

```json
{"schema":"yai.cli.error.v1","operation_id":"yai.case.show","status":"error","code":"case_not_found_or_not_visible","message":"case_not_visible"}
```

Invariant: NO_COLOR output contains no escape bytes; unknown and removed paths
do not mutate state; machine errors contain no human hint lines or ANSI.

## Qualification — complete local regression

- evidence_id: `W16-QUALIFICATION-01`
- run_id: `w16-full-qualification-20260901`
- execution_order: suite order owned by Makefile
- pre-state: W16 candidate worktree, isolated test homes/stores
- cwd: `/home/mothx/computer-science/projects/YAI/yai`
- exact commands: `make check`; `make characterization`;
  `make smoke-cli-product-surface`; `cargo fmt --all --check` in engine and CLI;
  repository-contract Clippy; docs/layout check; `git diff --check`
- exits: 0 for all repository-contract commands

```text
test result: ok. 154 passed; 0 failed; 0 ignored
test result: ok. 22 passed; 0 failed; 0 ignored
cli_product_surface: registry_help=pass
cli_product_surface: first_use=pass
cli_product_surface: never_started_case=pass
cli_product_surface: participant_provider_resource=pass
cli_product_surface: porcelain_governed_workflow=pass
cli_product_surface: provider_invocations=0
cli_product_surface: json_no_color_errors=pass
```

Invariant: H10 authority, W11 time/cancel, W12 Tenant, W13/H13 runtime,
W14/H14 resources, W15/H15 Workflow, endurance and characterization remain
green. Plain Clippy passes under the repository's existing warning contract;
the pre-existing warnings are not misreported as newly eliminated.

## YVEX black-box qualification

- evidence_id: `W16-YVEX-01`
- run_id: `w16-provider-precondition-20260901`
- execution_order: 01
- pre-state: W16 worktree based on
  `67fbf2a96b924f596caf0bc74976e402eda34bc6`
- exact command: `env | rg '^YAI_EXTERNAL_PROVIDER_(BASE_URL|MODEL|TIMEOUT)='`
- exit: 1 (no matching configuration)
- endpoint: not supplied
- model: not supplied
- state: `blocked_external_dependency`

Invariant: no live provider result is fabricated. The read-only YVEX CLI
architecture reference is not counted as black-box provider qualification and
establishes no external provider finding.
