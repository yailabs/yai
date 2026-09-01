# Wave 17 failure evidence

## W17-F01 — CLI registry flag mismatch found before closure

- run_id: `w17-cli-preclosure`
- pre-state: new Handoff handler existed, registry descriptor for reconcile
  still exposed the Workflow patch flag
- exact command: `./yai case handoff reconcile case:w17-source --handoff
  handoff:<id>`
- exit: 2
- raw stderr: `yai: yai.case.handoff.reconcile: unknown option --handoff`
- defect: interface metadata selected `--patch` while the one handler consumed
  `--handoff`
- correction: the registry descriptor now uses the shared exact Handoff ID
  argument; product evidence executes the identical semantic operation with
  `--handoff` and exits 0.

## Preserved negative proofs

The executable W17 matrix additionally records the actual refusals without
fabricating transcripts:

- stale patch after a competing adoption: `workflow_patch_stale`;
- model prose instead of strict patch JSON:
  `workflow_model_plan_patch_invalid`;
- adoption while the proposing ModelWork execution is active:
  `workflow_amendment_requires_quiescent_boundary`;
- second active Case wait edge forming A → B → A: refused;
- cross-Tenant offer: refused with no target payload;
- eight simultaneous acceptance writers: one commit, seven refusals.

No lower-wave authority, resource, Workflow replay or CLI architecture defect
was observed during focused W17 qualification.

## W17-F02 — sandbox socket limitation, not product failure

- run_id: `w17-characterization-sandboxed`
- exact command: `make characterization`
- exit: 2
- raw stderr: `failed to start ipc server: invalid`
- diagnosis: the restricted execution sandbox denied AF_UNIX `socket/bind`;
  direct `/tmp/y17.sock` reproduction failed before application service start
- identical rerun: `make characterization` with permission for local Unix
  sockets
- rerun exit: 0, including `daemon:started`, `ipc:status ok`, and
  `daemon:shutdown ok`
- classification: qualification-environment limitation; no YAI source change
  made to mask it
