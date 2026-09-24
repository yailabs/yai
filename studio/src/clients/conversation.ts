/** Durable SEND identity belongs to YAI. The local envelope only permits an exact
 * lost-acknowledgement retry; observing a request never dispatches it again. */
export interface ConversationSendInput {
  case_ref: string; participant_ref: string; thread_ref: string;
  submission_ref: string; expected_generation: number;
  parts: Array<{ modality: "text"; media_type: "text/plain"; bytes: number[] }>;
  memory_search_mode?: "standard" | "fast";
}
export interface ConversationExecution {
  case_ref: string; participant_ref: string; submission_ref: string;
  turn_ref: string; request_ref: string; observed_generation: number;
  posture: "admitted" | "running" | "completed" | "provider_result_recorded" | "refused" | "failed" | "cancelled" | "delivery_indeterminate" | "unresolved";
  invocation_refs: string[];
  primary_result?: { result_id: string; invocation_id: string; output: string; selection: { selected_target_id: string } } | null;
  attempt_outcomes: Array<{ status?: string; failure_class?: string; [key: string]: unknown }>;
}
export interface ConversationSubmission {
  created: boolean; execution: ConversationExecution;
  memory_search?: {
    requested: "standard" | "fast"; effective: "standard" | "fast";
    system_model_active: boolean; posture: string;
  } | null;
}
export const conversationStorageKey = (caseRef: string, participant: string) => `yai.studio.conversation.v1:${caseRef}:${participant}`;
export function makeConversationSend(caseRef: string, participant: string, thread: string | undefined, generation: number, text: string, memorySearchMode: "standard" | "fast" = "standard"): ConversationSendInput {
  if (!text.trim()) throw new Error("Write a message first.");
  const bytes = [...new TextEncoder().encode(text)];
  if (bytes.length > 65536) throw new Error("Messages are limited to 64 KiB of UTF-8 text.");
  return { case_ref: caseRef, participant_ref: participant, thread_ref: thread ?? `thread:studio:${crypto.randomUUID()}`, submission_ref: `studio-send:${crypto.randomUUID()}`, expected_generation: generation, parts: [{ modality: "text", media_type: "text/plain", bytes }], ...(memorySearchMode === "fast" ? { memory_search_mode: "fast" as const } : {}) };
}

export function conversationExecutionMessage(execution: ConversationExecution): string {
  if (execution.primary_result) return execution.primary_result.output;
  if (["admitted", "running"].includes(execution.posture)) return "Generating through YAI…";
  if (execution.attempt_outcomes.some(outcome => outcome.failure_class === "request_capacity_refused")) {
    return "YAI stopped this request before inference because its complete context could not be qualified against the target capacity. Inspect model context for the retained observation. Nothing was truncated or sent again.";
  }
  if (execution.attempt_outcomes.some(outcome => outcome.response_status === 413)) {
    return "The model server rejected this request as too large (HTTP 413). The request includes Case context, even for a short message. Check the model deployment’s input limits in Compute. It has not been sent again.";
  }
  if (execution.posture === "unresolved") return "YAI has no recorded response or active execution for this request. Checking its status does not send the request again.";
  return "No completed response is available. This execution has not been sent again.";
}
