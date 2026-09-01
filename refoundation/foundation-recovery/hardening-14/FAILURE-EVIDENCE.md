# H14 failure evidence

No content-valid forged fence, double writer, duplicate epoch, symlink escape,
duplicate process signal, PID-reuse signal, or partial terminal release was
observed in the final H14 implementations. No failing physical transcript is
invented.

## H14-F01 — published Wave-14 resource-busy work loss

- source: published Wave-14 execution evidence W14-E03
- baseline: `bdda5a707e1286c4586f3e3ce2b3ef315342c6b0`
- exact reproduction: `tests/characterization/shared-resource-fencing/test_cross_process_fencing.sh`
- before: `runtime_work_state: Failed` with `runtime_block_reason: resource_temporarily_owned`
- semantic defect: a transient PREPARE admission conflict terminalized the
  operational WorkItem although the Decision remained ALLOW and no effect ran
- fix: map the stop to `Blocked`, release the worker, wait for a resource-state
  change, and resume the same WorkItem only through canonical policy/time/
  generation and Grant freshness checks
- unchanged reproduction after fix: recorded in `EXECUTION-EVIDENCE.md` as
  `runtime_work_initial_state: Blocked` and
  `runtime_work_final_state: Completed`

## H14-F02 — pathname TOCTOU found by production-boundary inspection

- baseline behavior: the Wave-14 carrier canonicalized the attachment root but
  re-resolved the target pathname after PREPARE and fence validation
- risk: a parent component could be renamed and replaced with an outside
  symlink before the physical open
- destructive pre-fix exploit: not executed, so no raw failure is claimed
- fix: Linux `openat2` constrained beneath a verified root descriptor, then
  descriptor-relative temporary-file creation and atomic rename
- deterministic unchanged attack after fix:
  `h14_post_prepare_parent_swap_cannot_escape_open_directory_descriptor`
- result: the write remains on the opened original directory; outside data is
  unchanged

The first unprivileged smoke attempt also hit a sandbox Unix-socket denial. The
same command passed with the normal approved product-test permissions; this is
environmental evidence, not a YAI defect.
