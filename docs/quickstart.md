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
`filesystem.write` and approved review commands perform real effects through
the Grant/PREPARE carrier boundary; use only a disposable bound root and read
[Test cases](test-cases.md) first. The former direct write command is removed.

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
policy system, automatic recovery service, race-resistant hostile-namespace
confinement, or provider-independent ContextFrame implementation. Legacy
JSONL/record paths remain compatibility surfaces. Do not infer production
safety from a passing smoke suite.
