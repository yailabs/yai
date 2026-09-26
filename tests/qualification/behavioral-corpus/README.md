# Case behavioral evaluations

The corpus is a qualification client over typed Host operations, not a semantic
owner. `seed.json` contains portable structural checks and explicit overlay
requirements. An evaluation without executable steps reports `NOT_RUN`, never
PASS. Initial runner coverage is deliberately smaller than the program's final
acceptance coverage.

Real Case-bound model qualification in the everyday operator profile uses only
`case:tech-infra-inference-service`. The former DeepSeek-specific Cases are
historical observations, not a pattern for new runs. Synthetic Case/parallel
topology tests allocate their own disposable profiles; never create a model
qualification Case in the operator home.

`schema.json` is the versioned JSON Schema for a complete corpus. Overlays merge
by logical identity before validation/expansion. `enterprise.overlay.json` adds
an exact documentary Recall assertion; `enterprise.py inspect --profile PATH`
writes a new profile from acquisition inventory, independently of Recall. It
pins the release document's Source, revision and digest. A later reacquisition
requires a deliberately new profile; stale expectations are not silently updated.

Run the bounded inventory without a Host:

```sh
python3 tools/validation/behavioral_corpus.py tests/qualification/behavioral-corpus/seed.json --list
python3 tests/qualification/behavioral-corpus/test_runner.py
```

Execution requires an explicit profile JSON (`case_ref`, `tenant_ref`, and any
overlay-specific exact identities), `--home` and a new `--output` JSONL path.
The selected Host must already exist. The runner neither initializes storage nor
starts a model. Only catalog-declared read/derived operations run by default;
other impacts require explicit `--allow-mutations`. This switch is not authority:
the actual authenticated Application dispatcher remains responsible for admission.
Never select Golden or the continuity canary for mutation qualification.

Each evaluation declares its meaning, dimensions, preconditions, required refs,
allowed/forbidden actions, expected consequences, mode and repeatability. Exact
inputs use `{"$ref":"/profile/..."}`; later steps may reference earlier result
identities through `/results/STEP/...`. Assertions compare exact JSON facts, not
natural-language spelling. A repeated mutation must explicitly reuse its original
request identity; the transport never automatically retries a lost response.
Step-local assertions guard authoritative preconditions before later operations.
The complete operation allowlist and step identities are checked before dispatch.
`some` asserts exact fields of one array item; keys beginning with `/` are JSON
pointers within that item, so a matching filename cannot conceal a wrong revision.

Variant axes generate stable identities from canonical JSON; expansion is bounded
before execution (64 by default). Use explicit opt-in suites for larger matrices.
The transcript records the corpus/profile digests, actual Host catalog/instance,
ordered requests/results and per-operation latency. Wrap publication runs with
`tools/validation/capture_evidence.py` to retain source SHA, command, environment,
exit status and pre-state. Transcripts contain disclosed product data: select
non-secret qualification material and review before committing any evidence.

KNOWS is context-free provider characterization, not a claim about a populated
Case; SEES tests actual disclosure; RECALLS exact reconstruction; REMEMBERS temporal
continuity; REASONS grounded interpretation; CAN_DO admitted work; REFUSES denied
work; RECOVERS exact retry/restart; ISOLATES cross-scope absence. PERFORMS is a
separate measured axis. Structural PASS never grants a language-quality verdict.
Current repeated-read coverage does not qualify actual process restart.

Every run ends with a retained `summary` record and the same summary on stdout:
verdict counts overall and per dimension, including zero-count dimensions.
These are observed structural verdicts, not capability maturity or model scores.
Failed and incomplete evaluations retain their dimensions too. A suite mixing
PASS and NOT_RUN reports PARTIAL; an entirely unexecuted suite reports NOT_RUN
with exit 2. FAIL exits 1, bounded pending observation exits 2, and an executed
PASS/PARTIAL smoke suite exits 0. Inspect the summary rather than treating a zero
exit status as complete coverage. Language quality remains NOT_ASSESSED.

Case-specific suites supply independently selected expected facts and explicit
controlled perturbations; they must not derive expected evidence from the same
query under test. Historical Decision trajectory operations are reusable
structural evidence, not optimal-action or model-reasoning scores. Future E
absent/present comparisons require a public qualified producer contract; no
Experiential Computational State E or Deliberation State L is claimed here.

`portfolio.json` is a small reconstructible two-Case topology. Run
`make test-behavioral-portfolio` with the repository's supported Rust toolchain.
The runner creates a new profile, admits each Case through typed Application,
mutates distinct roles concurrently through independent Host clients, repeats
each exact mutation without generation inflation, checks stale snapshot refusal
and cross-Case role isolation, restarts the Host, and compares reopened snapshots
and CLI replay. This proves local product behavior, not model-driven competence.
It records allocated profile bytes, stops the Host and removes only its own new
profile. `portfolio.py --retain` explicitly preserves the stopped profile for
further Studio inspection. Neither operator Case nor Golden is a cleanup target.

Execution steps may use an explicit bounded `observe` clause only for cataloged
read operations. Its response pointer, pending values, observation count and
interval are authored in the evaluation. Each read and its timing are retained.
Exhausting the bound while execution remains pending reports `INCOMPLETE`, not
model failure or success; transport loss
is never automatically retried. Mutation or derived-computation operations
cannot use observation loops. Exact SEND retry remains a separate explicit step.
This lets the same corpus observe slow real providers without silently submitting
another Turn or disguising an unfinished execution as a failed model answer.

## Operational infrastructure Case

