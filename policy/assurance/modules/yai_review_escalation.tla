-------------------- MODULE yai_review_escalation --------------------
EXTENDS Naturals

ReviewState == {"clear", "review_required", "blocked", "escalated"}

EscalationInvariant(prev, next, severity) ==
  /\ prev \in ReviewState
  /\ next \in ReviewState
  /\ severity \in Nat
  /\ severity > 0 /\ prev = "review_required" => next \in {"review_required", "escalated", "blocked"}

=============================================================================
