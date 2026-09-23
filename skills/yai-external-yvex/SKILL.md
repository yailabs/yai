---
name: yai-external-yvex
description: Qualify YAI against an explicitly selected live YVEX provider endpoint as a black-box consumer.
---

# External YVEX qualification

Use only when live YVEX qualification is explicitly selected. It is an
independent evidence axis, not a default gate for unrelated YAI changes.
Follow the [external lane](../../docs/test-cases.md#external-yvex).

Consume the operator-supplied endpoint and exact provider-exposed model
identity through YAI's generic OpenAI-compatible provider boundary. Do not
inspect or administer YVEX source, repository, CLI, profiles, engines,
artifacts or sessions. Do not add a provider-brand authority or semantic
special case. Record YAI SHA, endpoint, model and run ID for every attempt.

Classify unavailable deployment or timeout honestly; do not fabricate PASS or
turn a characterization limitation into an unrelated work stop. Keep model
behavior, generic provider-contract defects, YAI defects and deployment
limitations distinct. Performance is informational unless the selected wave
qualifies it. Report `YVEX EXTERNAL FINDINGS` with findings, no new findings,
or the exact reason the lane was not run. Human Golden and continuity canary
remain separate operator verdicts.
