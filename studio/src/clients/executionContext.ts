import type { ConversationExecution } from "./conversation";
import type { WorkingEntry, WorkingState } from "./memory";

export interface PreparedFrame {
  frame_id: string; task: string; entries: WorkingEntry[];
  semantic_instructions: string[]; model_independent_constraints: string[];
}

export interface InputObservation {
  invocation_id: string; case_id: string; participant_id: string; case_generation: number;
  rendered_input_id: string; target_id: string; model_id: string;
  serialized_request_digest: string; serialized_request_bytes: number; observed_at_unix_ms: number;
  refusal: string | null;
  capacity: null | {
    public_contract: string; observation_kind: string; model_id: string;
    input_tokens: number | null; input_capacity_tokens: number; sequence_capacity_tokens: number;
    requested_output_tokens: number | null; effective_output_tokens: number | null;
    token_capacity_compatible: boolean | null; full_requested_output_fits: boolean | null;
    http_body_limit_bytes: number; resource_reservation: boolean;
  };
}
export interface PreparedContext {
  observed_generation: number; total_invocations: number; omitted_invocations: number;
  invocations: Array<{
    invocation_ref: string; lineage: { case_generation: number; rendered_input_id: string };
    working_state: WorkingState | null; projection: unknown; frame: PreparedFrame | null;
    input_observation: InputObservation | null; unavailable_reason: string | null;
  }>;
}
export type InspectedConversationExecution = ConversationExecution & { prepared_context?: PreparedContext | null };
