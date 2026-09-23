import type { ProviderModelsInput, ProviderModels } from "./compute";
import type { SemanticEvidence, CognitiveBinding, SuitabilityInput, CognitiveBindingInput } from "./compute";
import type { ConversationSendInput, ConversationSubmission } from "./conversation";
import type { InspectedConversationExecution } from "./executionContext";
import type { ExecutionGetInput, ExecutionObservation, ExecutionSubmission, SourceAcquireInput, SourceResumeInput, ResourceRequestInput, ProcessAttachmentInput, CaseRunInput, CaseStopInput } from "./execution";
import type { KnowledgeRequest, KnowledgeView, KnowledgeSearchResult, KnowledgeResolveResult, KnowledgeNavigationResult } from "./knowledge";
import type { WorkflowDefinitionInput, WorkflowDefinition, WorkflowBindInput, WorkflowPatchInput, WorkCommit, HandoffOfferInput, HandoffAcceptInput, HandoffDeclineInput, HandoffResultInput } from "./work";
import type { ProviderRegistration, ProviderTarget, ProviderQualificationInput, ProviderQualification, ProviderBindingInput, ProviderPosture } from "./compute";
import type { CaseCapabilityView, ApplicationCatalog, CasePolicyBindingInput, CasePolicyReplacementInput, CasePolicyUnbindingInput, SourceDeclarationInput, TenantPresentation } from "./application";
import type { PolicyIngestResult, PolicyLifecycleAction, PolicyLifecycleInput, PolicyLifecycleResult } from "./policy";
import type { IdentityPresentation } from "./application";
import type { RecallRequest, RecallResult, WorkingStateRequest, WorkingStateResult, WorkingState, WorkingRefreshRequest, PageRequest, FrontierResult, DecisionPrepareInput, DecisionPreparation } from "./memory";
export const APPLICATION_PROTOCOL = "yai.studio.application.v1";

export type ResultState =
  | "success"
  | "partial"
  | "unauthorized"
  | "stale"
  | "core_pending"
  | "not_implemented"
  | "transport_unavailable"
  | "error";

export interface OperationResult<T> {
  operation_ref: string;
  result_state: ResultState;
  correlation_ref: string;
  data?: T;
  error?: { code: string; message: string; safe_message: string; result_state: ResultState };
}

export interface LiveCaseRow {
  case_ref: string;
  display_name: string;
  case_status: string;
  generation: number;
  updated_at_unix_ms?: number;
  participant_count: number;
  source_count: number;
  resource_count: number;
  pending_review_count: number;
}

export interface CaseListProjection { cases: LiveCaseRow[]; authority: string }
export interface CaseAttachment {
  case_ref: string;
  case_status: string;
  generation: number;
  authenticated_principal: string;
  participant_ref: string;
  thread_ref?: string;
  attachment: "ephemeral";
}

export interface LiveNode { id: string; label?: string; kind?: string; detail?: string }
export interface LiveEdge { id: string; from: string; to: string; kind: string; from_kind?: string; to_kind?: string }
export interface TimelineEntry {
  id: string; sequence: number; committed_at_unix_ms: number; kind: string;
  participant_ref?: string; component: string; causal_refs: string[]; summary?: string;
}

