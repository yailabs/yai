# MANUAL ACCEPTANCE — ZERO TO USE CASE

From the YAI repository root. These are normal commands, not an assertion
script: no shell fail-fast flags, Python extraction or invented provider values.
No YVEX is needed for the local procedure. Tests own their fixture peers.

## Fresh product state

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

Negative path: the following unknown Case must refuse, not create a Case or
contact a provider. The following valid inspection must still succeed.

```sh
./yai case show case:validation-missing --json
./yai case show case:validation-manual --json
```

## Distinguish proof environments

Choose the relevant gate; these are not all mandatory sequential commands.
`check` already includes the deterministic contract and composition proofs.

```sh
make test-fast
make test-contract
make smoke-cognitive-execution-composition
make check characterization
```

Inspect `validation_entry` and Rust partition records: contracts use declared
loopback peers; I04 is product/loopback, not live YVEX. The final combined gate
shares Make dependencies, and the catalog audit prints coverage parity.

The following lists manual procedures and is **not** a PASS or execution:

```sh
make test-manual
```

## Separate external qualification

Only if the operator has configured the actual endpoint and exposed model in
`YAI_EXTERNAL_PROVIDER_BASE_URL` and `YAI_EXTERNAL_PROVIDER_MODEL`, the same
command can qualify real text interoperability. Optional credential:
`YAI_EXTERNAL_PROVIDER_API_KEY`. No model/server is launched by this command.

```sh
make test-external-yvex
```

Without that configuration expect nonzero, `DEPLOYMENT_LIMITATION`,
`external_request_attempted: false`, never a loopback fallback PASS. With real
configuration inspect the exact endpoint/model, actual invocation/result IDs
and final posture. This does not claim I03/I04 live YVEX typed-media execution.

## Cleanup

Only the directory created above is disposable. This removes its Case/store,
not repository files, external providers, resources or model artifacts.

```sh
rm -r -- "$YAI_VALIDATION_ROOT"
unset YAI_HOME YAI_VALIDATION_ROOT
```
