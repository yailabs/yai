# Test Cases

`tests/cases/` is a public test-case documentation and wrapper layer. It points
to the current validation paths without taking ownership away from the lower
level smoke tests and manual runbooks.

## Organization

- `00-repository-health/`: repository status and broad check path.
- `01-local-runtime-inspection/`: no-model runtime, hot-state, store, and daemon
  inspection.
- `02-filesystem-loop-manual/`: manual daemon-backed filesystem loop.
- `03-model-behavior-lab/`: optional model/provider lab artifact reference.

## Relationship To Smoke Tests

`tests/smoke/` contains implementation-level smoke checks. The case directories
here describe runnable or inspectable validation paths for technical readers.
They may point to smoke tests, Makefile targets, manuals, or curated lab
artifacts.

## Status Labels

- `current`: can be run from the current repository surface.
- `manual`: requires local setup or a manually prepared runtime.
- `experimental`: exists, but output or behavior may change.
- `internal`: primarily maintainer or engineering validation.
- `planned`: documented target, not a current runnable test.

## Model Requirements

The local gates already include deterministic repository-owned provider
fixtures. They exercise real generic adapter transport without deploying YVEX.
Real-model interoperability is the opt-in external lane, not a local fixture
claim. See `tests/README.md` for the current proof/provider axes.

## Current Limitations

- These are documentation wrappers, not new test harnesses.
- Some command output remains subject to command-surface stabilization.
- Manual cases may require locally built binaries and prepared runtime paths.
- Optional model labs require external provider/model setup and are not
  deterministic.
