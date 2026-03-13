# Enforcement Invariant Matrix

| Formal Invariant Class | Runtime Enforcement Surface | Expected Outcome Class |
|---|---|---|
| authority.command_gate | `yai_authority_command_gate` | allow / deny / review_required |
| authority.policy_gate | `yai_authority_policy_gate` | allow / deny / review_required |
| resolution.precedence | governance resolution output | deterministic final effect |
| policy.application | runtime policy state apply | active snapshot or pending |
| grants.validity | runtime grants refresh | valid / refresh_required / expired / revoked |
| containment.mode | runtime containment evaluate | normal / restricted / observe_only |
| protocol.control_envelope | `yai_validate_envelope_v1` | accept or typed validation error |
| review.escalation | enforcement decision review state | clear / blocked / review_required |

Notes:
- Matrix is informational counterpart to machine-readable linkage in
  `control/assurance/traceability/enforcement-linkage.json`.
