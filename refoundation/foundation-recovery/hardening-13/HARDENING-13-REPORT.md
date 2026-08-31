# YAI Foundation Hardening 13 Report

State: implementation and full qualification complete; publication pending at dossier capture.

Baseline: `5e1ae2fa286cb1052d480c79ed3b84410d881035` (`test: bind Wave 13 evidence to one run`). Semantic Wave-13 baseline: `d5c2820531d10340d8b7e560b1736e68b7b17da8`.

## Direct recovery verdict

H13 re-read the requested yai-dev epochs and executable paths rather than treating the Wave-13 dossier as authority. The strongest executable legacy mechanisms were a persisted runtime-machine state transition table with temp/rename publication, PID liveness through `kill(pid, 0)`, a real `process_watch_scan_all`, and a source spool with queued/delivered/failed directories, attempts, retry delay and idempotency keys. The later `runtime/execution/process_replay.c` was a stub. No inspected epoch bound PID to process birth, connected semantic completion to durable scheduler acknowledgement, provided restart-stable Tenant fairness, or contained a worker-panic contract. H13 is deliberately stronger than legacy on all four points.

Rejected topology remains: Supervisor as semantic owner, shell queue as Case-work authority, root Case as scheduler root, source/edge spool as Case recovery, carrier taxonomy, and daemon topology for its own sake.

## Failure reproduced and closed

The unchanged Wave-13 recovery bug was reproduced with only an observation failpoint added. A real WorkItem reached a `Completed` Case checkpoint and the scheduler exited before `Running → Completed`. Restart unconditionally requeued it; `update_resume_budgets` reset `Completed → Running`; a second provider connection was attempted. The one-shot provider refused it, leaving the WorkItem incorrectly `Failed`. There was one canonical ProviderResult and no Operation/Decision/Grant/effect duplication, but semantic terminal state was genuinely resurrected. Raw pre-fix output is preserved in `FAILURE-EVIDENCE.md`.

The unchanged reproduction now yields `provider_count=1`, `final_state=Completed`, and no second dispatch. Recovery reads only the exact checkpoint for that RuntimeInstance/WorkItem and repairs the operational WorkItem directly.

Full repository qualification then exposed a second compatibility defect: a
persisted Wave-13 `meta:runtime_instance_schema=yai.runtime_instance.v1`
marker was rejected before the deliberately retained v1 record reader could
run. The write boundary now admits only that exact known predecessor, upgrades
the marker transactionally to v2, and still rejects unknown/future schemas.
The unchanged `make check` rerun passes. This failure and fix did not mutate
Case truth or introduce a permissive compatibility fallback.

## Contracts implemented

`RuntimeWorkState::permits_transition_to` is the sole bounded WorkItem transition algebra. `update_runtime_work_state` rejects invalid old→new edges; terminal states have no outbound edges. Recovery is permitted to move stale `Running → Queued` only when the exact checkpoint is still nonterminal or absent. It maps terminal/parked checkpoint posture directly to the matching WorkItem posture.

Budget override and semantic resume authorization are separate. A budget change cannot reset `Completed`, `Denied`, `Cancelled`, `Closed` or fatal checkpoint state. Runtime WorkItems also treat operator stop, malformed output and exhausted budgets as terminal rather than automatic retry. Direct operator resume retains only the explicitly nonterminal compatibility cases.

RuntimeInstance evolved to operational schema `yai.runtime_instance.v2`. Ownership is the conjunction of authenticated Principal, owner token, PID and Linux process birth fingerprint (`boot_id + /proc/<pid>/stat starttime`). PID equality alone is insufficient. A live matching process prevents lease-only takeover even after heartbeat expiry and may renew after delay; a dead or PID-reused owner is reclaimable. RuntimeInstance identity remains stable across process restart, while process ownership does not.

The external `runtime stop` command records drain request intent. Only the owning process can turn that request into `Draining` and eventually `Stopped`. Worker panic is caught at the worker boundary, recorded as fail-closed Draining detail, and terminates the scheduler nonzero without inventing a semantic Failed/Completed result. A later process sweeps the still-Running WorkItem from checkpoint and canonical Case truth.

