# Current YAI ↔ YVEX external findings

YVEX is an external read-only system for YAI development. Findings here never
authorize a YVEX patch from this repository.

## YVEX-W14-001 — live qualification unavailable at discovered checkout

- finding_id: YVEX-W14-001
- observed_at: 2026-08-31 Europe/Rome
- YAI SHA: `f430624f547d65d090e94a18f92960a651ac5b5e`
- YVEX SHA: `2df3b84cc840dfca8b38f6fc387a833169b5598e`
- model: none installed (`MODELS count=0`)
- endpoint: `http://127.0.0.1:8001/v1`
- layer: deployment/runtime availability
- classification: `DEPLOYMENT_LIMITATION`
- severity: informational / qualification-blocking
- observed: checkout clean on `main`; executable reports protocol 11; no private runtime socket; no model; loopback endpoint refuses connections
- expected generic contract: reachable `/v1/models` followed by non-stream `POST /v1/chat/completions`
- reproduction: `YVEX_REPO=/tmp/yvex-research.bZQXAZ/repo tests/integration/yvex/qualification_yvex_provider.sh`
- YAI impact: deterministic generic-provider and Wave 14 core qualification remain valid; live external compatibility cannot be claimed
- YVEX impact: none; no defect inferred from an intentionally unstarted/unprovisioned server
- recommended owning repository: deployment/operator environment
- recommendation: install/select a model, start the existing YVEX OpenAI profile on loopback, then rerun the permanent harness
- status: open external dependency

No YVEX-side defect was established at this checkpoint.
