# YAI.SOURCE.REFOUNDATION.13

State at pre-publication closure: implemented and qualified; publication is
recorded only after the commit/push commands complete. Baseline:
`2166c6e98c01a9831127117280b81fd003606201`. Intended commit:
`feat: add tenant-fair multi-case runtime`. The existing `master` worktree was
used without branch, worktree, stash, reset or cleanup, and the 13 historical
dirty entries remain outside this Wave.

## Direct legacy verdict

Fresh inspection covered `68095595327a6ea11024c044ac9205496701a854`,
`5af820f68825a0d25cac97d42e7e558a8fed9a93`,
`776af9f33be01a7d8f54c4f25426a047c2781782` and stronger/final adjacent
runtime, Case registry, process-watch, supervisor and source-runtime paths.

The strongest executable legacy multi-runtime properties were: a persisted
machine lifecycle with PID reconciliation; bounded scan-many process
observation; low-level runtime-control admission distinct from Case policy; and
a durable bounded source-ingestion spool with idempotency, attempts and retry
posture. Legacy did not contain a Case worker pool, Tenant fairness, per-Tenant
quota or meaningful Case work scheduler. `scan_all` scanned process attachments,
not runnable Cases. The shell queue projected notices and equivalent commands;
it did not execute work.

Recovered properties are explicit operational lifecycle, independent runtime
identity, liveness/lease recovery, bounded durable work, scan-many recovery and
system admission distinct from Case authority. Rejected structures are
Supervisor/root-Case ownership, kernel/energy scheduler analogies, global
`authority_lock`, shell queue authority, Agent/Workflow scheduling, carrier
taxonomy, ambient root/arming and source-edge ownership topology.

## RuntimeInstance owner verdict

`RuntimeInstance` earned one independent **operational** owner because it has a
lifecycle across many Cases, owns a finite worker set and queue limits, and
survives individual Case completion. It does not own canonical semantic truth.
No new engine semantic module was introduced; the cohesive product owner is
`cmd/yai/src/runtime_instance.rs`, while LMDB owns transactional operational
persistence and the existing Case loop owns Case execution.

`yai.runtime_instance.v1` contains a catalog-local instance ID, content digest,
enrolled Principal, PID/token owner, Starting/Running/Draining/Stopped posture,
finite configuration, acquisition/heartbeat/expiry times, recovery count and
diagnostic detail. One live instance is admitted. A live second instance fails;
a stopped, expired or dead-PID owner can be reclaimed. The process observes its
kernel Principal again on every dispatch and the transactional claim resolves
the exact Tenant, Case and ownership. Instance IDs/tokens are not bearer
authority.

The lifecycle is durable operational state in LMDB, never a Case Transition.
Starting performs the bounded recovery sweep; Running accepts and dispatches;
Draining rejects submissions and dispatch, lets active Case loops reach their
safe stop boundary; Stopped releases the lease. `yai runtime serve` is a
foreground process. There is no daemon/service-manager or distributed lease.

## Work, workers and scheduling

`yai.runtime_work_item.v1` integrity-binds explicit request identity,
Principal, Tenant, Case, Participant, attachment, per-Case journal, bounded
64-KiB task, existing Case budgets, enqueue sequence, state, attempt/worker
lease, times and stop reason. An exact idempotency retry returns the same item;
same key with altered fields conflicts; identical prompt bytes with different
keys remain distinct. Queue records are noncanonical operational input and do
not become Operation/Decision/Grant until the existing Case loop reaches those
boundaries.

States with actual consumers are Queued, Running, WaitingReview, WaitingEffect,
Blocked, Completed, Denied, Cancelled and Failed. Waiting review/effect/policy
items release their thread. Live direct Case admission parks a WorkItem and is
rechecked without stealing. Cancellation/closure terminalizes unstarted work;
already prepared/indeterminate truth remains governed by Wave11 settlement.

The pool creates exactly configured worker threads once; validated bounds are
1–64 workers, a positive active-Case limit not exceeding workers, and finite
Tenant/global queue bounds. Workers call
`case_runtime::execute_runtime_work`, which uses the same run loop as direct
`case run/resume`. Checkpoint v2 binds RuntimeInstance + WorkItem + run + Case;
v1 remains readable. The scheduler does not shell out and does not duplicate the
Case runtime algorithm.

Scheduling is deterministic round-robin over eligible Tenants and FIFO by
monotonic enqueue sequence within each Tenant. It excludes active/earlier
nonterminal work for the same Case, respects per-Tenant active capacity,
revalidates security at claim, and finally acquires the existing per-Case
RuntimeAdmission. Queue admission returns typed Tenant/global capacity errors
before any Case mutation; queue/status are pure.

Known disjoint filesystem roots may run concurrently. Exact, overlapping or
unknown relations serialize inside this RuntimeInstance and emit
`serialized_due_to_resource_overlap_or_unknown_relation`. This is conservative
conflict avoidance, not a lease or fence. Wave12 still rejects cross-Tenant root
aliases before scheduling.

