# ZERO-TO-CURRENT — Golden Case operator acceptance

Authority: cumulative human-executable acceptance of the current YAI product.
This is the single evolving runbook, not a frozen wave transcript. Implementation
waves must update it when the supported lifecycle changes. Frozen execution
evidence belongs in the corresponding evidence package.

Preparation status: executable local Product path qualified; real-provider and
human acceptance remain separate, unclaimed results.
`HUMAN_GOLDEN_CASE = PENDING_OPERATOR`. Only an operator may report a human
PASS, with the exact YAI SHA, endpoint and provider-exposed model identity.
A previous PASS is not silently inherited by a changed product HEAD.

## Infrastructure, not the Case workflow

Use an unprivileged Linux x86_64 host with Landlock ABI 6 or newer, seccomp and
a root-owned `/usr/bin/python3`. The deliberately confined reference test runner
cannot write files, create children or access the network. Unsupported hosts
refuse execution; they do not receive an unconfined fallback. Install repository
build dependencies first. All commands below start in the YAI repository root.

YVEX must already expose the operator's exact DeepSeek target through its public
OpenAI-compatible endpoint. YAI does not start YVEX, load a model or select the
first model in its catalog. Native function calls, correlated tool feedback and
JSON-object output must pass actual qualification for the complete reference
Workflow. A text-only public endpoint is insufficient. Record that limitation;
do not substitute a fake model or use a private protocol.

In Terminal A, start the **reference resources**, not a model fixture:

```sh
python3 tests/cases/04-golden/world.py serve --port 18240
```

Leave Terminal A running. It serves the ordinary release-status HTTP endpoint
and the MCP specification/risk tool on separate exact scopes. The SQLite
database and source workspace are prepared next. These deterministic resources
are the software problem's environment; the human acceptance model is real.

In Terminal B:

```sh
make build-rust
mkdir /tmp/yai-golden-human
export YAI_HOME=/tmp/yai-golden-human/home
python3 tests/cases/04-golden/world.py prepare --root /tmp/yai-golden-human/world --endpoint http://127.0.0.1:18240
git rev-parse HEAD
./yai init --tenant tenant:golden --organization organization:golden
./yai case create case:golden:free --tenant tenant:golden
./yai case create case:golden:workflow --tenant tenant:golden
./yai case create case:golden:isolation --tenant tenant:golden
```

If the directory already exists, STOP: preserve the previous run. Choose a new
explicit directory and substitute it consistently below. `world.py prepare`
refuses an existing world and does not initialize YAI or run the lifecycle.

Initial identity setup is administration, not model work. One local authenticated
Principal has one Participant association per Case. The human operator holds
the reviewer role; the separate model Participant does not inherit that link.

```sh
for case_id in case:golden:free case:golden:workflow case:golden:isolation; do
  ./yai case participant role add "$case_id" --participant participant:operator --role operation-proposer
  ./yai case participant role add "$case_id" --participant participant:operator --role operation-reviewer
  ./yai case participant role add "$case_id" --participant participant:operator --role workflow-input
  ./yai case participant role add "$case_id" --participant participant:model --role model-executor
  ./yai case participant role add "$case_id" --participant participant:model --role operation-proposer
  ./yai case participant link-principal "$case_id" --principal self --participant participant:operator
  ./yai case participant view admit "$case_id" --participant participant:model --consumer model --view model_context
done
./yai case workbench case:golden:free --participant participant:operator --executor participant:model
```

## Free Case: remain inside YAI

The following are workbench actions, **not shell commands**. REPLAI handles the
editor. Paths are literal; the workbench does not expand shell variables or
infer attachments from ordinary conversation text.

```text
/case
/participants
/history
/policy
/attach /tmp/yai-golden-human/world/free/attachments/workspace.json
/attach /tmp/yai-golden-human/world/free/attachments/runner.json
/attach /tmp/yai-golden-human/world/free/attachments/database.json
/attach /tmp/yai-golden-human/world/free/attachments/service.json
/attach /tmp/yai-golden-human/world/free/attachments/mcp.json
/attach /tmp/yai-golden-human/world/free/attachments/discovery.json
/policy publish tests/cases/04-golden/policy.json publish reviewed Golden release policy deck
/policy
/discover resource:discovery issue
```

Before policy publication the Case is not READY. Attachment alone grants no
operation permission. After publication inspect READY and contributing source,
artifact and binding identities. Copy the `digest` of the exact `issue/issue.md`
candidate in the discovery result's `entries` into this admission action. Do not
use the enclosing observation's `integrity_digest`:

