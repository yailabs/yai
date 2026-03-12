------------------------- MODULE yai_grants --------------------------
EXTENDS Naturals

GrantValidity == {"valid", "refresh_required", "expired", "revoked"}

GrantInvariant(validity, issuedAt, refreshAfter, expiresAt) ==
  /\ validity \in GrantValidity
  /\ issuedAt \in Nat
  /\ refreshAfter \in Nat
  /\ expiresAt \in Nat
  /\ issuedAt <= refreshAfter
  /\ refreshAfter <= expiresAt

=============================================================================
