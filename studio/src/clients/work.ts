/** Bounded authored workflow form: human checkpoints; YAI owns validation/progression. */
export interface HumanCheckpoint { node_id: string; kind: "human_input"; actor_slot: string; prompt: string; required_roles: string[]; input_kind: "text"; max_bytes: number }
export interface WorkflowEdgeInput { from: string; to: string; kind: "always" }
export interface WorkflowDefinitionInput { schema: "yai.workflow_definition.v1"; tenant_id: string; workflow_key: string; declared_version: string; name: string; description: string; nodes: HumanCheckpoint[]; edges: WorkflowEdgeInput[] }
export interface WorkflowDefinition extends WorkflowDefinitionInput { workflow_definition_id: string; content_digest: string }
export interface WorkflowBindInput { case_ref: string; definition_ref: string; executor_bindings: Array<{ slot: string; participant_id: string }>; resource_bindings: Array<{ slot: string; attachment_id: string }>; case_bindings: Array<{ slot: string; case_id: string }> }
export interface WorkflowPatchInput { schema: "yai.workflow_plan_patch.v1"; base_effective_topology_digest: string; operations: Array<{ operation: "add_node"; node: HumanCheckpoint } | { operation: "add_edge"; edge: WorkflowEdgeInput } | { operation: "disable_node"; node_id: string }> }
export interface WorkCommit { state: { case_id: string; generation: number }; transition: { transition_id: string; payload: { kind: string; data: { patch?: { patch_id: string; base_effective_topology_digest: string }; offer?: { handoff_id: string; source_case_id: string; target_case_id: string }; acceptance?: { handoff_id: string }; decline?: { handoff_id: string }; result?: { handoff_id: string }; reconciliation?: { handoff_id: string } } } } }
export interface HandoffData { kind: "text"; value: string }
export interface HandoffOfferInput { source_case_ref: string; target_case_ref: string; request: HandoffData; required_target_roles: string[] }
export interface HandoffAcceptInput { target_case_ref: string; source_case_ref: string; handoff_ref: string; participant_ref: string }
export interface HandoffDeclineInput extends HandoffAcceptInput { reason: string }
export interface HandoffResultInput { target_case_ref: string; handoff_ref: string; participant_ref: string; outcome: "succeeded" | "failed" | "cancelled"; result: HandoffData; evidence_refs: string[] }
