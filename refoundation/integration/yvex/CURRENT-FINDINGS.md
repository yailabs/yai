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

## YVEX-H14-001 — black-box endpoint unavailable

- finding_id: YVEX-H14-001
- observed_at: 2026-09-01 Europe/Rome
- YAI SHA: `0b48edee499f7b74b3a529f728af7912a24d0e5a` (publication-bound probe)
- YVEX SHA: not inspected; H14 is black-box only
- model: not exposed
- endpoint: `http://127.0.0.1:8001/v1`
- layer: deployment/runtime availability
- classification: `DEPLOYMENT_LIMITATION`
- severity: informational / qualification-blocking
- observed: `/v1/models` refused the loopback connection; no endpoint/model configuration was supplied
- expected generic contract: operator-supplied reachable OpenAI-compatible endpoint and provider-exposed model ID
- reproduction: run `yvex-black-box-20260901T110023Z-1655196` from `tests/integration/yvex/qualification_yvex_provider.sh`
- YAI impact: deterministic generic-provider tests pass; live transport and X/Y/Z Case epistemic flows remain unexecuted
- YVEX impact: none; no server was administered or inspected and no defect is inferred
- recommended owning repository: deployment/operator environment
- recommendation: supply `YAI_EXTERNAL_PROVIDER_BASE_URL` and `YAI_EXTERNAL_PROVIDER_MODEL`, then rerun the same black-box harness
- status: open external dependency

H14 introduced no new YVEX-side finding. Normal YAI integration requires no
YVEX source/artifact/profile/engine/session identity or CLI operation.

## H15 — black-box qualification not executed

- observed_at: 2026-09-01T17:08:52+02:00
- YAI SHA: `184a32987958fa49a9098908721eb54410246a8c`
- endpoint: not supplied (`YAI_EXTERNAL_PROVIDER_BASE_URL` absent)
- model: not supplied (`YAI_EXTERNAL_PROVIDER_MODEL` absent)
- state: `blocked_external_dependency`
- classification: no new finding; the external qualification precondition was
  absent
- reproduction: supply both documented environment variables and run
  `make qualification-yvex-provider`

No new YVEX-side finding was established by H15. YVEX source, CLI, profiles,
artifacts, engines, sessions and administration were not inspected or used.

## W16 — provider-attachment qualification not executed

- observed_at: 2026-09-01T21:15:02+02:00
- YAI SHA: `301fd86c720e25a9b28df52435ae59525e644eb4`
- endpoint: not supplied (`YAI_EXTERNAL_PROVIDER_BASE_URL` absent)
- model: not supplied (`YAI_EXTERNAL_PROVIDER_MODEL` absent)
- state: `blocked_external_dependency`
- classification: no new external-provider finding; the black-box qualification
  precondition was absent
- reproduction: supply both documented environment variables and run
  `make qualification-yvex-provider`

No YVEX-side provider finding was established by W16. The separate read-only
study of the YVEX `models1` CLI at
`e6f8ac71ac862945b9dd500fb9d6043e21147064` informed only the YAILabs CLI-family
architecture; it did not inspect or administer a provider deployment and is not
external-provider qualification evidence.
