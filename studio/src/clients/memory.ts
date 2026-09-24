/** Published semantic operations. These are disposable qualified results,
 * never Case state, authority or a frontend memory store. */
export interface RecallBounds { candidates: number; events: number; relations: number; segments: number; expansion_depth: number; semantic_units: number; bytes: number }
export const defaultRecallBounds: RecallBounds = { candidates: 16, events: 64, relations: 128, segments: 16, expansion_depth: 4, semantic_units: 16384, bytes: 1048576 };
export interface RecallRequest {
  schema: "yai.recall_request.v2"; case_id: string; expected_generation: number; participant_id: string;
  at: { kind: "generation"; value: number }; query: string; required_refs: string[]; bounds: RecallBounds;
}
export interface RecalledEvidence {
  events: Array<{ event: { transition_id: string; recorded_generation: number; kind: string }; label: string; validity_at_cut: string; reasons: string[] }>;
  documentary?: { units: Array<{ unit: { id: string; source: string; text: string; kind: string; posture: string }; reasons: string[] }>; sources: Array<{ source: { id: string; source_id: string; revision_id: string; path: string }; applicable_at_cut: boolean; reasons: string[] }> } | null;
  source_closure: Array<{ source_ref: string; posture: string }>; closure_complete: boolean;
}
export interface RecallResult { trace: RecalledEvidence & { trace_id: string; request: RecallRequest; generation: number; limitations: string[]; omitted_candidates: number; expansion_stopped_at_depth: boolean } }
export interface WorkingStateRequest {
  case_id: string; expected_generation: number;
  compilation: {
    scope: { participant_id: string; purpose: "inspection"; consumer: "model"; view_kind: "model_context"; max_items: number; max_provider_claims: number; max_interaction_turns: number };
    intent: string; output_contract_id: string; max_semantic_units: number; max_derived_items: number;
    resource_refs: string[]; required_refs: string[]; previous_item_ids: string[]; view_selection_id: null;
  };
  at: { kind: "generation"; value: number } | null; recall_required_refs: string[]; recall_bounds: RecallBounds; max_output_bytes: number;
}
export interface SemanticReference { reference_id: string; family: string; members: string[]; sources: Array<{ source_id: string; revision_id: string; path: string }>; mandatory_task_dependency: boolean }
export interface WorkingEntry { entry_id: string; posture: string; value: { kind: string; value: Record<string, unknown> & { evidence?: RecalledEvidence; references?: SemanticReference[]; resident_references?: string[] } }; provenance: Array<{ kind: string; source_ref: string }> }
export interface WorkingState {
  [key: string]: unknown;
  working_state_id: string; case_id: string; case_generation: number; participant_id: string; request: WorkingStateRequest["compilation"];
  entries: WorkingEntry[]; bounds: { selected_items: number; selected_semantic_units: number; omitted_items: number; omitted_by_locality: number; omitted_by_budget: number };
  recall?: { recall_closure_complete: boolean; limitations: string[]; omitted_recall_candidates: number };
}
export interface WorkingStateResult { working_state: WorkingState; posture?: string; compilation_mode?: string }
export interface DecisionFrontier {
  [key: string]: unknown;
  frontier_id: string; request: { max_candidates: number }; candidates: Array<{ candidate: { candidate_id: string; candidate_kind: string; description: string; semantic_refs: string[] }; requirement: string; origins: unknown[] }>;
  visible_candidate_count: number; required_candidate_count: number; omitted_optional_candidates: number;
}
export interface FrontierResult { frontier: DecisionFrontier; origin_count: number }
export interface DecisionPreparation { request_id: string; decision_kind: string; case_generation: number; candidates: DecisionFrontier["candidates"][number]["candidate"][] }
export interface DecisionPrepareInput { working_state: WorkingState; frontier: DecisionFrontier; decision_kind: string; budget: { max_candidates: number; max_result_bytes: number; max_compute_millis: number } }
export interface WorkingRefreshRequest { schema: "yai.working_refresh_request.v1"; case_id: string; participant_id: string; base_working_state_id: string; budget: null }
export interface PageRequest { schema: "yai.semantic_page_request.v1"; case_id: string; participant_id: string; base_working_state_id: string; references: string[]; action: "page_in" | "page_out"; bounds: RecallBounds }

export interface FastSearchPreparation {
  schema: "yai.fast_search_prepare_result.v1";
  availability: "unavailable_producer" | "unavailable_no_choices";
  active: boolean; actual_path: string;
  navigation: {
    navigation_id: string; working_state_id: string;
    posture: "ready_for_optional_producer" | "deterministic_fallback_no_choice";
    omitted_optional_choices: number;
    choices: Array<{
      candidate: DecisionFrontier["candidates"][number]["candidate"];
      origin: { kind: "resident_recall_group"; entry_id: string }
        | { kind: "deferred_working_group"; reference_id: string; group_entry_id: string }
        | { kind: "deterministic_path" };
    }>;
  };
}
