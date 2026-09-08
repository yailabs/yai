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
OpenAI-compatible endpoint. YAI does not start YVEX or load a model. Discovery
selects an exact singleton automatically; multiple entries require an operator
choice, never an arbitrary first model. Native function calls, correlated tool feedback and
JSON-object output must pass actual qualification for the complete reference
Workflow. A text-only public endpoint is insufficient. Record that limitation;
do not substitute a fake model or use a private protocol.

Qualification of small synthetic inputs is not qualification of a complete
Case context. The 2026-09-08 [public-provider hardening evidence](../refoundation/validation/provider-connect-hardening/REPORT.md)
records successful native function/result qualification, but a real cognitive
SEND refused with HTTP 413 `request_too_large`. A separate bounded input returned
`token output capacity exceeded` even with `max_tokens=1`. That deployment does
not yet qualify this lifecycle. Resolve its public input-capacity contract with
the provider operator; do not discard governed Case context to hide the refusal.
An indeterminate submitted Turn must not be blindly retried after reconnection.

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
python3 tests/cases/04-golden/world.py prepare \
  --root /tmp/yai-golden-human/world
git rev-parse HEAD
./yai init
```

If the directory already exists, STOP: preserve the previous run. Choose a new
explicit directory and substitute it consistently below. `world.py prepare`
refuses an existing world and does not initialize YAI or run the lifecycle.

For `init`, answer Tenant `golden`, Organization `golden`, then explicitly type
`create`. An initialized home keeps its existing identity; multiple authorized
Tenants require selection, not an ambient default. Noninteractive automation
retains the explicit `--tenant` / `--organization` form.

Each shell command above is complete. The backslash continues the preparation
command; never place a newline between a flag and its value without it. The
reference endpoint defaults to `http://127.0.0.1:18240`.

Open the Case using its short name:

```sh
./yai open golden:free
```

For a new Case, review the exact Tenant/Case and type `create`. Accept the
suggested Participant names `operator` and `model` with Enter, inspect the roles
and Principal link, then type `admit`. The approved Participant facts commit
atomically. This does not attach resources, publish policy, grant an effect or
trust a provider. The human operator holds the reviewer role; the separate model
Participant does not inherit the human Principal link or review authority.

If you already created `case:golden:free` but its Participant setup failed, use
the same `YAI_HOME` and the same `./yai open golden:free`: it preserves that Case
and offers only the missing setup. Do not reset it or repeat world preparation
over an existing world. Reopening a ready Case does not repeat setup or mutate
its generation. `./yai open` selects from visible Cases; it writes no global
selected-Case state. `/setup` explicitly repairs/adds the shown Participant
profile within an open Case; it never silently transfers a Principal link.

## Free Case: remain inside YAI

The following are workbench actions, **not shell commands**. REPLAI handles the
editor. Paths are literal; the workbench does not expand shell variables or
infer attachments from ordinary conversation text.
Execute one action at a time. For a guided action, answer each displayed question
before continuing: the lines below show the order, not a script to paste as one
multiline draft. Multiline paste remains one editor submission, not a hidden
command batch. `/attach` alone can ask for a file instead of its complete form.

```text
/case
/help
/participants
/history
/policy
/attach /tmp/yai-golden-human/world/free/attachments/workspace.json
/attach /tmp/yai-golden-human/world/free/attachments/runner.json
/attach /tmp/yai-golden-human/world/free/attachments/database.json
/attach /tmp/yai-golden-human/world/free/attachments/service.json
/attach /tmp/yai-golden-human/world/free/attachments/mcp.json
/attach /tmp/yai-golden-human/world/free/attachments/discovery.json
/policy publish
tests/cases/04-golden/policy.json
publish reviewed Golden release policy deck
/policy
/discover discovery issue
```

Before policy publication the Case is not READY. Attachment alone grants no
operation permission. After publication inspect READY and contributing source,
artifact and binding identities. Copy the `digest` of the exact `issue/issue.md`
candidate in the discovery result's `entries` into this admission action. Do not
use the enclosing observation's `integrity_digest`:

```text
/admit
discovery
CANDIDATE_DIGEST
issue/issue.md
/artifacts
```

Use `/connect` and answer its conditional questions one at a time. There is no
connection profile to choose. Supply the exact **public endpoint from the YVEX
operator**. YAI reads `/v1/models`: a single identity is displayed without asking
you to type it; multiple entries require a displayed number or exact name. Check
that the selected identity is the intended DeepSeek target. Empty/malformed
catalogs refuse, without inference or Case binding.

Literal-IP/localhost locality needs no question. For ambiguous DNS names select
`loopback`, `private_network` or `remote` truthfully. Only an HTTP 401/403 challenge
asks for a credential reference such as `env:YAI_GOLDEN_PROVIDER_KEY`, with that
variable set before opening YAI. Never paste secret bytes into chat. Discovery
uses a bounded metadata GET, not a private runtime inspection or load request.

The final `approve` is your explicit trust and semantic-suitability attestation
for the displayed target and Case. YAI generates its exact provenance reference;
you no longer invent an evidence identifier. Suitability remains
operator-attested, not mechanically certified model quality. If a primary
already exists, the same confirmation requires `replace` and displays the old
binding. Empty input cancels; it does not approve. Only metadata GET occurs
before confirmation: synthetic inference, trust and binding follow approval.
Text qualification must pass before binding. Native functions and JSON are
reported independently in `capabilities` and `realization_shapes`; failed probes
remain in `qualification_failures`. For this full Golden lifecycle require both
`native functions: qualified` and `JSON: qualified` in the human connection
summary; `/details` exposes the exact corresponding booleans and evidence IDs.
A text-only connection is usable
for conversation but is not Golden readiness; do not continue tool/Workflow work
with missing evidence. Exact execution refuses any unqualified capability.

