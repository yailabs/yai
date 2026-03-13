---------------------- MODULE yai_runtime_state ----------------------
EXTENDS Naturals

RuntimeStates == {"halt", "preboot", "ready", "running", "suspend", "error"}

RuntimeTypeInvariant(state, energy, initialized) ==
  /\ state \in RuntimeStates
  /\ energy \in Nat
  /\ initialized \in BOOLEAN

RuntimeSafety(state, initialized) ==
  state = "running" => initialized

=============================================================================