export interface LiveWorkspace {
  case: {
    case_ref: string; display_name: string; case_status: string; generation: number;
    tenant_ref?: string; participant_ref: string; updated_at_unix_ms?: number;
  };
  overview: {
    attention: Array<{ kind: string; title: string; detail: string; ref?: string }>;
    participants: Array<{ id: string; roles: string[]; is_current: boolean }>;
  };
  environment: {
    sources: Array<{ id: string; label: string; kind: string; perimeter: string; media_type: string; roles: string[]; resource_ref: string; posture?: string; revision_ref?: string; items?: number; attempt?: number; progress_ref?: string }>;
    files: Array<{ id: string; source_ref: string; source_label: string; revision_ref: string; path: string; digest: string; bytes: number; media_type: string; backing: unknown }>;
    resources: Array<{ id: string; label?: string; kind: string; policy_ref: string; review_requirement: string; allowed_write_prefix: string; max_write_bytes: number; operations: string[]; read_prefixes: string[]; names: string[]; max_output_bytes?: number; max_items?: number; configuration_digest?: string }>;
    artifacts: unknown[];
  };
  knowledge: {
    status: string; message: string; profile?: string; source_closure?: string;
    sources: Array<{ id: string; source_ref: string; label: string; revision_ref: string; path: string; digest: string; media_type: string; extractor: string; status: string; detail: string }>;
    units: Array<{ id: string; source_ref: string; parent_ref?: string; kind: string; text: string; posture: string; entity_ref?: string; predicate?: string; value?: unknown; references: string[]; topics: string[] }>;
    entities: Array<{ id: string; definitions: string[] }>;
    topics: Array<{ name: string; units: string[] }>;
    contradictions: Array<{ id: string; entity: string; predicate: string; members: string[]; posture: string }>;
    relations: Array<LiveEdge & { posture?: string; backing_units?: string[] }>;
  };
  memory: { authority: string; timeline: TimelineEntry[]; relations: LiveEdge[]; generation: number };
  authority: { policies: Array<{ id: string; policy_key: string; lineage_ref: string; artifact_ref: string; source_ref: string; owner_ref: string; version: string; bound_at_generation: number; reason: string }>; reviews: Array<{ id: string; status: string; operation_ref: string; policy_ref: string; decision_ref?: string; evidence_ref?: string; required_roles: string[] }>; grants: Record<string, unknown>[]; last_decision?: Record<string, unknown>; empty: boolean };
  work: {
    status: string;
    message?: string;
    definition?: { name?: string; description?: string; nodes?: unknown[] };
    resolution?: { effective_topology_digest?: string; completed?: boolean; nodes: Array<{ node_id: string; node_kind: string; posture: string; reason: string; evidence_refs: string[] }> };
    nodes: Array<{ node_id: string; node_kind: string; posture: string; reason: string }>;
    edges: LiveEdge[];
  };
  compute: { cognitive_bindings?: CognitiveBinding[]; status: string; message: string; targets: Array<{ id: string; provider_key: string; adapter: string; model_id: string; locality: string; endpoint: string; posture?: ProviderPosture | string; management: string; extension_adapter_id?: string | null; semantic_evidence?: SemanticEvidence[] }> };
  conversation: { read_only: boolean; turns: Array<{ id: string; thread_ref: string; participant_ref: string; generation: number; execution_request_ref?: string | null; parts: Array<{ modality: string; media_type: string; text?: string }> }> };
  freshness: { generation: number; resync_operation: string };
}

export interface MaterialReadProjection {
  case_ref: string;
  generation: number;
  source_ref: string;
  revision_ref: string;
  path: string;
  digest: string;
  bytes: number;
  media_type: string;
  encoding: "utf-8" | "base64";
  content: string;
}

export interface CaseUpdate {
  protocol: string; event_ref: string; event_type: string; event_family: string;
  case_ref: string; generation: number; sequence: number; cursor: string; affected_views: string[];
}

declare global {
  interface Window {
    __TAURI__?: {
      core: { invoke<T>(command: string, args?: Record<string, unknown>): Promise<T> };
      event: { listen<T>(event: string, handler: (event: { payload: T }) => void): Promise<() => void> };
    };
  }
}

let correlation = 0;
export class LiveClient {
  readonly mode = "live" as const;

  private async call<T>(operation_ref: string, input: unknown = {}): Promise<OperationResult<T>> {
    const tauri = window.__TAURI__;
    if (!tauri) {
      return {
        operation_ref,
        result_state: "transport_unavailable",
        correlation_ref: `studio:web:${++correlation}`,
        error: { code: "tauri_host_unavailable", message: "tauri_host_unavailable", safe_message: "The local YAI application host is unavailable. Launch Studio as a desktop application.", result_state: "transport_unavailable" },
      };
    }
    const correlation_ref = `studio:${Date.now()}:${++correlation}`;
    try {
      return await tauri.core.invoke<OperationResult<T>>("studio_call", {
        request: { protocol: APPLICATION_PROTOCOL, operation_ref, correlation_ref, input },
      });
    } catch {
      // A dropped IPC request is never an instruction to resubmit an operation.
      return { operation_ref, correlation_ref, result_state: "transport_unavailable", error: {
        code: "host_transport_dropped", message: "host_transport_dropped",
        safe_message: "The connection to YAI was interrupted. Reconnect to refresh the current state.", result_state: "transport_unavailable",
      } };
    }
  }

