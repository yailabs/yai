# YAI quickstart

This is the repository-local product path. It uses only supported porcelain;
engineering commands remain available through `yai help --advanced`.

## Build and initialize

From the repository root:

```sh
make build-rust
export YAI_HOME=/tmp/yai-demo-home

./yai
./yai doctor
./yai init --tenant tenant:demo --organization organization:demo
./yai doctor
./yai identity whoami
```

`yai init` prepares the local layout and delegates identity/Tenant creation to
the existing security owner. Repeating the command is safe and reports the
existing state. Use a durable private directory instead of `/tmp` outside a
disposable evaluation.

## Create and inspect a Case

```sh
./yai case create case:demo --tenant tenant:demo
./yai case show case:demo
./yai case stop case:demo
```

A new Case can be shown or stopped before it has run. `stop` affects only an
active execution; it does not cancel or close the Case.

## Establish the execution identities

Participant identity is deliberately distinct from the authenticated
Principal. Binding a role does not silently create review authority or a
Principal link.

```sh
./yai case participant role add case:demo \
  --participant participant:model --role model-executor

./yai case participant role add case:demo \
  --participant participant:model --role operation-proposer

./yai case participant list case:demo
```

Attach a provider through its generic data-plane identity:

```sh
./yai case provider attach case:demo \
  --participant participant:model \
  --endpoint http://127.0.0.1:8080/v1/chat/completions \
  --model exposed-model-id \
  --provider provider:local
```

YAI needs no provider implementation profile, artifact or engine identity.
Credential values are referenced through configuration and are never printed
by `case show`.

## Attach a governed Resource

Use a disposable existing directory for this example:

```sh
mkdir -p /tmp/yai-demo-resource/allowed

./yai case resource attach filesystem case:demo \
  --resource resource:workspace \
  --root /tmp/yai-demo-resource \
  --allow-prefix allowed \
  --policy-owner participant:model

./yai case resource list case:demo
./yai case show case:demo --json
```

The Workflow or Case may propose work, but Policy/Decision/Grant and the
ResourceFence still control physical effects.

## Bind exact Policy

Ingest a bounded policy source, then validate, publish and bind its returned
artifact ID:

```sh
./yai policy ingest ./policy.json --tenant tenant:demo
./yai policy validate policy-artifact-id --reason 'operator validation'
./yai policy publish policy-artifact-id --reason 'operator publication'
./yai case policy bind case:demo \
  --artifact policy-artifact-id --reason 'Case policy selection'
```

The porcelain reads the current Case generation and preserves the engine's
optimistic concurrency check; it does not grant authority or overwrite a
concurrent Case change.

## Run free-form or Workflow-bound work

A Workflow is optional. Free-form bounded work uses:

```sh
./yai case run case:demo \
  --participant participant:model \
  --resource resource:workspace \
  --prompt 'inspect the governed task'
```

For deterministic progression, define and bind an exact immutable Definition:

```sh
./yai workflow define --tenant tenant:demo --file workflow.json
./yai workflow bind case:demo --definition workflow-definition-id \
  --executor analyst=participant:model \
  --resource workspace=resource:workspace
./yai workflow status case:demo
```

HumanInput is recorded with `yai workflow input`; authority is resolved with
the separate `yai review` family. Runtime hosting and control are under
`yai runtime`.

## Discover exact syntax

```sh
./yai case --help
./yai case provider attach --help
./yai help --advanced
./yai help --json
./yai completion zsh
```

All product operations support a deterministic JSON envelope. JSON and
redirected output contain no ANSI styling; `NO_COLOR=1` disables color for
human TTY output as well.

## What to read next

- [Command architecture](commands.md)
- [Executable architecture](architecture.md)
- [Test cases](test-cases.md)
- [Implementation roadmap](../ROADMAP.md)

Passing local qualification does not imply a stable public SDK, distributed
runtime, remote enterprise IAM, provider governance or adaptive Workflow.
