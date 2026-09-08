# Guided provider catalog discovery

Baseline: `51dd77204f633851b6456b819a085165d20d281f`; clean master, equal local,
origin and remote before mutation. Intended commit:
`feat: discover provider models in guided Case connection`.
This is product setup hardening, not I07. The containing publication SHA belongs
in the final handoff, not this document. Pre-publication qualification is
recorded below; publication requires the isolated commit and remote equality.

## Decision

The endpoint's public catalog supplies exact candidate identities, not semantic
suitability, trust, qualification, residency or loaded-engine truth. One valid
entry needs no model-name question; multiple entries require exact selection by
name or displayed index. Zero entries, malformed/duplicate identities, explicit
pagination or more than 128 models fail closed. Never select the first entry of
an ambiguous catalog or infer meaning from a model/provider name.

Public metadata discovery is authorized by entering the endpoint. It sends no
Case input, runs no inference and registers no target. Existing transport owns
TLS, exact endpoint path, DNS locality and redirect refusal. Metadata GET has
the 30s I/O budget, separate from the buffered inference response budget.

Literal-IP/localhost locality derives from existing address predicates, not
provider identity. Ambiguous DNS names ask for scope; dispatch still verifies
every resolved address. Credentials are requested only on 401/403, as an env
reference, with one bounded retry. No token enters normal input or Case state.

After discovery, one confirmation displays exact target, profile, scope and
Case generation. `approve` explicitly consents to synthetic qualification,
trust, an operator-attested PrimaryConversation suitability record, and the
exact pinned binding. Provenance reference is generated from exact actor/Case/
target/qualification facts; no evaluator is invented. An existing primary
requires `replace` in that same confirmation. Empty input cancels; it never
approves or emits an invalid identifier. No unneeded replacement question.

The controller revalidates the captured Case generation before any target
registration and again after qualification. Qualification re-reads the catalog
and refuses a missing exact model before synthetic inference. Discovery cannot
authorize a later target substitution. Explicit full-form plumbing remains
available with explicit locality/credential/attestation configuration.

No new owner, registry, persistent catalog, database or canonical schema.
Transition v18, CaseState v15, ProviderQualification v5 and LMDB 37/40 unchanged.
REPLAI remains the sole editor; its pin and terminal mechanics are unchanged.
The original operator Case and YVEX implementation are never modified by tests.

## Archaeology

Read `yai-dev` at `5c1c7b9d099eea9f2947146cd821d6501c4a6ddf`,
`src/models/registry/model_alias.c` (history: `cffb318b9`, root-layout epoch),
and `src/runtime/provider/runtime_provider_lifecycle.c`. They used public
`/v1/models` readiness/exact-name checks and bounded curl probes. Recover public
catalog/exact identity pressure in the current provider/controller boundary.
Reject substring matching, shell HTTP fallback, alias/profile registry and
runtime model-loading ownership. No published source found here established a
safe single-versus-multiple guided selector; the current typed mechanism closes
that consumer gap instead of copying historical ownership.

## Evidence contract

Use the existing `../provider-connect-hardening/capture.py` to retain exact
commands, baseline, run/order/pre-state, environment, exits and bounded unedited
stdout/stderr in this package. Product PTY transcripts are raw hex, separately
ordered per disposable home. All commands run from the repository root with
`TMPDIR=/tmp` and, for full terminal qualification, `build/r4-venv/bin` on PATH.

Local discovery scenarios must prove: singleton without redundant questions,
multiple models with exact selection rather than first-entry choice,
authentication challenge, empty/malformed/duplicate catalog, drift before probe,
stale Case approval, cancellation without target/Case mutation, and explicit
replacement. Existing strict-text full-profile refusal, cognitive SEND/result
and replay assertions remain intact. Golden free/Workflow use this same product
wizard. No loopback proof may be labeled real YVEX or human acceptance.

## Qualification results

`local.jsonl`, `gates.jsonl` and `product-local.jsonl` retain the actual commands
and per-home product identifiers. Run orders identify starts, not completion
order: some independent gates ran concurrently. No records from different
temporary Cases are one causal history.