Qualification performs real synthetic model work, not just a connectivity ping.
It prints stage/elapsed updates; a completed `/v1/models` request alone is not
execution qualification. Buffered inference defaults to a 300-second total
response budget, independently of the 30-second connection/resource budget.
For an explicitly slower deployment an operator may set
`YAI_PROVIDER_RESPONSE_TIMEOUT_SECS` (1–3600) before launching YAI. Expiry after
submission remains delivery-indeterminate: do not blindly retry or change target.
This setting changes waiting, not model speed. Text-only qualification never
authorizes tools, JSON Workflow work, or media. Inspect `provider show TARGET`
for stored shape/failure/timing evidence instead of re-running probes to inspect.

```text
/connect
PUBLIC_ENDPOINT
approve
/details
/provider
/capabilities
/resources
/memory
/graph
/work Investigate GOLDEN-42. Read the owned issue, source, release database, live release status and MCP specification. Discover and request admission of the migration note. Validate the proposed delay sequence with the risk tool. Run the initial tests. Request a protected source repair for human review and, after approval, execute the exact test runner. Treat refusals and external results as evidence, not authority. Do not claim completion without passing tests.
/details
```

Observe the separate YAI "Message saved" notice before provider completion;
it is not part of the model's reply. Normal responses are inside `[Model]`,
application status/errors inside `[YAI]`. No diagnostic JSON is appended to the
model reply. After the outcome, `/details` explicitly shows the last action's
canonical Turn ID, exact execution evidence and pending review lineage.
Use `/history` for durable inspection after another action or restart. The model can request
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

Use `/review`: it selects a pending review (asks if several exist), displays the
exact operation, proposed bytes, pre-state and lineage, then asks for your action
and reason. Review it before answering; `deny` and `defer` remain real actions.
An optional adversarial administrative check uses the exact displayed review ID:

```text
/review approve REVIEW_ID participant:model forbidden self approval
```

That model self-approval must refuse. Normal human approval is:

```text
/review
approve
checked independent release evidence and exact source change
/effects
```

Human approval alone must not claim an external write. Resume the **same** latest
committed Turn in the selected conversation; `/details` after its outcome exposes
the resolved exact ID. `/retry TURN_ID` remains available for an explicitly different historical
Turn. Neither action duplicates user SEND or weakens delivery safety:

```text
/retry
/read workspace src/retry.py
/test runner tests
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

Reopen with the same `YAI_HOME`, using `./yai open golden:free`.
Inspect `/history`, `/reviews`, `/effects`, `/artifacts`, `/provider`, `/verify`.
Retrying the completed Turn must reuse canonical outcomes, not repeat effects.
Never retry delivery-indeterminate work against a different target to obtain a
convenient answer; inspect the retained result/prepare and reconciliation posture.

## Workflow Case: same resources and authority

From the shell, enter the fresh Workflow Case:

```sh
./yai open golden:workflow
```

Confirm creation and Participant admission as for the free Case. Bootstrap
inside the workbench. These are separate Case bindings, not
shared authority. As before, use the exact candidate digest and operator's
exact public endpoint/model in place of uppercase identifiers.

```text
/attach /tmp/yai-golden-human/world/workflow/attachments/workspace.json
/attach /tmp/yai-golden-human/world/workflow/attachments/runner.json
/attach /tmp/yai-golden-human/world/workflow/attachments/database.json
/attach /tmp/yai-golden-human/world/workflow/attachments/service.json
/attach /tmp/yai-golden-human/world/workflow/attachments/mcp.json
/attach /tmp/yai-golden-human/world/workflow/attachments/discovery.json
/policy publish
tests/cases/04-golden/policy.json
publish reviewed independent Workflow Case deck
/policy
/discover discovery issue
/admit
discovery
CANDIDATE_DIGEST
issue/issue.md
/connect
```

Answer the conditional connection questions as above and confirm this Case's
own target/attestation. No provider authority is copied from the other Case.

```text
/workflow bind
tests/cases/04-golden/workflow.json
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
then `/retry`. Inspect real process results and `/verify`. The deterministic
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

First create/open the empty isolation Case from the shell with
`./yai open golden:isolation`, confirm its own setup and `/exit`. It must have
no source resources or policy. Reopen the completed source with
`./yai open golden:free` (or `golden:workflow`), then offer a bounded handoff:

```text
/handoff offer case:golden:isolation operation-proposer Acknowledge the GOLDEN-42 finding without acquiring source resource access.
/handoffs
/exit
```

Enter the isolation Case from the shell:

```sh
./yai open golden:isolation
```

```text
/case
/resources
/read workspace src/retry.py
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

After the initial run, the same Case can receive an exact Qwen target with
`/connect`: supply its actual endpoint, select Qwen if several catalog
entries exist, review the displayed target and type `replace` at the one final
trust/attestation/replacement confirmation. Scope/credentials are asked only when
necessary, as above.
The compact action is the same; replacement is never an implicit default.

```text
/connect
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
