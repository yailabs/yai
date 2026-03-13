------------------------ MODULE yai_authority ------------------------
EXTENDS Naturals, Sequences

AuthorityLevels == {"none", "operator", "system"}
AuthorityDecision == {"allow", "review_required", "deny"}

AuthorityTypeInvariant(level, decision) ==
  /\ level \in AuthorityLevels
  /\ decision \in AuthorityDecision

AuthorityGate(level, commandClass, armed) ==
  IF commandClass = "provider" /\ armed = FALSE THEN "deny"
  ELSE IF level = "none" THEN "review_required"
  ELSE "allow"

=============================================================================