| Proof / provider mode | Command and evidence | Result |
| --- | --- | --- |
| Product + setup / loopback + no provider | `make smoke-provider-connection smoke-guided-case-setup`, discovery-local/1 | exit 0, 9.33s |
| Unit/component / no provider | `make test-fast`, discovery-gates/6, actual host | exit 0, 6.38s |
| Static / no provider | `cargo clippy --manifest-path cmd/yai/Cargo.toml --all-targets -- -D warnings`, discovery-gates/7 | exit 0, 2.31s |
| Golden product/recovery / loopback | `make test-golden-local`, discovery-gates/4 | exit 0, 35.17s |
| Complete local publication union + characterization / no provider + loopback | `make test-release characterization`, discovery-gates/8 | exit 0, 281.14s |
| Documentation/layout/topology / no provider | `make check-docs check-layout check-validation-topology`, discovery-gates/9 | exit 0, 1.23s |
| Formatting / no provider | `cargo fmt --manifest-path cmd/yai/Cargo.toml -- --check`, discovery-gates/10 | exit 0, 0.36s |

Golden free `yai-golden-free-rz3lewgy` and Workflow
`yai-golden-free-met1wxrl` use fresh homes, the real executable, REPLAI PTYs,
canonical stores and real local resource/provider peers. Reviewed effects,
restart/retry, Workflow, Handoff and derived-index rebuild assertions remain
reachable; no remote intelligence or human acceptance is claimed.

Coverage is preserved: guided cancellation after catalog discovery moved from
the no-provider setup test to the loopback product matrix. The no-provider test
now proves invalid credential-bearing endpoints refuse before network. No
runtime semantics were weakened to preserve a classification. The topology
audit identifies 472 entries, 371 Rust tests, 85 Make leaves, all 71 previous
smoke leaves retained, and zero duplicate Make leaves in the composite union.

Failed development attempts are retained, not rewritten as passes:
discovery-gates/1 was a misplaced test namespace, /2 was PTY synchronization on
an obsolete prompt, /3 was host executable-identity confinement under the agent
sandbox (the same fast gate passed on the actual host at /6). Publication attempt
/5 exposed a duplicate `scenario` keyword in test evidence recording; this was
fixed without changing product behavior or relaxing assertions. Final publication
union and documentation checks passed separately after these corrections.
`product-local.jsonl` retains all ten final per-scenario PASS records as well as
the partial earlier run; failed attempts are not conflated with the final union.

## YVEX external findings

`external.jsonl` and `product-external.jsonl` record independent run
`yai-connect-product-bwor7atp`, YAI baseline above plus this working delta,
endpoint `http://127.0.0.1:18001`, exact exposed model
`deepseek-v4-flash-mixed-mxfp4-release-v1`. Public metadata reports
`yvex.openai.compat.v2`; no YVEX commit or private runtime truth was inspected.

Discovery GET (discovery-external/1) exited 0 in 0.016s. The actual external PTY
(discovery-external/2) auto-selected that singleton, accepted one combined
confirmation, performed real text qualification and published its pinned Case
binding: **setup NO_ISSUE**, not merely source inspection or a loopback fixture.
Target `provider-target:49ae1c4734762311996ec3631d9c3947` is recorded in the
unaltered product transcript.

The subsequent canonical SEND failed: whole external test exit 1 in 15.31s,
HTTP 413 `request_too_large`, request bytes 6153, existing
`delivery_indeterminate` posture, no blind retry. Turn
`conversation-turn:sha256:f679883d7f4097cea6160eea7754f27b9655079ddeaf1f2ba4ff9dcb6ea9420b`
was committed before refusal. This reproduces the previous public input-capacity
pressure, classified **YVEX_CANDIDATE** pending provider-side investigation;
discovery does not resolve it and YAI does not remove governed context to hide it.
No claim of full real dialogue or external Golden PASS is authorized.

## Remaining boundary / cumulative acceptance

The cumulative [ZERO-TO-CURRENT runbook](../../../docs/zero-to-current.md) is
updated in place, preserving the whole product lifecycle. Ordinary singleton
loopback connection needs only endpoint plus explicit `approve` (or `replace`).
Credential references and ambiguous DNS scope remain conditional. Catalog size
and exact identity constraints are bounded; generic paginated-provider onboarding
is not implemented. Providers with public catalogs but separately authenticated
inference can still use the explicit full-form credential configuration.

`GOLDEN_CASE_LOCAL = PASS`; `GOLDEN_CASE_EXTERNAL_YVEX = NOT_RUN` for the full
lifecycle, with real text SEND blocked as above; `CONTINUITY_CANARY = NOT_RUN`;
`HUMAN_GOLDEN_CASE = PENDING_OPERATOR`. No private canary/operator home was reset.
Schema delta 0; semantic/operational owners +0; LMDB databases +0 (37/40).
I07 remains unselected. Runtime loading, capacity repair and model selection by
private engine state remain outside YAI onboarding.
