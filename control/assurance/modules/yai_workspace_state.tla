--------------------- MODULE yai_workspace_state ---------------------
EXTENDS Naturals

WorkspaceStates == {"created", "active", "attached", "suspended", "destroyed", "error"}

WorkspaceInvariant(state, runtimeAttached, wsId) ==
  /\ state \in WorkspaceStates
  /\ runtimeAttached \in BOOLEAN
  /\ wsId \in STRING
  /\ state = "attached" => runtimeAttached

=============================================================================
