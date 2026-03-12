----------------------- MODULE yai_containment -----------------------
EXTENDS Naturals

ContainmentModes == {"normal", "restricted", "observe_only"}

ContainmentInvariant(mode, policyActive, validGrantCount) ==
  /\ mode \in ContainmentModes
  /\ policyActive \in BOOLEAN
  /\ validGrantCount \in Nat
  /\ mode = "normal" => policyActive /\ validGrantCount > 0
  /\ mode = "observe_only" => ~policyActive \/ validGrantCount = 0

=============================================================================
