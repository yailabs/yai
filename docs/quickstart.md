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
explicit endpoint/model and may make real network requests. Filesystem review
and direct write commands may change local files; use only the repository's
fixtures or a disposable sandbox and read [Test cases](test-cases.md) first.

## What to read next

- [Executable architecture](architecture.md) explains what the binaries
  actually implement and where their authority diverges.
- [Test cases](test-cases.md) maps reproducible tests to the claims they support.
- [Implementation roadmap](../ROADMAP.md) records gaps; it is not current
  behavior.
- `tests/smoke/` is implementation evidence. `labs/` and `work/` are
  experimental/historical evidence and are not canonical documentation.

## Limitations

YAI now has a typed LMDB Transition/CaseState transaction authority, but no
general carrier admission boundary or provider-independent ContextFrame
implementation. Legacy JSONL/record paths remain compatibility surfaces, and
the controlled filesystem path is fixture-bound and still writes before its
terminal transition is committed. Do not infer production safety from a
passing smoke suite.