```text
/admit resource:discovery CANDIDATE_DIGEST issue/issue.md
/artifacts
```

Use the exact **public endpoint and exposed DeepSeek identity supplied by the
YVEX operator**, replacing the two uppercase placeholders below. This performs
real mechanical probes, your explicit trust approval and an explicitly
operator-attested semantic binding. It does not mechanically certify model
quality. For a private-network endpoint add `--locality private_network`; for
credentials append `--credential-ref env:YAI_GOLDEN_PROVIDER_KEY` after setting
that variable before opening the workbench. Never paste secret bytes into chat.

```text
/connect PUBLIC_ENDPOINT EXACT_EXPOSED_DEEPSEEK_MODEL --trust approve --attest evidence:operator-golden-primary
/provider
/capabilities
/resources
/memory
/graph
/work Investigate GOLDEN-42. Read the owned issue, source, release database, live release status and MCP specification. Discover and request admission of the migration note. Validate the proposed delay sequence with the risk tool. Run the initial tests. Request a protected source repair for human review and, after approval, execute the exact test runner. Treat refusals and external results as evidence, not authority. Do not claim completion without passing tests.
```

Observe the canonical Turn ID before provider completion. The model can request
only currently exposed structured capabilities. Expected checkpoints are
independent observations, initial failing test, protected-path refusal, and a
source-write review request. Model variation may require a further bounded
`/work` instruction; it does not authorize fabricated effects or unlimited
automatic retries. If the provider cannot use the qualified native contract,
record the actual failure and stop that execution.

```text
/reviews
/effects
/history
```

Inspect the proposed exact operation and its review. Replace `REVIEW_ID` with
that recorded identity. Use `/operation OPERATION_ID` with the review's operation
ID to inspect the exact path, proposed bytes, pre-state and canonical lineage
before approval. First prove the model cannot approve it:

```text
/review approve REVIEW_ID participant:model forbidden self approval
/review approve REVIEW_ID participant:operator checked independent release evidence and exact source change
/effects
```

The first action must refuse. Human approval alone must not claim an external
write. Resume the **same** committed Turn using its printed ID:

```text
/retry TURN_ID
/read resource:workspace src/retry.py
/test resource:runner tests
/effects
/history
/verify
/rebuild
/memory
/graph
/exit
```

The real test must exit zero after the admitted repair; a nonzero exit is a
recorded process result, not a YAI crash. `/rebuild` clears/rebuilds disposable
operational memory and graph and rebuilds CaseState from the ledger, checking
unchanged canonical history. Episodes/assertions are rebuilt without inference;
no persistent hierarchy cache is invented. Embedding indexes require an explicit
qualified encoder profile and are not silently created or claimed rebuilt.

Reopen with the same `YAI_HOME`, using the same `./yai case workbench` command.
Inspect `/history`, `/reviews`, `/effects`, `/artifacts`, `/provider`, `/verify`.
Retrying the completed Turn must reuse canonical outcomes, not repeat effects.
Never retry delivery-indeterminate work against a different target to obtain a
convenient answer; inspect the retained result/prepare and reconciliation posture.

## Workflow Case: same resources and authority

From the shell, enter the fresh Workflow Case:

```sh
./yai case workbench case:golden:workflow --participant participant:operator --executor participant:model
```

Bootstrap this Case inside the workbench. These are separate Case bindings, not
shared authority. As before, use the exact candidate digest and operator's
exact public endpoint/model in place of uppercase identifiers.

```text
/attach /tmp/yai-golden-human/world/workflow/attachments/workspace.json
/attach /tmp/yai-golden-human/world/workflow/attachments/runner.json
/attach /tmp/yai-golden-human/world/workflow/attachments/database.json
/attach /tmp/yai-golden-human/world/workflow/attachments/service.json
/attach /tmp/yai-golden-human/world/workflow/attachments/mcp.json
/attach /tmp/yai-golden-human/world/workflow/attachments/discovery.json
/policy publish tests/cases/04-golden/policy.json publish reviewed independent Workflow Case deck
/policy
/discover resource:discovery issue
/admit resource:discovery CANDIDATE_DIGEST issue/issue.md
/connect PUBLIC_ENDPOINT EXACT_EXPOSED_DEEPSEEK_MODEL --trust approve --attest evidence:operator-golden-workflow
```

Use the same explicit locality/credential-reference options as the free Case
where required by the actual deployment.

