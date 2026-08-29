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

The current command surface can then report local paths and state:

```sh
build/bin/yai doctor
build/bin/yai hot status
build/bin/yai store status
build/bin/yai store summary
```

Depending on the build target, the Rust binary may instead be under Cargo's
target directory or installed by `make install-local`. Use `make
print-install-paths` to inspect configured install locations.

These status commands do not require a provider. Provider commands require an
explicit endpoint/model and may make real network requests. Controlled
`filesystem.write` performs real effects only through the Grant/PREPARE carrier
boundary. A review command records a typed human participant action and never
performs the effect itself; a later `yai case resume` may execute an approved
Operation through that same carrier. Use only a disposable bound root and read
[Test cases](test-cases.md) first. The former direct write command is removed.

Governance authoring also needs no Case, provider or carrier. With a constrained
`yai.policy_source_input.v2` JSON file (including bounded declared
`source_origin` provenance):

```sh
target/debug/yai policy ingest ./policy.json --as participant:policy-admin
target/debug/yai policy validate <artifact-id> --as participant:policy-admin
target/debug/yai policy publish <artifact-id> --as participant:policy-admin
target/debug/yai policy inspect <artifact-id>
```

Publication means the immutable qualified artifact is eligible for future Case
binding; Wave 8 does not bind it to a Case or turn it into authority. Use an
isolated `YAI_HOME`: these commands append a canonical governance lifecycle.

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
identity, Case PolicyBinding, EffectivePolicy or policy-driven authority.
Typed Projection/ContextFrame, local Case-native review and deterministic
Case-independent PolicyArtifact intake are implemented. Legacy JSONL/record
paths remain compatibility surfaces. Do not infer production safety from a
passing smoke suite.