`last_dispatched_tenant` is integrity-bound operational state and is committed in the same LMDB transaction as a WorkItem claim. Restart therefore continues round-robin after the last dispatched Tenant. It is not a Case Transition and deleting it can affect only next dispatch order.

Checkpoint publication now writes a unique temp file, syncs it, atomically renames it and syncs the parent directory. Stale temp files are ignored. Checkpoint v2 remains bound to stable RuntimeInstance ID, WorkItem, run and Case, not to a dead process token.

The compatibility journal is still semantic input to Projection/ContextFrame/provider rendering. It is therefore mechanically Case-qualified at submission and again before provider use: every loaded Record must carry the target `case_ref`. A caller pointing Case B at Case A's journal receives `journal_case_identity_mismatch` before provider invocation.

The shared LMDB environment now remembers its requested map size. A second live open of the same canonical path with a different size fails explicitly instead of silently reusing the first size. Eight-reader/writer and heartbeat contention tests produced no `MDB_BAD_RSLOT` or reader leak. H13 added no named DB: usage remains 32/40.

## Fairness and scale

The real repeated-crash proof dispatched Tenant A, crashed after claim, restarted and dispatched Tenant B, crashed again, then completed four WorkItems with two workers. The persisted cursor prevented lexicographic restart bias.

Terminal-history scale was characterized at 100, 1,000 and 5,000 WorkItems. Both LMDB listing and selection remain linear; the final focused run listed 5,000 items in 229.537 ms and selected from 5,002 in 0.157 ms. With a fixed 50 ms idle tick, historical scanning is visible but not yet a demonstrated capacity breach. No speculative active-work DB was added; terminal retention/indexing remains later operational work.

## Qualification verdict

`make check`, `make characterization`, the H10 authority hardening, Wave-11
temporal governance, Wave-12 Tenant security, Wave-13 multi-Case runtime and
the new H13 smoke all pass in one unchanged tree. Full Rust qualification is
118 engine tests and 9 CLI tests; focused H13 qualification is 8 engine tests
and 7 CLI tests. R1-R6, A1-A6, the governed 26-turn runtime, both 128-iteration
endurance bodies, PREPARE recovery, Tenant isolation and the multi-Case product
flow remain green. Formatting, layout, docs and whitespace checks pass.

The repository's existing Clippy warning baseline remains 14 engine-library
and 17 CLI-binary warnings. H13 initially added four test-only and four CLI
diagnostics; those were fixed before closure. The final H13 delta adds no
warning to the existing baseline.

## Ownership and source footprint

Semantic owners added: zero. `runtime_instance.rs` still owns scheduler/pool/recovery policy; `case_runtime.rs` owns checkpoint and resume eligibility; `store/lmdb.rs` owns operational transactions/FSM/process ownership; `provider.rs` enforces the existing journal input boundary. `main.rs` remains parse/dispatch only and is unchanged at 1,995 lines (baseline and H13).

H13 source paths are `Makefile`, `cmd/yai/src/{case_runtime,provider,runtime_instance}.rs`, `engine/yai-engine/src/store/lmdb.rs`, one characterization script, and this dossier. Transition and CaseState schemas are unchanged. No Wave-14 ResourceLease, fencing token, resource epoch, cross-process resource lock or second carrier was introduced.

## Recovery classification

- RuntimeInstance lifecycle: `refounded_proven + process-identity-qualified` for the local Linux/POSIX trust model.
- multi-Case runtime: `refounded_proven + crash-ack-qualified`.
- fairness: `refounded_proven + restart-stable` for local Tenant round-robin/FIFO.
- worker failure containment: `refounded_proven` for caught local worker panic and process restart.
- recovery sweep: `refounded_proven + terminal-ack-qualified`.
- resource concurrency: still partial; scheduler-local conservative serialization is not fencing.

## Exact Wave-14 delta

Wave 14 must freshly recover canonical shared-resource ownership: resource lease/generation, fencing token validation at the carrier boundary, cross-process direct-run versus RuntimeInstance exclusion, stale-writer rejection, crash/reclaim behavior, and a second real carrier. H13 provides no claim of cross-process filesystem safety and authorizes none of that implementation.
