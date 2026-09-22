/** Published local Application execution contracts. References survive transport;
 * observation is read-only and never authorizes a second dispatch. */
export type ExecutionReference = { domain: "runtime_work" | "resource_request"; submission_ref: string }
  | { domain: "source_acquisition"; source_ref: string; attempt: number };
export interface ExecutionGetInput { case_ref: string; participant_ref: string; execution: ExecutionReference }
export interface SourceAcquireInput { case_ref: string; participant_ref: string; source_ref: string; attempt: number; expected_generation: number }
export interface SourceResumeInput extends SourceAcquireInput { previous_progress_ref: string }
export interface ExecutionObservation {
  case_ref: string; participant_ref: string; schema?: string; execution_ref?: string; submission_ref?: string;
  operation_ref?: string; source_ref?: string; attempt?: number; progress_ref?: string;
  state?: string; phase?: string; current_source_phase?: string; observed_generation?: number;
  posture?: string | { state: string; result_ref?: string; receipt_ref?: string; effect_ref?: string; outcome?: string; review_ref?: string; decision_ref?: string; external_execution_started?: boolean };
  runner?: { run_ref: string; posture: string; stop_requested: boolean };
}
export interface ExecutionSubmission { created?: boolean; execution: ExecutionObservation; advancement?: string; outcome?: { posture: string; reason?: string } }
export interface RuntimeBudgets { max_invocations: number; max_operations: number; max_semantic_units: number; max_resident_items: number; max_estimated_input_units: number; max_provider_retries: number; max_runtime_ms: number; stop_on_deny: boolean; continue_after_malformed: boolean }
export interface CaseRunInput { case_ref: string; participant_ref: string; resource_ref: string; submission_ref: string; task: string; budgets: RuntimeBudgets }
export interface CaseStopInput { case_ref: string; participant_ref: string; submission_ref: string; run_ref: string }
export type ResourceAction = { action: "filesystem_read" | "discover"; path: string }
  | { action: "filesystem_search"; path: string; needle: string }
  | { action: "process_run" | "database_query" | "database_mutation" | "http_fetch"; name: string }
  | { action: "mcp_catalog" };
export interface ResourceRequestInput { case_ref: string; participant_ref: string; resource_ref: string; submission_ref: string; expected_generation: number; request: { schema: "yai.resource_request.v1"; configuration_digest: string; action: ResourceAction } }
export interface ProcessAttachmentInput { case_ref: string; attachment_ref: string; pid: number; policy_owner_participant_ref: string; actions: Array<"terminate" | "suspend" | "resume">; review_requirement: "automatic" | "require_review" }

/** Window-local navigation receipts only, bounded and namespaced by authenticated
 * Case/Participant. No task text, results, authority or canonical execution store. */
export function executionKey(caseRef: string, participant: string) { return `yai.studio.execution-refs.v1:${caseRef}:${participant}`; }
export function readExecutionRefs(key: string): ExecutionReference[] {
  try { const items: unknown = JSON.parse(sessionStorage.getItem(key) ?? "[]"); return Array.isArray(items) ? items.filter((item): item is ExecutionReference => item && (item.domain === "source_acquisition" ? typeof item.source_ref === "string" && Number.isSafeInteger(item.attempt) && item.attempt > 0 : ["runtime_work", "resource_request"].includes(item.domain) && typeof item.submission_ref === "string")).slice(-12) : []; } catch { return []; }
}
export function rememberExecution(key: string, execution: ExecutionReference) {
  const next = [...readExecutionRefs(key).filter(item => JSON.stringify(item) !== JSON.stringify(execution)), execution].slice(-12);
  // Fail before submission if its recovery reference cannot be retained locally.
  sessionStorage.setItem(key, JSON.stringify(next));
  window.dispatchEvent(new CustomEvent("yai:execution-reference", { detail: { key } }));
}
