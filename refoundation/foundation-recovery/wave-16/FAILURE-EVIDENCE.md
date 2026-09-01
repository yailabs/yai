# Wave 16 failure evidence

Baseline: `67fbf2a96b924f596caf0bc74976e402eda34bc6`.

The unedited bounded pre-fix transcript was captured before implementation in
`/tmp/yai-w16-manual-failures-pre.log` from the preserved baseline binary. The
load-bearing failures were:

- `yai doctor` exited 0 and began `yai doctor: ok` while `run`, `store`, `log`,
  `tmp`, `cases`, `sockets` and `config` were all reported missing;
- both `case status` and `case stop` on a valid never-run Case exited 2 and
  leaked the missing checkpoint pathname;
- `tenant status` exited 2 with `--tenant is required` despite its usage line
  presenting the selector as optional;
- the two process-observation spellings both dispatched independently;
- bare `yai` exited 2 instead of presenting a product map.

`manual-failure-reproduction.tsv` binds each reproduction to the same semantic
post-fix check. No transcript was reconstructed as raw output. The final
product run retained in `EXECUTION-EVIDENCE.md` reruns the same properties.

No failure was found in H15 workflow truth or lower-wave authority owners. W16
therefore changes projection/admission behavior and fixes the two checkpoint
product interpretations without adding a semantic owner.
