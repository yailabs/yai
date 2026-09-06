# Test 02 -- Filesystem Loop Manual Validation

Status: historical fixture procedure; not current product acceptance

## Purpose

Inspect an existing daemon-backed operational path with case material, policy
material, filesystem attempts, decisions, receipts, projection, and hot-state
evidence.

## Prerequisites

- Build or install local `yai` and `yaid` binaries.
- Prepare a local runtime home and daemon socket as described by the manual.
- Stage the manual policy-pack fixtures described by the manual.

The core filesystem loop path does not require a model. Later prompt sections
in the manual may require external provider/model setup.

## Commands Or Manual Reference

Use the existing manual:

```text
work/archive/manuals/manual-filesystem-loop-validation.md
```

The archived manual records daemon-fixture setup and optional historical prompt
sections. It is not the current governed product effect acceptance procedure.
Use `make smoke-controlled-effect` for current deterministic loopback product
proof, and the current wave's manual for operator acceptance. Do not reinterpret
C fixture receipts as product-controlled effects.

## Expected Evidence

- daemon status and info responses;
- filesystem loop completion with journal path, record count, receipt count,
  projection count, and decision/receipt outcomes;
- hot-state status showing active case/session/context material;
- projection inspection showing freshness material;
- journal residue containing attempts, decisions, receipts, and projection
  records.

## Limitations

- This is still a manual validation path.
- The manual contains deeper engineering context and may be longer than a
  first-contact public test.
- Optional provider/model sections are outside the baseline repository tests.

## Deeper References

- [Public test cases](../../../docs/test-cases.md)
- [Manual filesystem loop validation](../../../work/archive/manuals/manual-filesystem-loop-validation.md)
- [Testing doctrine](../../../work/spines/testing.md)
