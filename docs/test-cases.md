# Validation and operator evidence

Authority: current validation entrypoints and the limits of their claims.
The [classification](../tests/classification.tsv) and [testing doctrine](../tests/README.md)
separate proof, posture, provider, dependencies, mutations and cadence.

## Local gates

From the repository root with the toolchain and dependencies installed:

```sh
make test-fast
```

For a documentation-only closure with no runtime/schema change:

```sh
make check-docs check-layout test-roadmap test-topology
python3 tools/validation/topology.py audit --static
git diff --check
```

The roadmap guard derives counts solely from its maturity rows and tests reject
stale counts, undefined evidence and competing execution/authority declarations.
This is no-provider documentation/unit proof, not new product qualification.
The full publication gate below remains required for implementation changes.

Before publication, without starting YVEX:

```sh
make check
```

When a wave asks for characterization too, select both in one graph:

```sh
make check characterization
```

Shared prerequisites execute once. Separate invocations intentionally rerun
overlapping evidence. Build only compiles. `test-local` omits separately
classified endurance; `check` retains formerly default bounded endurance.
The additional large previously ignored scale characterizations are opt-in:

```sh
make test-endurance
```

Failure isolation: `test-unit`, `test-component`, `test-contract`,
`test-product`, `test-recovery`, followed by the named failing leaf.
Loopback tests use actual HTTP/TLS, stores, processes and temporary files.
A C component assertion is not product-reachable effect qualification.

## External YVEX

`make test-golden-external-yvex` is the separate full reference free/Workflow
Product lane. It uses the same actual YAI workbench and reference resources but
never starts a model fixture. Missing exact endpoint/model exits nonzero with
DEPLOYMENT_LIMITATION before any external request. Optional
`YAI_EXTERNAL_PROVIDER_LOCALITY` is `private_network` by default (explicit
`loopback` or `remote` also admitted). Set `YAI_EXTERNAL_PROVIDER_API_KEY` only
when needed; it is passed by environment reference, not stored in Case payloads.
This lane is prepared but has not qualified a live deployment in this wave.
It does not replace the narrower historical text lane below or human acceptance.

The operator supplies `YAI_EXTERNAL_PROVIDER_BASE_URL` and the exact exposed
`YAI_EXTERNAL_PROVIDER_MODEL`. Optional: `YAI_EXTERNAL_PROVIDER_API_KEY`,
`YAI_EXTERNAL_PROVIDER_TIMEOUT_MS`, `YAI_EXTERNAL_PROVIDER_REF`.
Historical `YVEX_BASE_URL` / `YVEX_MODEL` aliases remain accepted.
No endpoint/model is guessed; the first catalog entry is never auto-selected.

With those real values configured:

```sh
make test-external-yvex
```

`qualification-yvex-provider` is the same explicit entrypoint. Missing
configuration exits 3 from the script (nonzero through Make), reporting
`blocked_external_dependency` / `DEPLOYMENT_LIMITATION` before any network
request. An unreachable endpoint or unavailable exact model likewise blocks.
Invocation/contract failures fail the lane. There is no fallback fixture and
YAI does not start or administer YVEX.

Current external proof is **text Chat Completions** through the public generic
OpenAI-compatible boundary and retained Advanced legacy prompt client. It
proves real invocation, ContextFrame/result lineage and exposed-model
correlation when returned; not model quality or I03/I04 media/composition
interoperability. Logs distinguish discovery attempts, invocation attempts and
confirmed ProviderResults. Any supplied YVEX ref is attributed to the operator,
not represented as independently observed repository truth.

## Manual acceptance

The [ZERO-TO-CURRENT Golden runbook](zero-to-current.md) is the cumulative human
product procedure. It evolves at current HEAD instead of spawning a separate
manual for each wave. Its real model is operator-supplied YVEX/DeepSeek, never
the deterministic model peer used by automation. Only the operator may report
human acceptance. Local Product automation and structural runbook checks do not
change `HUMAN_GOLDEN_CASE = PENDING_OPERATOR` into a human PASS.

`make test-golden-local` is a separate explicit cadence: real Product workbench,
stores, confined filesystem/process, SQLite and HTTP/MCP resource peers with a
deterministic loopback model. Run it **in addition to** the ordinary publication
union for substantive Case/product changes. It is not hidden inside `check` and
is never external YVEX evidence. The executable topology audits its reachability.

`make test-manual` lists procedures, not PASS. Use the repository-root `./yai`
launcher. This fresh product inspection requires no provider:

```sh
make build
YAI_VALIDATION_ROOT="$(mktemp -d /tmp/yai-validation-manual.XXXXXX)"
export YAI_HOME="$YAI_VALIDATION_ROOT/home"
./yai init --tenant tenant:validation-manual --organization organization:validation
./yai case create case:validation-manual --tenant tenant:validation-manual
./yai case participant role add case:validation-manual --participant participant:model --role model-executor
./yai case participant link-principal case:validation-manual --principal self --participant participant:model
./yai case show case:validation-manual --json
./yai doctor
./yai store status
```

After inspection, remove only that disposable directory:

```sh
rm -r -- "$YAI_VALIDATION_ROOT"
unset YAI_HOME YAI_VALIDATION_ROOT
```

See [TEST.TOPOLOGY.0](https://github.com/yailabs/yai/blob/c187648e9d9909d4d9b6711f131fa726d48cd585/refoundation/validation/test-topology-0/REPORT.md) for the
full procedure and retained evidence. [Test-case wrappers](../tests/cases/README.md)
distinguish current inspection from archived fixture/lab evidence. Archived
manual commands are not automatically current product acceptance.

## Report claims separately

Preserve command/run and status for:

- unit/component: no provider;
- contract: no-provider parsers and/or deterministic loopback transport;
- product/recovery: local stores/processes and declared fixture peers;
- endurance: actual scenarios/bounds/timing, or NOT RUN;
- external YVEX: exact target/shape and real result or dependency limitation;
- manual acceptance: performed procedure or NOT RUN.

None of these observations promotes provider output, fixture effects, derived
memory, graph or context into canonical or operational authority.

## Cumulative publication evidence

Future substantive handoffs report these independent observations, not a single
aggregate count or cached verdict:

| Evidence | Admitted status |
|---|---|
| AUTOMATED REGRESSION | proof classes and provider modes, PASS/FAIL |
| GOLDEN CASE LOCAL | PASS/FAIL/NOT_AFFECTED with reason |
| GOLDEN CASE EXTERNAL YVEX | PASS/FAIL/NOT_RUN/DEPLOYMENT_LIMITATION; exact endpoint/model when executed |
| CONTINUITY CANARY | COMPATIBLE/BLOCKED/NOT_RUN against retained operator state |
| HUMAN GOLDEN CASE | PENDING_OPERATOR/PASSED_BY_OPERATOR/FAILED_BY_OPERATOR with SHA |
| ZERO-TO-CURRENT RUNBOOK | updated or unchanged with reason |
| GOLDEN CASE BLOCKERS | exact product or public-provider gaps |

No automated result can promote human acceptance to PASS. A changed product HEAD
does not silently inherit a previous human PASS. Preserve operator canary data;
fresh automated homes never reset it.
