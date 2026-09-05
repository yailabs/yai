# I02 failure and falsification evidence

I02 did not manufacture a pre-fix runtime defect: at the reconciled baseline the
requested semantic binding/planning property did not exist. Qualification kept
the absence distinct from implementation failures.

Real negative results retained by the I02 tests include:

- a target named `whisper-vision-deepseek-bge` produced no speech-to-text or
  image-understanding suitability without exact evidence;
- a qualified/trusted target outside the current Case provider envelope was
  rejected with `cognitive_target_not_admitted_by_provider_envelope`;
- evidence from another Tenant/target was rejected with
  `semantic_suitability_evidence_binding_mismatch`;
- replacing a primary binding without explicit replacement was rejected with
  `cognitive_binding_replacement_requires_replace`;
- changing the provider envelope made the old cognitive binding unresolved as
  `primary_target_not_admitted`; it did not silently remap;
- wrong-capability evidence and an unauthenticated Principal could not replace
  the binding;
- cross-lane/cross-target continuation references were rejected while a
  missing continuation retained a valid semantic plan.

The first sandboxed aggregate run reported two CLI transport tests as
`Operation not permitted` when opening local sockets. The same unchanged tests
passed when rerun with loopback socket permission. This is classified as a test
environment limitation, not a YAI or YVEX defect.

The full characterization recreated ignored `engine/target` and
`cmd/yai/target` build directories because historical scripts do not all use
the repository root target directory. The first subsequent layout check found
generated `aws-lc-sys` C sources below `cmd/yai/target` and failed as designed.
Only those two rebuildable ignored build directories were removed; the repeated
layout/docs check passed. This is build-artifact hygiene, not a semantic I02
failure.

No YVEX endpoint was invoked. An absent or stopped fixture endpoint did not
prevent planning, which is the required I02 no-execution posture.
