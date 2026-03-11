----------------------- MODULE yai_runtime_legacy -----------------------
\* Historical artifact name retained for continuity.
\* Primary semantic ontology: runtime-core sovereignty + orchestration/agents governed participation.
EXTENDS Naturals, yai_ids_legacy

CONSTANTS 
    MaxEnergy,
    ActionCost,
    TraceBoundMax

States == {"HALT", "PREBOOT", "READY", "HANDOFF_COMPLETE", "RUNNING", "SUSPEND", "ERROR"}
Authority == {"NONE", "CORE", "EXEC"}

VARIABLES 
    state,
    authority,
    cognitive_map,
    energy,
    trace_id,
    external_effect,
    compliance_context_valid

TypeInvariant ==
    /\ state \in States
    /\ authority \in Authority
    /\ cognitive_map \in BOOLEAN
    /\ energy \in Nat
    /\ trace_id \in Nat
    /\ external_effect \in BOOLEAN
    /\ compliance_context_valid \in BOOLEAN

EnergySafe ==
    energy >= 0

CognitiveIntegrity ==
    (cognitive_map = FALSE) => (state /= "RUNNING")

AuthorityRequired ==
    (state = "RUNNING") => (authority # "NONE")

ExternalEffectGuard ==
    external_effect =>
        /\ authority # "NONE"
        /\ compliance_context_valid = TRUE

TraceBound ==
    trace_id <= TraceBoundMax

VaultAbiVersionOk ==
    VaultAbiVersion = 1

Init ==
    /\ state = "HALT"
    /\ authority = "NONE"
    /\ cognitive_map = TRUE
    /\ energy = MaxEnergy
    /\ trace_id = 0
    /\ external_effect = FALSE
    /\ compliance_context_valid = TRUE

Strap_Preboot ==
    /\ state = "HALT"
    /\ state' = "PREBOOT"
    /\ authority' = "CORE"
    /\ energy' = MaxEnergy
    /\ external_effect' = FALSE
    /\ UNCHANGED <<cognitive_map, trace_id, compliance_context_valid>>

Preboot_Ready ==
    /\ state = "PREBOOT"
    /\ state' = "READY"
    /\ external_effect' = FALSE
    /\ UNCHANGED <<authority, cognitive_map, energy, trace_id, compliance_context_valid>>

Handoff_Complete ==
    /\ state = "READY"
    /\ state' = "HANDOFF_COMPLETE"
    /\ authority' = "EXEC"
    /\ external_effect' = FALSE
    /\ UNCHANGED <<cognitive_map, energy, trace_id, compliance_context_valid>>

Handoff_Run ==
    /\ state = "HANDOFF_COMPLETE"
    /\ state' = "RUNNING"
    /\ external_effect' = FALSE
    /\ UNCHANGED <<authority, cognitive_map, energy, trace_id, compliance_context_valid>>

Engine_Execute ==
    /\ state \in {"READY", "RUNNING"}
    /\ authority # "NONE"
    /\ cognitive_map = TRUE
    /\ energy >= ActionCost
    /\ state' = "RUNNING"
    /\ energy' = energy - ActionCost
    /\ trace_id' = trace_id + 1
    /\ external_effect' = FALSE
    /\ UNCHANGED <<authority, cognitive_map, compliance_context_valid>>

Engine_ExternalEffect ==
    /\ state = "RUNNING"
    /\ authority # "NONE"
    /\ compliance_context_valid = TRUE
    /\ external_effect' = TRUE
    /\ trace_id' = trace_id + 1
    /\ compliance_context_valid' = compliance_context_valid
    /\ UNCHANGED <<state, authority, cognitive_map, energy>>

Critical_Invalidation ==
    /\ state = "RUNNING"
    /\ cognitive_map' = FALSE
    /\ state' = "SUSPEND"
    /\ external_effect' = FALSE
    /\ UNCHANGED <<authority, energy, trace_id, compliance_context_valid>>

Suspend_Resume ==
    /\ state = "SUSPEND"
    /\ cognitive_map = TRUE
    /\ state' = "RUNNING"
    /\ external_effect' = FALSE
    /\ UNCHANGED <<authority, cognitive_map, energy, trace_id, compliance_context_valid>>

System_Reset ==
    /\ state = "SUSPEND"
    /\ state' = "HALT"
    /\ authority' = "NONE"
    /\ cognitive_map' = TRUE
    /\ external_effect' = FALSE
    /\ UNCHANGED <<energy, trace_id, compliance_context_valid>>

Engine_Error ==
    /\ state = "RUNNING"
    /\ state' = "ERROR"
    /\ external_effect' = FALSE
    /\ UNCHANGED <<authority, cognitive_map, energy, trace_id, compliance_context_valid>>

Engine_Halt ==
    /\ state = "RUNNING"
    /\ state' = "HALT"
    /\ external_effect' = FALSE
    /\ UNCHANGED <<authority, cognitive_map, energy, trace_id, compliance_context_valid>>

Error_Reset ==
    /\ state = "ERROR"
    /\ state' = "HALT"
    /\ external_effect' = FALSE
    /\ UNCHANGED <<authority, cognitive_map, energy, trace_id, compliance_context_valid>>

Reconfigure ==
    /\ state = "SUSPEND"
    /\ cognitive_map = FALSE
    /\ cognitive_map' = TRUE
    /\ external_effect' = FALSE
    /\ UNCHANGED <<authority, state, energy, trace_id, compliance_context_valid>>

Next ==
    Strap_Preboot
    \/ Preboot_Ready
    \/ Handoff_Complete
    \/ Handoff_Run
    \/ Engine_Execute
    \/ Engine_ExternalEffect
    \/ Critical_Invalidation
    \/ Reconfigure
    \/ Suspend_Resume
    \/ System_Reset
    \/ Engine_Error
    \/ Engine_Halt
    \/ Error_Reset

Spec ==
    Init /\ [][Next]_<<state, authority, cognitive_map, energy, trace_id, external_effect, compliance_context_valid>>

THEOREM Spec =>
    []TypeInvariant
    /\ []EnergySafe
    /\ []CognitiveIntegrity
    /\ []AuthorityRequired
    /\ []ExternalEffectGuard
    /\ []VaultAbiVersionOk
    /\ [](external_effect => compliance_context_valid)
=============================================================================
