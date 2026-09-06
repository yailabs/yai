# Test 00 -- Repository Health

Status: current

## Purpose

Verify that the repository can report its current status and run the broad
repository check path.

## Prerequisites

- Work from the repository root.
- Use the local toolchain expected by the Makefile.

No external model/provider, API key or manual runtime setup is required. The
gate starts its own deterministic loopback providers and test daemons.

## Commands

```sh
make info
make check
```

## Expected Behavior

`make info` prints repository status and main path information.

`make check` runs the publication union documented in `tests/README.md`: guards,
build, unit/component, provider contract, product, recovery and retained bounded
endurance. Use `make test-fast` while coding.

## Failure Interpretation

- Layout failures usually mean repository paths or required files are out of
  sync with current guards.
- Documentation failures usually mean public or engineering docs no longer meet
  guard expectations.
- Build and smoke failures should be read from the failing target output.
- Failures from unrelated dirty work should be reported separately.

## Deeper References

- [Public test cases](../../../docs/test-cases.md)
- [Testing doctrine](../../../work/spines/testing.md)
- [Repository README](../../../README.md)