The operator retired `case:yai-enterprise-launch`; do not recreate or reopen it
as the active product use case. The replacement is
`case:tech-infra-inference-service`, an internal infrastructure service change
on the operator's Exon/Spark stack. No invented company, customer or production
acceptance is claimed. `infra-operations.md` is the requested operating brief;
`infra-workflow.json` defines seven human checkpoints, not automated effects.

`enterprise.py advance --scenario infrastructure` reuses normal YAI Source
declaration/acquisition and Knowledge construction for this Case. It requires
explicit `--yai`, `--tenant`, `--evidence`, `--perimeter` and `YAI_HOME`. The
perimeter admits only the named documentary files and a read-only policy.
The command does not start YVEX, load a model or approve infrastructure changes.
The infrastructure profile uses separate `operations_*` references from acquisition
inventory and `infra.overlay.json`; release expectations cannot resolve against
that profile. Their assertions do not describe the infrastructure Case. Workflow definition/binding
remains an ordinary separate YAI operation; acquiring the recipe does not bind it.

## Retained governed actions

`retained-conversation.json` observes an already completed Conversation by its
durable cognitive request. It checks exact Turn/result/target identity and the
archived input observation, model and serialized-request digest, repeats the read,
and checks current hidden-Participant refusal without disclosed output. It never
calls SEND and can run after Host restart while another Case is executing.

Pin `case_ref`, `participant_ref`, `request_ref`, `turn_ref`, `result_ref`,
`target_ref`, `invocation_ref`, `input_observation_ref`,
`serialized_request_digest` and `model_id` from the original completed execution
and its context evidence. Do not derive these expectations from the read being
tested. Use the normal runner without `--allow-mutations`. This avoids trying to
reconstruct the original submission with a newer Case generation just to inspect
its answer. The suite assesses retained identity and disclosure, not answer
quality or inference performance; those remain separate evidence classes.

`retained-action.json` is a reusable, read-only structural suite over
`execution.get` for an already executed Resource request. It checks exact Case,
Participant, Operation, effect, result and receipt identities; repeated reads;
and refusal without disclosed data for an unlinked Participant. It neither
submits the action nor claims that a model chose it correctly.

Supply an independent profile from the original successful product result:
`case_ref`, `participant_ref`, `submission_ref`, `operation_ref`, `effect_ref`,
`result_ref`, `receipt_ref`, `outcome`, and `external_execution_started`. For a
Resource effect receipt, `result_ref` is its `post_observation_id`. Never obtain
expected identities from the observation being tested. Use this corpus directly
with the ordinary runner, explicit `--profile`, `--home` and fresh `--output`.
No `--allow-mutations` is needed. A successful retained receipt is distinct from
current authority to perform the action again. Wrong expected receipt identity
must fail; language/model competence remains `NOT_ASSESSED`.

## Governed question corpus

`conversation.json` is the reusable effectful question lane. It uses ordinary
`conversation.send`, bounded `execution.get` observation, exact disclosed context
and explicit idempotent retry. A profile must supply `case_ref`, `participant_ref`,
`target_ref`, exact `model_id`, new `thread_ref`/`submission_ref`, `question` and its
exact `question_utf8` byte array. The profile is input, not authority. Select the
operator's configured primary model; the suite never registers or trusts a target.
It requires explicit `--allow-mutations`, appends real Conversation history and
must not be run against Golden or a canary as disposable setup.

The suite requires a successful answer: a truthful refusal does not pass this
positive lane. Pending observation ends `INCOMPLETE`; uncertain delivery is not
retried by the transport. It checks exact Case/Participant/target/model identity,
committed question text, nonempty candidate answer, W/projection/frame/input
observation, matching result identity between ordinary and contextual reads,
hidden-Participant refusal, and unchanged state after exact SEND retry. It does
not assume an unknown target's capacity is known or score prose by exact spelling.
Use separate negative capacity and authority suites for refusal properties.

`context_capacity.py` runs this exact corpus against its isolated controlled
producer in addition to exact-byte preflight/refusal/restart checks. Controlled
PASS qualifies the pipeline, not DeepSeek or language reasoning. The same corpus
can run against an operator-selected real target with a new profile; assess the
answer separately against Case-specific independently selected refs and questions.
For the infrastructure Case, ask what blocks service readiness, which Sources
support the conclusion, what remains unmeasured and which changes require approval.
`REASONS` names the evaluation pressure; `language_quality=NOT_ASSESSED` remains
explicit until a separate reviewer supplies a grounded verdict. The typed catalog
is unchanged: this is a qualification consumer of existing operations.

## Case-bound public capacity

`provider-capacity.json` is a separate read-only lane for a selected, already
governed deployment. Pin its `target_ref` independently from the Case setup and
its `model_id`, `engine_generation`, `runtime_binding_identity`,
`runtime_model_identity`, `capacity_plan_identity`, `input_capacity_tokens` and
`sequence_capacity_tokens` from a separate public `/v1/models` observation.
Combine those fields with the Case/Tenant/Participant profile produced from
independent Source inventory; do not derive expected capacity from the YAI
`provider.models` response being tested. The suite compares the primary Case
binding, approved target and exact public catalog through the typed Host, then
confirms the Case projection did not change. It uses no `--allow-mutations` and
never sends an inference request. A changed engine or capacity needs a newly
observed profile, not silent expectation refresh.

```sh
python3 tools/validation/behavioral_corpus.py \
  tests/qualification/behavioral-corpus/provider-capacity.json \
  --profile /path/to/independently-pinned-profile.json \
  --home "$YAI_HOME" --output /path/to/new-observations.jsonl
```

A PASS proves only a point-in-time public capacity claim for the exact Case
target. It does not prove current model residency, resources, the next request's
fit, successful generation or answer quality. Keep the public catalog snapshot
and corpus transcript as distinct evidence; review their actual payloads before
publishing Case context or machine details.
