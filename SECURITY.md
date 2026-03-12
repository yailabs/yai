# Security Policy

## Scope

This policy applies to the **`yai` runtime implementation repository** (Boot/Root/Kernel/Engine/Mind workspace integration, tooling, and release artifacts).

Normative contract behavior (control/protocol/control/graph/vault/compliance) is defined in **`governance` (canonical) and `governance/runtime-package` (runtime contract surface)**. Contract issues may be triaged here but the source-of-truth fix may live in `governance`.

## Reporting a Vulnerability

Preferred: **GitHub Security Advisory** (private report) for this repository.

Fallback: open a GitHub issue labeled **`security`** and include only non-sensitive details (impact + affected paths + high-level reproduction). If the issue requires sensitive material, do **not** post it publicly—use private channels.

Minimum report content:
- Impact and expected vs actual behavior
- Affected components/paths (e.g., `root/`, `kernel/`, `engine/`, `runtime/`, `mind/`, `tools/`)
- Reproduction steps that do **not** require secrets
- Version/commit information (tag or SHA)
- Any mitigations you already tried

Do **not** include:
- credentials, tokens, private keys, session material
- private datasets or personal data
- internal infrastructure URLs or sensitive logs

## Response and Triage

We aim to:
- acknowledge receipt and start triage as soon as practical
- request additional information if needed
- provide a mitigation or fix plan when the issue is confirmed

Fixes may be shipped as:
- a commit on `main`
- a tagged release with notes and checksums (see `VERSIONING.md` and `COMPATIBILITY.md`)

## Supported Versions

- Active development line: **`main`**
- Release support windows and compatibility guarantees are defined in:
  - `VERSIONING.md`
  - `COMPATIBILITY.md`

## Exposure Model

`yai` is **local-first** and is **not internet-exposed by default**.

Primary runtime surfaces are **local process boundaries** and **workspace-scoped Unix Domain Sockets (UDS)**. Any network exposure is an operator choice and must be treated as a separate hardening task.

## Threat Model Summary

Primary security boundaries and high-risk surfaces:

- **Root plane authority & supervision** (`root/`)
- **Kernel enforcement & workspace isolation** (`kernel/`)
- **Envelope validation / protocol dispatch** (`runtime/` and protocol bindings)
- **Engine external-effect boundary and provider gates** (`engine/`)
- **Mind orchestration** (`mind/`) as a proposer that must not bypass authority

Key risks include:
- authority bypass or confused-deputy flows
- workspace escape via filesystem/run directories/sockets
- protocol/envelope downgrade or version confusion
- unsafe provider attachment/trust transitions
- sensitive data leakage via logs/artifacts

## Hardening Baselines

Operators and contributors should ensure:

- Contract pins are verified before upgrades (`tools/release/check_pins.sh`)
- Request envelopes are validated before dispatch (version/role/arming/authority)
- Privileged operations hard-fail without explicit authority
- Workspace runtime files (sockets/locks/state) are isolated per workspace
- Critical state transitions and denials emit auditable events/logs
- Secrets, runtime logs, generated state, and local datasets are not committed

## License

This security policy is distributed under Apache-2.0. See `LICENSE` and `NOTICE`.