# H15 Failure Evidence

## F-H15-TEST-01 — direct non-owner materialization rejected

The first independent-process test design attempted to call workflow
materialization from processes that did not own the current RuntimeInstance.
The unedited failure was:

```text
called `Result::unwrap()` on an `Err` value: "runtime_instance_owner_process_mismatch"
test store::lmdb::tests::h15_process_workflow_start_contender ... FAILED
```

Classification: `NO_ISSUE`. The product correctly rejected the attack. The
final test races all eight processes through RuntimeInstance acquisition; the
single exact owner then materializes one node start while seven processes
observe `runtime_instance_active`.

## F-H15-ENV-01 — restricted socket environment

The first ModelWork characterization attempt inside the default sandbox could
not bind the local fixture daemon Unix socket and ended before any YAI semantic
scenario. Re-running the identical product script with permission for its
isolated local socket passed. Classification: test deployment limitation, not
a YAI or YVEX defect.

No valid forged progression, duplicate start, duplicate Operation/effect,
replay divergence, Definition fallback or budget-reset failure remained in the
final H15 runs.
