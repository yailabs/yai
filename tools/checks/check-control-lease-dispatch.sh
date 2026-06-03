#!/usr/bin/env sh
set -eu

# YAI - control lease dispatch guard (CORE.ENFORCE.1)
#
# Purpose:
#   Keep carrier execution gated behind a permitting CapabilityLease and an
#   allow decision, and keep skeleton/model_provider carriers non-executing.
#
# Scope:
#   Behaviour/grep checks over the admission gate, the lease predicate, the
#   executable carriers, the skeleton registry and the smoke wiring.
#
# Non-goals:
#   Does not run the build. Does not validate runtime dispatch wiring beyond
#   the admission contract.

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
cd "$ROOT"

fail() {
  printf 'check-control-lease-dispatch: %s\n' "$1" >&2
  exit 1
}

require_file() {
  [ -f "$1" ] || fail "missing file: $1"
}

ADMIT_C="system/effect/dispatch_admission.c"
ADMIT_H="include/yai/effect/dispatch_admission.h"
LEASE_C="system/control/capability_lease.c"
FS_CARRIER="system/effect/carriers/filesystem_carrier.c"
PROC_CARRIER="system/effect/carriers/process_carrier.c"
SKELETON="system/effect/carrier_skeleton.c"
SMOKE="tests/smoke/control-lease-dispatch/test_control_lease_dispatch.c"

require_file "$ADMIT_C"
require_file "$ADMIT_H"
require_file "$LEASE_C"
require_file "$SKELETON"
require_file "$SMOKE"

# Admission gate must validate the lease AND check the decision allow.
grep -Fq 'yai_capability_lease_permits_execution' "$ADMIT_C" ||
  fail "admission gate does not validate the lease"
grep -Fq 'YAI_DECISION_ALLOW' "$ADMIT_C" ||
  fail "admission gate does not check decision allow"
grep -Fq 'execution_admitted' "$ADMIT_C" ||
  fail "admission gate has no execution_admitted decision"

# Lease predicate must exist and be fail-closed (default/non-minted -> no exec).
grep -Fq 'yai_capability_lease_permits_execution' "$LEASE_C" ||
  fail "lease execution predicate missing"

# Executable carriers must keep their decision/safety gate and reference the
# CORE.ENFORCE.1 admission boundary.
grep -Fq 'CORE.ENFORCE.1' "$FS_CARRIER" ||
  fail "filesystem carrier missing lease-dispatch note"
grep -Fq 'CORE.ENFORCE.1' "$PROC_CARRIER" ||
  fail "process carrier missing lease-dispatch note"
grep -Fq 'YAI_DECISION_ALLOW' "$FS_CARRIER" ||
  fail "filesystem carrier dropped its decision gate"

# Skeleton carriers must never claim execution.
if grep -Eq 'EXECUTED|execution_performed[[:space:]]*=[[:space:]]*(1|true)|carrier_attempted[[:space:]]*=[[:space:]]*1' "$SKELETON"; then
  fail "skeleton carrier path claims execution"
fi

# Every skeleton registry entry must keep execution_available = 0
# (the ", 0, 1, 1," triplet after the four adapter-status fields).
entries="$(grep -n '{YAI_CARRIER_FAMILY_' "$SKELETON" || true)"
if [ -n "$entries" ]; then
  printf '%s\n' "$entries" | while IFS= read -r line; do
    case "$line" in
      *", 0, 1, 1,"*) : ;;
      *) printf 'check-control-lease-dispatch: skeleton entry is execution-capable: %s\n' "$line" >&2; exit 1 ;;
    esac
  done
fi

# model_provider must remain a non-executing skeleton.
grep -Eq 'YAI_CARRIER_FAMILY_MODEL_PROVIDER.*"model_provider".*, 0, 1, 1,' "$SKELETON" ||
  fail "model_provider is not a non-executing skeleton"

# Smoke test must prove the negative and positive cases.
for label in \
  "admit:no_lease denied" \
  "admit:subject_only denied" \
  "admit:proposal_no_lease denied" \
  "admit:deny_defer_review no_execute" \
  "carrier:no_lease not_executed" \
  "carrier:deny_decision blocked" \
  "carrier:lease_allow executed" \
  "skeleton:model_provider no_execution"; do
  grep -Fq "$label" "$SMOKE" || fail "smoke missing label: $label"
done

# Makefile wiring.
grep -Fq 'smoke-core-enforce1' Makefile || fail "Makefile missing smoke-core-enforce1"
grep -Fq 'check-control-lease-dispatch' Makefile || fail "Makefile missing check-control-lease-dispatch"

printf 'check-control-lease-dispatch: ok\n'
