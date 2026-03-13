------------------- MODULE yai_policy_application --------------------
EXTENDS Naturals

PolicyMode == {"pending", "active"}

PolicyApplicationInvariant(mode, generation, effectCount, snapshotId) ==
  /\ mode \in PolicyMode
  /\ generation \in Nat
  /\ effectCount \in Nat
  /\ snapshotId \in STRING
  /\ mode = "active" => snapshotId # ""

=============================================================================
