# Integration Tests

Integration tests are optional and environment-dependent.
Use this directory for end-to-end checks that require running runtime services.

Canonical operator-surface guardrails:
- `help_guardrail.sh`
- `output_contract_v1_guardrail.sh`
- `operator_capability_pack_guardrail.sh`
- `container_output_guardrail.sh`
- `truth_surface_coherence_wsv83_guardrail.sh`

These scripts validate the active CLI operator truth path:
- command/help taxonomy,
- startup/runtime/status/inspect output semantics,
- container-first binding semantics,
- capability-family visibility and readiness interpretation.