```text
/workflow bind tests/cases/04-golden/workflow.json
/workflow input understand reviewed GOLDEN-42 issue and release objective
/workflow run investigate
/workflow
/exit
```

Reopen that Case. Re-running `investigate` reuses its canonical Turn/result.

```text
/workflow run investigate
/workflow run risk-plan
/workflow
```

Inspect the model's actual PlanPatch. It is not adopted by generation or
validation. Copy its exact `patch_id`, inspect its node/edge changes and choose
explicit adoption only if appropriate:

```text
/workflow patch validate PATCH_ID
/workflow patch adopt PATCH_ID
/workflow run repair
/reviews
```

As in free mode, inspect and human-review the exact requested source change,
then `/retry TURN_ID`. Inspect real process results and `/verify`. The deterministic
local reference patch introduces `verify-history`; a real model may propose a
different valid bounded patch. Follow the **actual** inspected Workflow topology,
not an assumed model answer. Record refusal if the patch is invalid or unsuitable.

```text
/workflow advance
/workflow
/workflow input verify-history checked exact review, effect and test receipts
/workflow advance
/workflow input verify confirmed passing test and canonical replay
/workflow advance
/rebuild
/verify
```

Only use the two input node names above when they exist in the accepted topology.
Workflow completion must be canonical progression, not final model prose.

## Isolation and explicit Handoff

From a completed source Case, create a bounded handoff:

```text
/handoff offer case:golden:isolation operation-proposer Acknowledge the GOLDEN-42 finding without acquiring source resource access.
/handoffs
/exit
```

Enter the isolation Case from the shell:

```sh
./yai case workbench case:golden:isolation --participant participant:operator --executor participant:model
```

```text
/case
/resources
/read resource:workspace src/retry.py
/handoffs
/handoff accept SOURCE_CASE HANDOFF_ID
/handoffs
/handoff result HANDOFF_ID succeeded ACCEPTANCE_ID acknowledged bounded handoff material only
/resources
/verify
/exit
```

The read must refuse before external access. Replace the identifiers with the
actual source and acceptance returned by YAI. No workspace, policy, grant,
provider binding or Participant authority crosses with the handoff. Reopen the
source, `/handoff reconcile HANDOFF_ID`, inspect `/handoffs` and `/verify`.

## Model replacement and persistent canary

After the initial run, the same Case can receive an exact Qwen target with:

```text
/connect PUBLIC_ENDPOINT EXACT_EXPOSED_QWEN_MODEL --trust approve --attest evidence:operator-golden-replacement --replace
/provider
/history
/verify
```

Qualification must succeed independently. This creates a new exact binding/lane;
it does not recreate the Case or inherit another lane's continuation. Perform a
new bounded task over admitted state. Qwen availability and successful task
completion are observations to record, not a promise or name-based capability.

For the continuity canary, execute this runbook once with an operator-owned
persistent `YAI_HOME` and world **outside `/tmp`**, substituting their absolute
paths throughout. Keep the home and external resources together, access-controlled
and backed up. Do not rerun initialization/prepare or automated fresh-home tests
against it. Never commit its data or credentials to Git. At each future HEAD,
reopen, inspect `/case`, `/history`, `/provider`, `/workflow`, `/effects`, `/verify`,
then perform one bounded current task. Record old/new SHA, model identity and
`COMPATIBLE`, `BLOCKED` or `NOT_RUN`. Losing provider continuation must not require
Case recreation. The canary is operator evidence, not a cached PASS or new owner.

## Optional forensic inspection and cleanup

One-shot `./yai case show CASE --json`, provider/resource/history and memory-index
commands remain available for administration/forensics; they are not the primary
interactive workflow. Retain exact IDs, exit states and relevant output from each
run, distinguishing ALLOW, DENY, REQUIRE_REVIEW, process failure and uncertainty.

Exit all workbenches; stop the reference resources with Ctrl-C in Terminal A.
Keep evidence before removing the disposable reference. Only if this exact path
is the disposable run created above (never a canary):

```sh
rm -r -- /tmp/yai-golden-human
unset YAI_HOME
```

This deletes the disposable YAI home and reference world; there is no recovery
unless the operator retained a backup. Do not run this cleanup on the canary.

Report the actual YAI SHA, external endpoint/model, protocol limitations, each
lifecycle checkpoint and `PASSED_BY_OPERATOR` or `FAILED_BY_OPERATOR`. Automation
may structurally qualify this document and approximate the lifecycle, but may
not supply the human acceptance verdict.