  acquireSource(input: SourceAcquireInput) { return this.call<ExecutionSubmission>("source.acquire", input); }
  resumeSource(input: SourceResumeInput) { return this.call<ExecutionSubmission>("source.resume", input); }
  execution(input: ExecutionGetInput) { return this.call<ExecutionObservation>("execution.get", input); }
  requestResource(input: ResourceRequestInput) { return this.call<ExecutionSubmission>("resource.request", input); }
  attachProcess(input: ProcessAttachmentInput) { return this.call<unknown>("resource.attach_process", input); }
  runCase(input: CaseRunInput) { return this.call<ExecutionSubmission>("case.run", input); }
  stopCase(input: CaseStopInput) { return this.call<ExecutionObservation>("case.stop", input); }
  inspectKnowledge(request: KnowledgeRequest) { return this.call<KnowledgeView>("knowledge.inspect", { request }); }
  searchKnowledge(request: KnowledgeRequest, query: string, limit: number) { return this.call<KnowledgeSearchResult>("knowledge.search", { request, query, limit }); }
  resolveKnowledge(request: KnowledgeRequest, unit_ref: string) { return this.call<KnowledgeResolveResult>("knowledge.resolve", { request, unit_ref }); }
  navigateKnowledge(request: KnowledgeRequest) { return this.call<KnowledgeNavigationResult>("knowledge.navigation", { request }); }
  defineWorkflow(input: WorkflowDefinitionInput) { return this.call<WorkflowDefinition>("workflow.define", { definition: input }); }
  bindWorkflow(input: WorkflowBindInput) { return this.call<unknown>("workflow.bind", input); }
  proposeWorkflowPatch(input: { case_ref: string; patch: WorkflowPatchInput }) { return this.call<WorkCommit>("workflow.patch.propose", input); }
  adoptWorkflowPatch(input: { case_ref: string; patch_ref: string }) { return this.call<WorkCommit>("workflow.patch.adopt", input); }
  offerHandoff(input: HandoffOfferInput) { return this.call<WorkCommit>("handoff.offer", input); }
  acceptHandoff(input: HandoffAcceptInput) { return this.call<WorkCommit>("handoff.accept", input); }
  declineHandoff(input: HandoffDeclineInput) { return this.call<WorkCommit>("handoff.decline", input); }
  resultHandoff(input: HandoffResultInput) { return this.call<WorkCommit>("handoff.result.record", input); }
  reconcileHandoff(input: { source_case_ref: string; handoff_ref: string }) { return this.call<WorkCommit>("handoff.reconcile", input); }
  sendConversation(input: ConversationSendInput) { return this.call<ConversationSubmission>("conversation.send", input); }
  observeConversation(input: { case_ref: string; participant_ref: string; include_context?: boolean; execution: { domain: "conversation"; submission_ref: string } | { domain: "cognitive_composition"; request_ref: string } }) { return this.call<InspectedConversationExecution>("execution.get", input); }
  attestProvider(input: SuitabilityInput) { return this.call<SemanticEvidence>("provider.suitability.record", input); }
  bindCognition(input: CognitiveBindingInput) { return this.call<CognitiveBinding>("cognitive.binding.set", input); }
  providerInventory(tenant_id: string) { return this.call<{ tenant_ref: string; targets: LiveWorkspace["compute"]["targets"]; total_visible_targets: number; omitted: number; case_usage: string }>("provider.inventory", { tenant_id }); }
  discoverProviderModels(input: ProviderModelsInput) { return this.call<ProviderModels>("provider.models", input); }
  registerProvider(input: ProviderRegistration) { return this.call<ProviderTarget>("provider.register", input); }
  qualifyProvider(input: ProviderQualificationInput) { return this.call<ProviderQualification>("provider.qualify", input); }
  trustProvider(input: { target_ref: string; posture: "approved" | "denied" }) { return this.call<unknown>("provider.trust.set", input); }
  bindProvider(input: ProviderBindingInput) { return this.call<unknown>("provider.case.bind", input); }
  caseCapabilities(input: { case_ref: string; participant_ref: string }) { return this.call<CaseCapabilityView>("case.capabilities", input); }
  publishSource(input: { case_ref: string; source_ref: string; reason: string }) { return this.call<unknown>("source.publish", input); }
  applicationCapabilities() { return this.call<ApplicationCatalog>("application.capabilities"); }
  currentIdentity() { return this.call<IdentityPresentation>("identity.current"); }
  bootstrapIdentity(input: { tenant_id: string; organization_ref: string }) { return this.call<unknown>("identity.bootstrap", input); }
  tenant(input: { tenant_id: string }) { return this.call<TenantPresentation>("tenant.get", input); }
  recall(request: RecallRequest) { return this.call<RecallResult>("semantic.recall", { request }); }
  prepareFrontier(working_state: WorkingState, max_candidates: number) { return this.call<FrontierResult>("decision.frontier.prepare", { working_state, max_candidates }); }
  prepareDecision(input: DecisionPrepareInput) { return this.call<DecisionPreparation>("decision.request.prepare", input); }
  compileWorkingState(request: WorkingStateRequest, pageable: boolean) { return this.call<WorkingStateResult>("semantic.working_state.compile", { request, pageable }); }
  refreshWorkingState(working_state: WorkingState, request: WorkingRefreshRequest) { return this.call<WorkingStateResult>("semantic.working_state.refresh", { working_state, request }); }
  pageWorkingState(working_state: WorkingState, request: PageRequest) { return this.call<WorkingStateResult>("semantic.working_state.page", { working_state, request }); }
  admitParticipantView(input: { case_ref: string; participant_ref: string; consumer: "model"; view_kind: "model_context" }) { return this.call<unknown>("participant.view.admit", input); }
  listTenants() { return this.call<TenantPresentation[]>("tenant.list"); }
  createCase(input: { tenant_id: string; case_ref: string }) { return this.call<unknown>("case.create", input); }
  addParticipantRole(input: { case_ref: string; participant_ref: string; role: string }) { return this.call<unknown>("participant.role.add", input); }
  linkCurrentPrincipal(input: { case_ref: string; participant_ref: string }) { return this.call<unknown>("participant.principal.link", { ...input, principal_ref: "self" }); }
  caseLifecycle(action: "close" | "cancel", input: { case_ref: string; reason: string }) { return this.call<unknown>(`case.${action}`, input); }
  resolveReview(action: "approve" | "deny" | "defer", input: { case_ref: string; review_ref: string; participant_ref: string; reason: string }) { return this.call<unknown>(`review.${action}`, input); }
  recordWorkflowInput(input: { case_ref: string; node_ref: string; value: string }) { return this.call<unknown>("workflow.input.record", input); }
  declareSource(input: SourceDeclarationInput) { return this.call<unknown>("source.declare", input); }
  revokeSource(input: { case_ref: string; source_ref: string; reason: string }) { return this.call<unknown>("source.revoke", input); }
  bindPolicy(input: CasePolicyBindingInput) { return this.call<unknown>("policy.case.bind", input); }
  replacePolicy(input: CasePolicyReplacementInput) { return this.call<unknown>("policy.case.replace", input); }
  unbindPolicy(input: CasePolicyUnbindingInput) { return this.call<unknown>("policy.case.unbind", input); }
  ingestPolicy(input: { tenant_id: string; source_bytes: number[] }) { return this.call<PolicyIngestResult>("policy.ingest", input); }
  policyLifecycle(action: PolicyLifecycleAction, input: PolicyLifecycleInput) { return this.call<PolicyLifecycleResult>(`policy.${action}`, input); }
  listCases() { return this.call<CaseListProjection>("case.list"); }
  openCase(case_ref: string) { return this.call<CaseAttachment>("case.open", { case_ref, reason: "studio_local_attachment" }); }
  caseSummary(case_ref: string, expected_generation?: number) { return this.call<LiveWorkspace>("case.summary", { case_ref, expected_generation }); }
  readMaterial(input: { case_ref: string; source_ref: string; revision_ref?: string; path: string; expected_generation?: number }) { return this.call<MaterialReadProjection>("material.read", input); }
  subscribe(resume_token?: string) { return this.call("events.subscribe", { resume_token }); }
  heartbeat(case_ref: string) { return this.call<{ case_ref: string; generation: number; cursor: string; stream_state: string }>("events.heartbeat", { case_ref }); }
  async listen(handler: (update: CaseUpdate) => void): Promise<() => void> {
    if (!window.__TAURI__) return () => undefined;
    return window.__TAURI__.event.listen<CaseUpdate>("yai://case-update", (event) => handler(event.payload));
  }
}
