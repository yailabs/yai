# Mesh Sovereign Authority Law Alignment (MF-3)

## Purpose

Capture MF-3 authority semantics that must be consumable by `yai-law` without
flattening mesh participation into sovereign authority.

## MF-3 signals relevant to law

- peer legitimacy state is explicit (`candidate|enrolled|trusted|suspended|revoked` baseline)
- enrollment/trust finalization is owner-anchored
- authority scope state is explicit and revocable
- suspension/revocation are first-class boundary markers
- participation/awareness state is explicitly non-sovereign

## Law-facing implications

Future law slices can govern:

- legitimacy transition policies
- trust bootstrap prerequisites
- authority scope constraints by role/workspace/profile
- suspension/revocation triggers and effects
- provenance/trust binding requirements for sovereign acceptance

## Explicit boundary

MF-3 does not move sovereign adjudication into mesh coordination.
It introduces structured authority-plane inputs so law can govern legitimacy and
scope boundaries without ambiguity.

## References

- `docs/architecture/sovereign-mesh-authority-foundation-model.md`
- `docs/architecture/workspace-authority-and-truth-plane-model.md`
