------------------------ MODULE yai_resolution -----------------------
EXTENDS Naturals, Sequences

ResolutionEffects == {"allow", "hold", "deny", "review"}

PrecedenceOrder == <<"deny", "hold", "review", "allow">>

HigherOrEqual(a, b) ==
  LET ia == SelectSeq(PrecedenceOrder, LAMBDA x: x = a)
      ib == SelectSeq(PrecedenceOrder, LAMBDA x: x = b)
  IN Len(ia) > 0 /\ Len(ib) > 0 /\ Head(ia) <= Head(ib)

ResolutionInvariant(effect, authorityDecision) ==
  /\ effect \in ResolutionEffects
  /\ authorityDecision \in {"allow", "review_required", "deny"}
  /\ authorityDecision = "deny" => effect = "deny"

=============================================================================
