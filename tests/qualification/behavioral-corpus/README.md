# Case behavioral evaluations

The corpus is a qualification client over typed Host operations, not a semantic
owner. `seed.json` contains portable structural checks and explicit overlay
requirements. An evaluation without executable steps reports `NOT_RUN`, never
PASS. Initial runner coverage is deliberately smaller than the program's final
acceptance coverage.

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
