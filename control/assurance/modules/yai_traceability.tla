---------------------- MODULE yai_traceability -----------------------
EXTENDS Naturals, Sequences

TraceClasses == {
  "authority.command_gate",
  "resolution.precedence",
  "policy.application",
  "grants.validity",
  "containment.mode",
  "protocol.control_envelope",
  "review.escalation"
}

TraceabilityInvariant(className, runtimeSurface) ==
  /\ className \in TraceClasses
  /\ runtimeSurface \in STRING
  /\ runtimeSurface # ""

=============================================================================
