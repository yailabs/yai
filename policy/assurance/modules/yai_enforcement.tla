----------------------- MODULE yai_enforcement -----------------------
EXTENDS Naturals

EnforcementStatus == {"ok", "error"}
EnforcementCode == {"OK", "AUTHORITY_BLOCK", "POLICY_BLOCK", "WORKSPACE_BIND_MISMATCH"}
ReviewStates == {"clear", "blocked", "review_required"}

EnforcementInvariant(status, code, reviewState) ==
  /\ status \in EnforcementStatus
  /\ code \in EnforcementCode
  /\ reviewState \in ReviewStates
  /\ status = "error" => code # "OK"

=============================================================================
