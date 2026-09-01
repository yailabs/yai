# Repository quickstart

Authority: current local build and inspection procedure. This is not a
production deployment guide or a stable CLI compatibility promise.

## Prerequisites

Work from the repository root with GNU Make, a C toolchain, Cargo/Rust, and the
native dependencies expected by the Makefile. Commands store mutable runtime
data under `YAI_HOME` when set, otherwise under the implementation's local
default; use an isolated temporary path for evaluation.

## Inspect and validate the repository

```sh
make info
make check-docs
make check-layout
make build
```

`make check` additionally runs the complete smoke suite. It is broader and
slower than the orientation path:

```sh
make check
```

Build products include the Rust `yai` command, the narrow C `yaid` daemon, and
component/smoke executables. Building all components does not imply that all C
library modules are reachable from `yaid`.

## Use an isolated runtime home

After a build, select an empty evaluation directory rather than an existing
operator store:

```sh
export YAI_HOME=/tmp/yai-evaluation-home
```

The repository-root `./yai` launcher is the public local-development entrypoint.
It delegates to the built Rust product without exposing Cargo's internal output
path. The current command surface can then report local paths and state:

```sh
./yai doctor
./yai hot status
./yai store status
./yai store summary
```

Installed deployments may invoke `yai` through `PATH` after `make
install-local`. Use `make print-install-paths` to inspect configured install
locations.

These status commands do not require a provider. Provider commands require an
explicit endpoint/model and may make real network requests. Controlled
`filesystem.write` performs real effects only through the Grant/PREPARE carrier
boundary. A review command records a typed human participant action and never
performs the effect itself; a later `yai case resume` may execute an approved
Operation through that same carrier. Use only a disposable bound root and read
[Test cases](test-cases.md) first. The former direct write command is removed.

Governance authoring also needs no Case, provider or carrier. With a constrained
`yai.policy_source_input.v3` JSON file (including bounded declared
`source_origin` provenance):

```sh
./yai policy ingest ./policy.json --as participant:policy-admin
./yai policy validate <artifact-id> --as participant:policy-admin
./yai policy publish <artifact-id> --as participant:policy-admin
./yai policy inspect <artifact-id>
```

Publication makes the immutable qualified artifact eligible for an exact Case
binding; it is not authority by itself. Use `yai case policy bind` to pin an
artifact, `yai case bind-participant-role` to record required Case roles, and
`yai case policy status` to inspect derived readiness. New live governed
operations require Ready policy with an explicit applicable ALLOW; evaluation
then records DecisionBasis/Decision and only final ALLOW can issue a Grant.

## What to read next

- [Executable architecture](architecture.md) explains what the binaries
  actually implement and where their authority diverges.
- [Test cases](test-cases.md) maps reproducible tests to the claims they support.
- [Implementation roadmap](../ROADMAP.md) records gaps; it is not current
  behavior.
- `tests/smoke/` is implementation evidence. `labs/` and `work/` are
  experimental/historical evidence and are not canonical documentation.

## Limitations

YAI has typed LMDB Transition/CaseState authority and one controlled local
`filesystem.write` vertical: durable PREPARE precedes mutation, and observed
outcome is finalized or reconciled afterward. It has no general carrier or
policy language, automatic multi-Case recovery service, race-resistant hostile-
namespace confinement, distributed run admission, authenticated remote human
identity, expiry/revoke, or general RBAC/ABAC. Exact Case PolicyBinding,
EffectivePolicy, policy-driven local-role admission, typed
Projection/ContextFrame and Case-native review are implemented. Legacy JSONL/record
paths remain compatibility surfaces. Do not infer production safety from a
passing smoke suite.
