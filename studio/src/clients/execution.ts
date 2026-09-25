/** Published local Application execution contracts. References survive transport;
 * observation is read-only and never authorizes a second dispatch. */
export type ExecutionReference = { domain: "runtime_work" | "resource_request"; submission_ref: string }
  | { domain: "controlled_effect"; operation_ref: string }
  | { domain: "cognitive_realization"; plan_ref: string }
  | { domain: "cognitive_composition"; request_ref: string }
  | { domain: "source_acquisition"; source_ref: string; attempt: number };
export interface ExecutionListInput { case_ref: string; participant_ref: string; limit?: number }
export interface ExecutionListEntry { execution: ExecutionReference; recorded_at_unix_ms: number }
export interface ExecutionList { schema: string; case_ref: string; participant_ref: string; generation: number; entries: ExecutionListEntry[]; limit: number; scope: string }
export function isExecutionReference(value: unknown): value is ExecutionReference {
  if (!value || typeof value !== "object") return false;
  const ref = value as Record<string, unknown>;
  const text = (name: string) => typeof ref[name] === "string" && (ref[name] as string).length > 0;
  switch (ref.domain) {
    case "source_acquisition": return text("source_ref") && Number.isSafeInteger(ref.attempt) && Number(ref.attempt) > 0;
    case "controlled_effect": return text("operation_ref");
    case "cognitive_realization": return text("plan_ref");
    case "cognitive_composition": return text("request_ref");
    case "runtime_work": case "resource_request": return text("submission_ref");
    default: return false;
  }
}
export function executionReferenceKey(ref: ExecutionReference): string {
  return JSON.stringify(ref.domain === "source_acquisition" ? [ref.domain, ref.source_ref, ref.attempt]
    : [ref.domain, ref.domain === "controlled_effect" ? ref.operation_ref : ref.domain === "cognitive_realization" ? ref.plan_ref : ref.domain === "cognitive_composition" ? ref.request_ref : ref.submission_ref]);
}
export interface ExecutionGetInput { case_ref: string; participant_ref: string; execution: ExecutionReference; include_output?: boolean }
export interface EffectProposeInput { case_ref: string; participant_ref: string; resource_ref: string; candidate_ref: string; expected_generation: number }
export interface EffectSubmitInput { case_ref: string; participant_ref: string; operation_ref: string; expected_generation: number }
export interface EffectReconcileInput extends EffectSubmitInput { effect_ref: string; retry_no_effect: boolean }
export interface ControlledOperation {
  schema: string; operation_id: string; operation_digest: string; case_id: string; participant_id: string;
  kind: "filesystem_write" | "process_signal"; resource_attachment_id: string; expected_case_generation: number;
  origin: { kind: string; provider_result_id?: string };
  filesystem_write: { relative_path: string; content: string; content_digest: string; content_bytes: number };
  process_signal?: { action: string; target_identity_digest: string } | null;
}
export type EffectProposal = { posture: "recorded"; operation: ControlledOperation } | { posture: "normalization_refused"; failure: { code: string; detail: string } };
export interface SourceAcquireInput { case_ref: string; participant_ref: string; source_ref: string; attempt: number; expected_generation: number }
export interface SourceResumeInput extends SourceAcquireInput { previous_progress_ref: string }
export interface ExecutionObservation {
  case_ref: string; participant_ref: string; schema?: string; execution_ref?: string; submission_ref?: string;
  generation?: number; continuation?: ResourceRequestInput;
  process?: {
    observation_ref: string; observed_at_unix_ms: number;
    status: { exit_code: number | null; signal: number | null; timed_out: boolean; output_limit_exceeded: boolean; elapsed_ms: number; timeout_ms: number };
    output?: { stdout: string; stderr: string; stdout_digest: string; stderr_digest: string; lossy_utf8: boolean };
  };
  operation_ref?: string; source_ref?: string; attempt?: number; progress_ref?: string;
  state?: string; phase?: string; current_source_phase?: string; observed_generation?: number;
  posture?: string | { state: string; result_ref?: string; receipt_ref?: string; effect_ref?: string; outcome?: string; review_ref?: string; decision_ref?: string; external_execution_started?: boolean };
  runner?: { run_ref: string; checkpoint_digest: string; posture: string; stop_requested: boolean };
  progress?: { status: "normalization_rejected" | "denied" | "awaiting_review" | "finalized" | "indeterminate"; operation_id?: string | null; decision_id?: string | null; review_id?: string | null; effect_id?: string | null; receipt_id?: string | null; outcome?: string | null } | null;
  plan_ref?: string; request_ref?: string;
  provider_result?: { result_id: string; output: string; invocation_id: string } | null;
  attempt_outcomes?: import("./conversation").ProviderAttemptObservation[];
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
  try { const items: unknown = JSON.parse(sessionStorage.getItem(key) ?? "[]"); return Array.isArray(items) ? items.filter(isExecutionReference).slice(-12) : []; } catch { return []; }
}
export function rememberExecution(key: string, execution: ExecutionReference, activate = true) {
  const next = [...readExecutionRefs(key).filter(item => executionReferenceKey(item) !== executionReferenceKey(execution)), execution].slice(-12);
  // Fail before submission if its recovery reference cannot be retained locally.
  sessionStorage.setItem(key, JSON.stringify(next));
  window.dispatchEvent(new CustomEvent("yai:execution-reference", { detail: { key, activate } }));
}

export interface CaseResumeInput { case_ref: string; participant_ref: string; previous_submission_ref: string; submission_ref: string; run_ref: string; checkpoint_digest: string; budgets: RuntimeBudgets }
