# Wave 18 failure evidence

## F18-01 — new runtime posture initially absent from FSM

- run_id: `W18-INITIAL-CHECK-20260902`
- order: 1
- pre-state: first W18 integration build
- cwd: repository root
- command: `make build-rust`
- exit: 2
- raw excerpt:

```text
test store::lmdb::tests::h13_runtime_work_fsm_is_explicit_and_terminal_states_are_closed ... FAILED
Running -> DeliveryIndeterminate
test result: FAILED. 191 passed; 1 failed
make: *** [Makefile:366: build-rust] Error 101
```

- cause: the H13 operational FSM did not yet admit the load-bearing W18
  no-retry terminal posture.
- correction: `DeliveryIndeterminate` was added as an explicit terminal state;
  later engine suite: `195 passed; 0 failed`.

## F18-02 — explicit Case selected an unrelated compatibility journal

- run_id: `W18-PRODUCT-PREFIX-20260902`
- order: 2
- pre-state: fresh temporary YAI_HOME, governed Case never executed
- cwd: repository root
- command: `make smoke-provider-governance`
- exit: 2
- raw excerpt:

```text
runtime_status: DeliveryIndeterminate
stop_detail: journal_case_identity_mismatch: expected=case:w18-smoke observed=case:new12-filesystem record_id=rec:new12-fs-case
invocations: 0
provider_failures: 1
```

- cause: `case_journal_path` searched the working tree before deriving the
  explicit Case's compatibility path, and the runtime's fallback classifier
  called an unknown local error remote-delivery indeterminate.
- correction: explicit Case now selects its YAI_HOME-scoped journal first;
  only explicit transport prefixes can become delivery indeterminate; local
  setup failures are zero-byte/not-dispatched. The identical product lane now
  completes with one invocation and zero provider failures.

## F18-03 — governed selection lacked invocation-scoped projection admission

- run_id: `W18-PRODUCT-PROJECTION-20260902`
- order: 3
- pre-state: qualified, approved and selected governed target; Participant had
  exact model-executor role but no interactive shell view
- cwd: repository root
- command: `tests/characterization/provider-governance/test_provider_governance.sh`
- exit: 1
- raw excerpt:

```text
warning: qualified memory retrieval failed; using canonical fallback: retrieval_view_not_admitted
provider_safe_failover: attempt=2 reason:provider_not_dispatched:local_projection:projection_view_not_admitted
```

- cause: the governed product lane incorrectly depended on the legacy
  interactive `case enter` admission.
- correction: an exact canonical ProviderSelection now derives a bounded
  model projection admission for that invocation only. Invocation start still
  transactionally revalidates binding, target, qualification, trust and
  circuit before network dispatch. No ambient interactive view is added.
- post-fix evidence: `governed_provider_modelwork_completed: true`.

No reproduced W18 run accepted an unqualified/denied/cross-Tenant target,
forged capability, duplicate semantic result, or automatic failover after
possible delivery. Remaining adversarial expansion is explicitly assigned to
H18 rather than reported as a W18 pass.

## F18-04 — local budget classification lost through transport prefixing

- run_id: `W18-MAKE-CHECK-20260902-25feba`
- order: 4
- pre-state: W18 delivery prefixes active; lower-wave agentless runtime smoke
- cwd: repository root
- command: `make check`
- exit: 2
- raw excerpt:

```text
provider_retry: 1 reason:provider_not_dispatched:local_projection:residency_budget_below_mandatory_state: required_items=5 max_items=12 required_units=504 max_units=1
make: *** [Makefile:582: smoke-agentless-case-runtime] Error 1
```

- cause: the Case runtime recognized the budget class only at byte zero of the
  error, before W18 added an explicit local/pre-dispatch envelope.
- correction: budget exhaustion is classified from the typed inner marker and
  remains `ContextBudgetExhausted`; it is not retried as provider transport.
