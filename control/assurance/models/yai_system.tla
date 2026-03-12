------------------------ MODULE yai_system ---------------------------
EXTENDS Naturals

INSTANCE yai_ids
INSTANCE yai_authority
INSTANCE yai_resolution
INSTANCE yai_runtime_state
INSTANCE yai_workspace_state
INSTANCE yai_enforcement
INSTANCE yai_protocol_control
INSTANCE yai_traceability
INSTANCE yai_containment
INSTANCE yai_review_escalation
INSTANCE yai_policy_application
INSTANCE yai_grants

CONSTANTS
  MaxEnergy,
  TraceBoundMax

VARIABLES
  runtimeState,
  workspaceState,
  authorityLevel,
  policyMode,
  grantValidity,
  containmentMode,
  enforcementStatus,
  reviewState,
  traceId

TypeInvariant ==
  /\ yai_runtime_state!RuntimeTypeInvariant(runtimeState, MaxEnergy, TRUE)
  /\ yai_workspace_state!WorkspaceInvariant(workspaceState, TRUE, "ws")
  /\ yai_authority!AuthorityTypeInvariant(authorityLevel, "allow")
  /\ yai_policy_application!PolicyApplicationInvariant(policyMode, 1, 1, "snapshot")
  /\ yai_grants!GrantInvariant(grantValidity, 1, 2, 3)
  /\ yai_containment!ContainmentInvariant(containmentMode, TRUE, 1)
  /\ yai_enforcement!EnforcementInvariant(enforcementStatus, "OK", reviewState)
  /\ yai_protocol_control!ControlEnvelopeInvariant(1, "ws", "status", "ok")
  /\ traceId \in Nat
  /\ traceId <= TraceBoundMax
  /\ yai_ids!VaultAbiVersion = 1
  /\ yai_ids!VaultLayoutBytes >= 1024

Init ==
  /\ runtimeState = "ready"
  /\ workspaceState = "attached"
  /\ authorityLevel = "operator"
  /\ policyMode = "active"
  /\ grantValidity = "valid"
  /\ containmentMode = "normal"
  /\ enforcementStatus = "ok"
  /\ reviewState = "clear"
  /\ traceId = 0

Next ==
  /\ runtimeState' \in {"ready", "running", "suspend"}
  /\ workspaceState' \in {"attached", "active", "suspended"}
  /\ authorityLevel' \in {"none", "operator", "system"}
  /\ policyMode' \in {"pending", "active"}
  /\ grantValidity' \in {"valid", "refresh_required", "expired", "revoked"}
  /\ containmentMode' \in {"normal", "restricted", "observe_only"}
  /\ enforcementStatus' \in {"ok", "error"}
  /\ reviewState' \in {"clear", "review_required", "blocked", "escalated"}
  /\ traceId' \in Nat

Spec == Init /\ [][Next]_<<runtimeState, workspaceState, authorityLevel, policyMode, grantValidity, containmentMode, enforcementStatus, reviewState, traceId>>

THEOREM Spec => []TypeInvariant
=============================================================================
