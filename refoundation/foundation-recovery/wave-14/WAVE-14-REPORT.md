# Wave 14 — Shared-resource authority and second carrier

Status: implementation and focused executable qualification complete; full
repository qualification and publication are recorded in `EXECUTION-EVIDENCE.md`.
The external YVEX lane is independently `blocked_external_dependency` because
the exact read-only checkout exposes no installed model and no live server.

Semantic commit: `f430624f547d65d090e94a18f92960a651ac5b5e`.
`make check`, 122 engine tests, 9 CLI tests, H10/W11/W12/W13/H13
characterizations, both Wave 14 smoke targets, formatting, repository docs and
the baseline Clippy contract all passed on that exact commit.

## Verdict

Wave 14 earns one Case-independent semantic owner: `resource_control.rs`.
The owner is justified because two Cases and two processes can name the same
physical resource, neither Case ledger can serialize the other, the logical
epoch must survive either runtime, and the carrier needs one current-writer
answer. RuntimeInstance and RuntimeWorkItem remain operational schedulers and
never own the resource lease.

`ResourceControlState v1` is Tenant-bound and stores an immutable resource
identity, monotonically increasing epoch, event sequence, and at most one
active exact fence. `ResourceControlEvent v1` retains acquired, reclaimed and
released history. Filesystem identities use canonical absolute roots;
equal/ancestor/descendant/unknown relations conflict conservatively. Process
identities use Linux PID + boot ID + `/proc/<pid>/stat` start ticks.

For new Tenant Cases, Grant validation, current temporal policy validation,
resource conflict detection, epoch acquisition, fence sealing and Case PREPARE
are one LMDB write transaction. FINALIZE/reconciled terminal truth and release
are also one transaction. INDETERMINATE retains ownership. A recovery process
may reclaim only the same unresolved effect after the exact owner dies; reclaim
increments the epoch and makes every older carrier request stale.

The carrier re-resolves canonical resource state immediately before mutation.
A fence is not bearer authority: it must match resource, Tenant, epoch, Case,
Operation, Grant, effect, current owner PID and exact current process-birth
identity. A live exact owner cannot be displaced by elapsed time alone.

The second carrier is `process.signal`. It recovers the legacy's strongest
real property—a finite semantic action mapped to a real kernel signal—without
recovering Supervisor ownership or deny-means-kill. Only an attached exact
process-birth identity may be targeted; the model may propose `terminate`,
`suspend` or `resume`, never an integer signal or raw authoritative PID. The
same policy/Decision/review/Grant/PREPARE spine is used. A successful `kill(2)`
records only kernel acceptance; observed process state is separate and no exit
is fabricated.

## Legacy verdict

Direct inspection covered `03b72f5d4`, `4e4fa4ebd`, `3e6c93e65` and the final
process map/watch, host-mediated carrier and runtime-control bridge sources.
Legacy YAI had executable process observation and SIGTERM/SIGSTOP/SIGCONT
application, but no resource epoch, carrier-resolved current fence, or stale
writer exclusion. Wave 14 is deliberately stronger than legacy here.

## Schema and ownership delta

- `yai.transition.v9` / `yai.case_state.v9`: process effect history and typed
  operation/effect kind; v1-v8 remain readable.
- `yai.operation.v2`: typed `process_signal` payload; filesystem v1 remains
  readable.
- `yai.prepared_effect.v2`: filesystem fence evidence; v1 remains readable.
- `yai.prepared_process_effect.v1`, `yai.process_observation.v1`,
  `yai.process_effect_receipt.v1`.
- `yai.resource_control_state.v1`, `yai.resource_control_event.v1`,
  `yai.resource_fence.v1`, `yai.local_process_identity.v1`.
- Derived projection/frame/rendered-input v5 add canonical Tenant-domain and
  process attachment facts. Invocation-local request attribution is not faked
  as Case truth.
- LMDB adds two named databases, 32 → 34 of 40. Process and filesystem local
  bindings share the already existing operational binding database.
- Semantic owner delta: +1 resource-control owner; the process carrier is a
  narrow mechanical extension, not an authority owner.

## Non-claims

This is local YAI-mediated fencing. It does not fence arbitrary non-YAI host
writers, provide distributed consensus, network-partition safety, remote
leases, cross-Tenant sharing, provider governance, or Workflow execution.

## External provider verdict

YVEX was treated read-only at exact checkpoint
`2df3b84cc840dfca8b38f6fc387a833169b5598e`. The repository was clean on
`main`; executable `yvex 0.1.0` reported protocol 11, no private runtime socket,
and `MODELS count=0`. The live generic provider run was therefore not
fabricated. The permanent harness and cumulative findings dossier are present.

## Foundation Recovery disposition

- shared resource identity/authority: `refounded_proven` for local YAI media;
- local carrier fencing: `refounded_proven`;
- filesystem carrier fencing: `refounded_proven`;
- process signal carrier: `refounded_proven` on Linux for test-owned processes;
- cross-process exclusion: implemented at the shared LMDB carrier boundary;
  H14 retains adversarial/high-contention qualification;
- YVEX generic compatibility: `blocked_external_dependency` at the exact
  observed checkpoint, not externally qualified.