The prior process-global `std::env::set_var("YAI_JOURNAL", ...)` was removed.
Journal selection is explicit on each WorkItem/provider invocation. LMDB
environments are process-cached and shared through `Arc<Environment>` with
`NO_TLS`; worker code never reopens independent handles for one filesystem
environment.

## Recovery

Startup scans only stored nonterminal WorkItems. Classification order is:
cancel/close, unresolved PREPARE/INDETERMINATE, Review, checkpoint/canonical
provider result, policy/security, then safe requeue. It does not invoke a
provider while classifying. Running work owned by a dead RuntimeInstance is
reclaimed operationally; the existing Case admission/checkpoint and canonical
history still decide whether execution resumes.

Canonical ProviderResult prevents a duplicate call through the existing
single-Case recovery path. Prepared/Indeterminate effect truth is reconciled
before provider work. The product crash proof killed the complete instance at
`after_prepare_before_effect` (exit 85), restarted it, reclaimed the exact
Running WorkItem, reported a nonzero recovery count, reconciled the prepared
write and completed after exactly the fixture’s second provider turn.

## Failures discovered and fixed

Wave13 exposed three real single-Case-hidden defects; raw failure evidence is
retained in `EXECUTION-EVIDENCE.md`:

1. new provider/resource Transition IDs omitted Case identity, so two Cases at
   the same local sequence collided (`duplicate_transition_id`). New live IDs
   are Case-qualified; historical IDs remain readable;
2. independently opening one LMDB filesystem environment from multiple worker
   threads produced `MDB_BAD_RSLOT` and later a process fault. All stores in one
   process now share one environment handle, with a four-worker repeated-open
   regression test;
3. the first scheduler selector unit test accidentally opened the default LMDB
   and failed read-only under isolation. The pure selector now receives a root
   resolver; the product wrapper alone reads LMDB.

The initial AF_UNIX failure was sandbox infrastructure and was rerun unchanged
with explicit local socket permission. No authority bypass or duplicate
physical effect was hidden.

## Ownership and footprint

Canonical semantic authority remains Case Transitions/CaseState plus the
existing security/governance owners. RuntimeInstance, WorkItems, queue, worker
leases and Case checkpoint/admission are operational durable. Metrics, queue
summaries, fairness counters and worker views are derived/observational.

- tracked files: 898 → 913
- C/H/Rust source files: 158 → 159
- Rust files: 34 → 35
- engine semantic owners: 17 → 17
- operational CLI owners: +1 (`runtime_instance.rs`)
- `cmd/yai/src/main.rs`: 1,985 → 1,995 lines; additions are declaration,
  usage and dispatch only
- LMDB named databases: 29 → 32, below `set_max_dbs(40)`
- added databases: `runtime_instances`, `runtime_work_items`,
  `runtime_work_idempotency`
- canonical Transition/CaseState remain v8; CaseRuntimeCheckpoint v1 → v2

## Qualification verdict

The complete engine suite passes 110/110 and the operational CLI suite passes
2/2. `make check`, `make characterization`, all governance/H10/temporal/Tenant
smokes, `smoke-multi-case-runtime`, R1–R6, the governed 26-turn runtime and both
128-iteration endurance bodies pass. Formatting, layout, docs and
`git diff --check` pass.

The repository's unwaived `clippy -D warnings` audit still reports known
pre-Wave13 warnings in historical code. After allowing only those enumerated
baseline lint categories, both engine and CLI Clippy runs are clean; Wave13
introduces no new diagnostic. The exact commands, exits and representative raw
outputs are retained in `EXECUTION-EVIDENCE.md` rather than converting that
baseline debt into a false green claim.

## Foundation Recovery classification

- Runtime administrative root / RuntimeInstance lifecycle:
  `refounded_proven` for the local foreground runtime.
- Multi-Case runtime and bounded worker pool: `refounded_proven` locally.
- Tenant-fair scheduling: `refounded_proven` for deterministic unweighted
  round-robin + Tenant FIFO.
- quotas/backpressure: `refounded_proven` for operational capacity, explicitly
  not billing or policy entitlement.
- recovery sweep: `refounded_proven` for durable WorkItems and the current
  provider/filesystem Case runtime.
- Tenant runtime isolation: `refounded_proven` within the Wave12 local POSIX
  trust boundary.
- shared resource concurrency: still `missing/deferred`; serialization is not
  fencing.

## Exact Wave14 delta

Wave14 remains: direct archaeology for one canonical shared-resource lease
owner, resource generation/epoch, fencing token enforced at carrier admission,
cross-process resource exclusion and recovery, plus one second real carrier and
its PREPARE/effect/reconcile proofs. Wave13 introduced none of those. It also
did not add provider governance, remote workers, cluster scheduling, priorities,
billing, SSO, retention, membership removal or distributed locks.
