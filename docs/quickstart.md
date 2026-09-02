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

For a reusable Tenant-governed target, keep configuration, contract evidence,
administrative approval, operational health and Case binding explicit:

```sh
./yai provider add --tenant tenant:demo --provider-key local-fixture \
  --endpoint http://127.0.0.1:8080 --model exposed-model-id \
  --locality loopback
./yai provider qualify --target provider-target-id
./yai provider trust approve --target provider-target-id
./yai provider show --target provider-target-id
./yai case provider bind case:demo --participant participant:model \
  --target provider-target-id --failover safe_only --max-attempts 3
./yai case provider show case:demo
```

Qualification sends fixed synthetic probes, never Case context. Binding does
not imply qualification, approval, health or authority. With several repeated
`--target` flags, their order is the deterministic preference order.

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

When future progression must change, propose and adopt it explicitly:

```sh
./yai workflow patch propose case:demo --file plan-patch.json
./yai workflow patch validate case:demo --patch workflow-plan-patch-id
./yai workflow patch adopt case:demo --patch workflow-plan-patch-id
./yai workflow status case:demo
```

The patch file names the current effective-topology digest. Adoption is a
Tenant-Owner action at a quiescent boundary; a model-produced candidate uses
the same validation and cannot adopt itself.

Bounded same-Tenant work information can cross Case boundaries without
sharing authority:

```sh
./yai case handoff offer case:source --target case:target \
  --value 'inspect the bounded request' --role operation-proposer
./yai case handoff pending case:target
./yai case handoff accept case:target --source case:source \
  --handoff handoff-id --participant participant:operator
./yai case handoff result case:target --handoff handoff-id \
  --participant participant:operator --outcome succeeded --value 'result'
./yai case handoff reconcile case:source --handoff handoff-id
```

The target performs ordinary work under its own Case owners. Reconciliation
copies no Decision, Grant, resource authority or Effect truth into the source.

## Discover exact syntax

```sh
./yai case --help
./yai case provider attach --help
./yai provider --help
./yai case provider bind --help
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
runtime, remote enterprise IAM, provider-governance adversarial hardening or a
stable public API.
