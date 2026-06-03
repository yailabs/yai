# CORE.ENFORCE.1 - Control Lease Dispatch Verification

Status: implemented (partial enforcement)

Macro:

```text
CONTROL
```

Type: enforcement/verification wave.

## What Was Verified

Traced the real path: `op_attempt → yai_control_decide → decision → carrier
checks decision.outcome == ALLOW → execute`.

Findings:

- Executable carriers (filesystem, process) are gated by the control **decision**
  (filesystem requires ALLOW/ALLOW_WITH_CONSTRAINTS; process requires the
  test-owned/spawned-by-yai safety posture). This gate works.
- The **CapabilityLease** (SPINE.51B) was derived and inspectable but **not
  consumed** by the C carrier path — `lease`/`capability` appears only in
  `cmd/yai/src/main.rs` and `system/control/capability_lease.c`, never in
  `system/effect/dispatch*.c` or `system/effect/carriers/`.
- Carriers are reached **only from smoke tests**; there is no runtime dispatch
  path yet. Skeleton carriers (network_http … model_provider) have no execution
  code (`execution_available = 0`).

Answer to the wave question: an operation could reach the (test-only) carrier
path on an ALLOW decision **without a CapabilityLease** — lease-before-dispatch
was **not** enforced. Now partially enforced via a fail-closed admission gate.

## Code / Tests Changed

Added:

```text
include/yai/effect/dispatch_admission.h   admission outcome + yai_dispatch_admit
system/effect/dispatch_admission.c        lease+decision admission (fail-closed)
tests/smoke/control-lease-dispatch/test_control_lease_dispatch.c
tools/checks/check-control-lease-dispatch.sh
work/waves/core-enforce-1-control-lease-dispatch-verification.md
```

Changed:

```text
include/yai/control/capability_lease.h    + yai_capability_lease_permits_execution
system/control/capability_lease.c         fail-closed execution predicate
include/yai/yai.h                          export dispatch_admission.h
system/effect/carriers/filesystem_carrier.c  CORE.ENFORCE.1 note (gate unchanged)
system/effect/carriers/process_carrier.c     CORE.ENFORCE.1 note (gate unchanged)
Makefile                                   C source, smoke + guard wiring
work/spines/core-enforcement-status.md     capability lease row
work/spines/core-properties.md             CP3 evidence/status
work/spines/core-hardening-index.md        CORE.ENFORCE.1 delivered (partial)
```

New rule (`yai_dispatch_admit`): execution is admitted only when a minted lease
permits execution AND the decision allows. Review/defer/evidence/redaction/
observe → review (no execution). Deny, missing decision, or allow-without-lease
→ deny. Default is fail-closed.

## Enforcement Posture

Lease-before-dispatch is now **partially enforced**: the admission gate is
deterministic, fail-closed and tested. It is not yet wired into carrier
signatures, so carriers still independently re-check the decision as defense in
depth. CapabilityLease therefore stays `implemented_limited`.

## Command Surface

No command surface change.

## Validation

```text
make smoke-core-enforce1          all admission/carrier/skeleton assertions pass
build-c                           compiles clean under -Wall -Wextra -Werror
make check-control-lease-dispatch ok (self-tested: fails if a skeleton becomes
                                      execution-capable)
make check-spine-consistency      ok
make check-layout / make info     ok
```

Targeted: no `execution_performed = 1/true` in skeleton paths; model_provider
stays `execution_available = 0`; carriers retain their decision gate.

## Next Wave

CORE.CARRIER.1 — Process Carrier Hardening, carrying the carrier-signature
rewiring so executable carriers require the admission token directly.
