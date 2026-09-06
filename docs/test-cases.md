# Validation and operator evidence

Authority: current validation entrypoints and the limits of their claims.
The [classification](../tests/classification.tsv) and [testing doctrine](../tests/README.md)
separate proof, posture, provider, dependencies, mutations and cadence.

## Local gates

From the repository root with the toolchain and dependencies installed:

```sh
make test-fast
```

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

See [TEST.TOPOLOGY.0](../refoundation/validation/test-topology-0/REPORT.md) for the
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
