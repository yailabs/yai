import type { ProviderProbeInput } from "./compute";
import type { DecisionHistoryInput, DecisionInspectInput } from "./work";
import { providerCatalogKey, readProviderCatalogCapacity, type ProviderCatalogObservation, type ProviderModelsInput } from "./compute";
import type { SuitabilityInput, CognitiveBindingInput } from "./compute";
import type { ConversationSendInput } from "./conversation";
import type { MachineRegisterInput, MachineRevokeInput } from "./machines";
import type { CognitiveComposeInput, CognitivePrepareInput, CognitiveRealizeInput } from "./cognitive";
import type { EffectProposeInput, EffectSubmitInput, EffectReconcileInput, ExecutionGetInput, ExecutionListInput, SourceAcquireInput, SourceResumeInput, ResourceRequestInput, ProcessAttachmentInput, CaseRunInput, CaseStopInput, CaseResumeInput } from "./execution";
import type { KnowledgeRequest } from "./knowledge";
import type { WorkflowDefinitionInput, WorkflowBindInput, WorkflowPatchInput, HandoffOfferInput, HandoffAcceptInput, HandoffDeclineInput, HandoffResultInput } from "./work";
import type { ProviderRegistration, ProviderQualificationInput, ProviderBindingInput } from "./compute";
import { APPLICATION_PROTOCOL, type LiveClient, type OperationResult } from "./live";
import type { HostServices } from "../platform/host";
import type { PolicyLifecycleAction, PolicyLifecycleInput } from "./policy";
import type { RecallRequest, WorkingStateRequest, WorkingState, WorkingRefreshRequest, PageRequest, DecisionPrepareInput } from "./memory";
import { toDisposable, type Disposable } from "../platform/lifecycle";

export interface ApplicationOperation {
  operation_id: string; meaning: string; input_contract: string; output_contract: string;
  impact: string; authority: string;
}
export interface ApplicationCapability {
  capability_id: string; meaning: string; application_posture: "ready" | "deferred" | "not_applicable";
  application_operation_ids: string[]; application_deferred_reason?: string;
  studio_posture: string;
}
export interface ApplicationCatalog {
  schema: "yai.application_capability_catalog.v1"; application_protocol: string;
  operations: ApplicationOperation[]; capabilities: ApplicationCapability[];
}
export interface TenantPresentation { membership: string; tenant: { tenant_id: string; organization_ref: string } }
export interface IdentityPresentation { principal: { principal_id: string; authentication_method: string }; tenants: TenantPresentation[]; authentication: { binding_ref: string } }
/** Authored source setup; action values are the published Application contract. */
export interface SourceDeclarationInput {
  case_ref: string; perimeter: string; logical_name: string; participant_ref: string; resource_ref: string;
  roles: Array<"knowledge" | "operational" | "policy">;
  action: { action: "discover"; path: string } | { action: "database_query" | "http_fetch"; name: string };
  bootstrap_policy: boolean; media_type: string;
}
export interface ResourceImportInput {
  case_ref: string;
  definition: {
    schema: "yai.resource_definition.v1"; attachment_id: string; policy_owner: string;
    participant_ids: string[]; operations: string[]; read_prefixes: string[]; names: string[];
    max_output_bytes: number; max_items: number; review_requirement: "require_review";
    address: { kind: "filesystem" | "discovery"; root: string }
      | { kind: "process_runner"; root: string; runners: Record<string, { executable: string; executable_digest: string; argv: string[]; working_directory: string; environment: Record<string, string>; timeout_ms: number }> }
      | { kind: "sqlite"; root: string; path: string; queries: Record<string, string> }
      | { kind: "http_service"; endpoint: NetworkResourceInput; paths: Record<string, string> }
      | { kind: "mcp"; endpoint: NetworkResourceInput };
  };
}
interface NetworkResourceInput { endpoint: string; allowed_ip_addresses: string[]; credential_ref: string | null }
export interface CasePolicyBindingInput { case_ref: string; artifact_ref: string; expected_generation: number; reason: string }
export interface CasePolicyReplacementInput extends CasePolicyBindingInput { prior_binding_ref: string }
export interface CasePolicyUnbindingInput { case_ref: string; binding_ref: string; expected_generation: number; reason: string }
export type ResourceOperationKind = "filesystem_write" | "process_signal" | { resource_access: string };
export function resourceOperationLabel(kind: ResourceOperationKind): string {
  return (typeof kind === "string" ? kind : kind.resource_access).replaceAll("_", " ");
}
export interface CaseCapabilityView {
  case_id: string; case_generation: number; participant_id: string; view_id: string; effective_policy_id: string;
  entries: Array<{ resource: { attachment_id: string }; operation_kind: ResourceOperationKind; requires_current_decision: boolean; policy_constraints: Array<{ kind: string; effect?: string; required?: boolean; resolution: string }> }>;
  exclusions: Array<{ resource_id: string; operation_kind: ResourceOperationKind; reason: string }>;
}
export interface ApplicationAvailability {
  state: "checking" | "available" | "unavailable";
  catalog?: ApplicationCatalog; reason?: string;
  providerCatalogs?: Readonly<Record<string, ProviderCatalogObservation>>;
}

export function isApplicationCatalog(value: unknown): value is ApplicationCatalog {
  if (!value || typeof value !== "object") return false;
  const c = value as Partial<ApplicationCatalog>;
  return c.schema === "yai.application_capability_catalog.v1" && c.application_protocol === APPLICATION_PROTOCOL
    && Array.isArray(c.operations) && c.operations.every(op => op && typeof op.operation_id === "string" && typeof op.meaning === "string" && typeof op.impact === "string" && typeof op.authority === "string")
    && Array.isArray(c.capabilities) && c.capabilities.every(item => item && typeof item.capability_id === "string" && Array.isArray(item.application_operation_ids) && item.application_operation_ids.every(id => typeof id === "string"));
}

/** Current Host contract discovery, never a policy evaluator or an operation registry. */
export class ApplicationAccess implements Disposable {
  private value: ApplicationAvailability = { state: "checking" };
  private listeners = new Set<() => void>();
  private epoch = 0;
  private hostStamp = "";
  private readonly stop: Disposable;
  constructor(private readonly client: LiveClient, private readonly host: HostServices) {
    this.stop = host.subscribe(() => this.hostChanged());
    this.hostChanged();
  }
  snapshot = () => this.value;
  subscribe = (listener: () => void) => { this.listeners.add(listener); return toDisposable(() => this.listeners.delete(listener)); };
  supports(operation: string) { return this.value.state === "available" && this.value.catalog?.operations.some(item => item.operation_id === operation) === true; }
  reason(operation: string) { return this.value.reason ?? (this.value.state === "checking" ? "Checking the connected YAI Host…" : `The connected Host does not expose ${operation}.`); }
  private publish(value: ApplicationAvailability) { this.value = value; this.listeners.forEach(listener => listener()); }
  private hostChanged() {
    const current = this.host.snapshot();
    const stamp = `${current.state}:${current.telemetry?.instance_id ?? ""}`;
    if (stamp === this.hostStamp) return;
    this.hostStamp = stamp;
    if (current.state === "live") void this.refresh();
    else { this.epoch++; this.publish({ state: "unavailable", reason: current.reason ?? (this.host.capabilities.nativeDesktop ? `YAI Host is ${current.state}.` : "A native desktop connection is required.") }); }
  }
  async refresh() {
    if (this.host.snapshot().state !== "live") return;
    const epoch = ++this.epoch;
    this.publish({ state: "checking" });
    const result = await this.client.applicationCapabilities();
    if (epoch !== this.epoch) return;
    this.publish(result.result_state === "success" && isApplicationCatalog(result.data)
      ? { state: "available", catalog: result.data }
      : { state: "unavailable", reason: result.error?.safe_message ?? "The Host did not return a compatible capability catalog." });
  }
  private unavailable<T>(operation: string): OperationResult<T> {
    return { operation_ref: operation, correlation_ref: "studio:capability-not-dispatched", result_state: "not_implemented", error: { code: "host_operation_unavailable", message: operation, safe_message: this.reason(operation), result_state: "not_implemented" } };
  }
  private invoke<T>(operation: string, action: () => Promise<OperationResult<T>>) {
    return this.supports(operation) ? action() : Promise.resolve(this.unavailable<T>(operation));
  }
  proposeEffect(input: EffectProposeInput) { return this.invoke("effect.propose", () => this.client.proposeEffect(input)); }
  reconcileEffect(input: EffectReconcileInput) { return this.invoke("effect.reconcile", () => this.client.reconcileEffect(input)); }
  submitEffect(input: EffectSubmitInput) { return this.invoke("effect.submit", () => this.client.submitEffect(input)); }
  acquireSource(input: SourceAcquireInput) { return this.invoke("source.acquire", () => this.client.acquireSource(input)); }
  resumeSource(input: SourceResumeInput) { return this.invoke("source.resume", () => this.client.resumeSource(input)); }
  executions(input: ExecutionListInput) { return this.invoke("execution.list", () => this.client.executions(input)); }
  execution(input: ExecutionGetInput) { return this.invoke("execution.get", () => this.client.execution(input)); }
  composeCognition(input: CognitiveComposeInput) { return this.invoke("cognitive.compose", () => this.client.composeCognition(input)); }
  prepareCognition(input: CognitivePrepareInput) { return this.invoke("cognitive.realization.prepare", () => this.client.prepareCognition(input)); }
  realizeCognition(input: CognitiveRealizeInput) { return this.invoke("cognitive.realize", () => this.client.realizeCognition(input)); }
  requestResource(input: ResourceRequestInput) { return this.invoke("resource.request", () => this.client.requestResource(input)); }
  importResource(input: ResourceImportInput) { return this.invoke("resource.import", () => this.client.importResource(input)); }
  attachProcess(input: ProcessAttachmentInput) { return this.invoke("resource.attach_process", () => this.client.attachProcess(input)); }
  resumeCase(input: CaseResumeInput) { return this.invoke("case.resume", () => this.client.resumeCase(input)); }
  runCase(input: CaseRunInput) { return this.invoke("case.run", () => this.client.runCase(input)); }
  stopCase(input: CaseStopInput) { return this.invoke("case.stop", () => this.client.stopCase(input)); }
  inspectKnowledge(request: KnowledgeRequest) { return this.invoke("knowledge.inspect", () => this.client.inspectKnowledge(request)); }
  searchKnowledge(request: KnowledgeRequest, query: string, limit: number) { return this.invoke("knowledge.search", () => this.client.searchKnowledge(request, query, limit)); }
  resolveKnowledge(request: KnowledgeRequest, unit_ref: string) { return this.invoke("knowledge.resolve", () => this.client.resolveKnowledge(request, unit_ref)); }
  navigateKnowledge(request: KnowledgeRequest) { return this.invoke("knowledge.navigation", () => this.client.navigateKnowledge(request)); }
  defineWorkflow(input: WorkflowDefinitionInput) { return this.invoke("workflow.define", () => this.client.defineWorkflow(input)); }
  bindWorkflow(input: WorkflowBindInput) { return this.invoke("workflow.bind", () => this.client.bindWorkflow(input)); }
  proposeWorkflowPatch(input: { case_ref: string; patch: WorkflowPatchInput }) { return this.invoke("workflow.patch.propose", () => this.client.proposeWorkflowPatch(input)); }
  adoptWorkflowPatch(input: { case_ref: string; patch_ref: string }) { return this.invoke("workflow.patch.adopt", () => this.client.adoptWorkflowPatch(input)); }
  pendingHandoffs(input: { case_ref: string }) { return this.invoke("handoff.pending", () => this.client.pendingHandoffs(input)); }
  inspectHandoff(input: { case_ref: string; handoff_ref: string }) { return this.invoke("handoff.inspect", () => this.client.inspectHandoff(input)); }
  offerHandoff(input: HandoffOfferInput) { return this.invoke("handoff.offer", () => this.client.offerHandoff(input)); }
  acceptHandoff(input: HandoffAcceptInput) { return this.invoke("handoff.accept", () => this.client.acceptHandoff(input)); }
  declineHandoff(input: HandoffDeclineInput) { return this.invoke("handoff.decline", () => this.client.declineHandoff(input)); }
  resultHandoff(input: HandoffResultInput) { return this.invoke("handoff.result.record", () => this.client.resultHandoff(input)); }
  reconcileHandoff(input: { source_case_ref: string; handoff_ref: string }) { return this.invoke("handoff.reconcile", () => this.client.reconcileHandoff(input)); }
  sendConversation(input: ConversationSendInput) { return this.invoke("conversation.send", () => this.client.sendConversation(input)); }
  observeConversation(input: Parameters<LiveClient["observeConversation"]>[0]) { return this.invoke("execution.get", () => this.client.observeConversation(input)); }
  attestProvider(input: SuitabilityInput) { return this.invoke("provider.suitability.record", () => this.client.attestProvider(input)); }
  bindCognition(input: CognitiveBindingInput) { return this.invoke("cognitive.binding.set", () => this.client.bindCognition(input)); }
  providerInventory(tenant: string) { return this.invoke("provider.inventory", () => this.client.providerInventory(tenant)); }
  listMachines(tenant: string) { return this.invoke("machine.list", () => this.client.listMachines(tenant)); }
  getMachine(tenant: string, asset: string) { return this.invoke("machine.get", () => this.client.getMachine(tenant, asset)); }
  registerMachine(input: MachineRegisterInput) { return this.invoke("machine.register", () => this.client.registerMachine(input)); }
  revokeMachine(input: MachineRevokeInput) { return this.invoke("machine.revoke", () => this.client.revokeMachine(input)); }
  async discoverProviderModels(input: ProviderModelsInput) {
    const invoke = () => this.invoke("provider.models", () => this.client.discoverProviderModels(input));
    if (!("target_ref" in input)) return invoke();
    const key = providerCatalogKey(input.tenant_id, input.target_ref);
    const epoch = this.epoch;
    const pending: ProviderCatalogObservation = { state: "checking" };
    const publish = (observation: ProviderCatalogObservation) => {
      // Bound window-local metadata; no persistence, extra probe or Case mutation.
      const entries = Object.entries(this.value.providerCatalogs ?? {}).filter(([id]) => id !== key).slice(-63);
      this.publish({ ...this.value, providerCatalogs: Object.fromEntries([...entries, [key, observation]]) });
    };
    publish(pending);
    const current = () => epoch === this.epoch && this.value.providerCatalogs?.[key] === pending;
    try {
      const result = await invoke();
      if (current()) {
        const data = result.data;
        if (result.result_state === "success" && data?.target_ref === input.target_ref
          && Array.isArray(data.models) && data.models.every(model => typeof model === "string")
          && typeof data.observed_at_unix_ms === "number" && Number.isSafeInteger(data.observed_at_unix_ms)) {
          const capacity = data.capacity == null ? undefined : readProviderCatalogCapacity(data.capacity, data.models);
          if (data.capacity != null && !capacity) publish({ state: "unavailable", empty: false,
            reason: "The Host returned mismatched catalog capacity identity. Observation withheld." });
          else publish({ state: "observed", models: data.models, at: data.observed_at_unix_ms, capacity });
        } else publish({ state: "unavailable", empty: result.error?.code.startsWith("provider_catalog_empty") ?? false,
          reason: result.error?.safe_message ?? "The Host returned no matching timestamped target catalog." });
      }
      return result;
    } catch (error) {
      if (current()) publish({ state: "unavailable", empty: false, reason: "Catalog observation unavailable. Check the Host connection and retry." });
      throw error;
    }
  }
  registerProvider(input: ProviderRegistration) { return this.invoke("provider.register", () => this.client.registerProvider(input)); }
  probeProvider(input: ProviderProbeInput) { return this.invoke("provider.probe", () => this.client.probeProvider(input)); }
  listProviderProbes(target: string) { return this.invoke("provider.probe.list", () => this.client.listProviderProbes(target)); }
  observeProviderProbe(input: Pick<ProviderProbeInput, "target_ref" | "submission_ref">) { return this.invoke("provider.probe.get", () => this.client.observeProviderProbe(input)); }
  qualifyProvider(input: ProviderQualificationInput) { return this.invoke("provider.qualify", () => this.client.qualifyProvider(input)); }
  trustProvider(input: { target_ref: string; posture: "approved" | "denied" }) { return this.invoke("provider.trust.set", () => this.client.trustProvider(input)); }
  bindProvider(input: ProviderBindingInput) { return this.invoke("provider.case.bind", () => this.client.bindProvider(input)); }
  caseCapabilities(input: { case_ref: string; participant_ref: string }) { return this.invoke("case.capabilities", () => this.client.caseCapabilities(input)); }
  publishSource(input: { case_ref: string; source_ref: string; reason: string }) { return this.invoke("source.publish", () => this.client.publishSource(input)); }
  tenants() { return this.invoke("tenant.list", () => this.client.listTenants()); }
  identity() { return this.invoke("identity.current", () => this.client.currentIdentity()); }
  bootstrapIdentity(input: { tenant_id: string; organization_ref: string }) { return this.invoke("identity.bootstrap", () => this.client.bootstrapIdentity(input)); }
  tenant(input: { tenant_id: string }) { return this.invoke("tenant.get", () => this.client.tenant(input)); }
  recall(request: RecallRequest) { return this.invoke("semantic.recall", () => this.client.recall(request)); }
  decisionCorpus(input: DecisionHistoryInput) { return this.invoke("decision.trajectory.corpus", () => this.client.decisionCorpus(input)); }
  inspectDecision(input: DecisionInspectInput) { return this.invoke("decision.trajectory.inspect", () => this.client.inspectDecision(input)); }
  evaluateDecisions(input: DecisionHistoryInput) { return this.invoke("decision.trajectory.evaluate", () => this.client.evaluateDecisions(input)); }
  prepareFastSearch(working_state: WorkingState, max_candidates: number) { return this.invoke("semantic.fast_search.prepare", () => this.client.prepareFastSearch(working_state, max_candidates)); }
  prepareFrontier(working_state: WorkingState, max_candidates: number) { return this.invoke("decision.frontier.prepare", () => this.client.prepareFrontier(working_state, max_candidates)); }
  prepareDecision(input: DecisionPrepareInput) { return this.invoke("decision.request.prepare", () => this.client.prepareDecision(input)); }
  compileWorkingState(request: WorkingStateRequest, pageable: boolean) { return this.invoke("semantic.working_state.compile", () => this.client.compileWorkingState(request, pageable)); }
  refreshWorkingState(state: WorkingState, request: WorkingRefreshRequest) { return this.invoke("semantic.working_state.refresh", () => this.client.refreshWorkingState(state, request)); }
  pageWorkingState(state: WorkingState, request: PageRequest) { return this.invoke("semantic.working_state.page", () => this.client.pageWorkingState(state, request)); }
  admitParticipantView(input: { case_ref: string; participant_ref: string; consumer: "model"; view_kind: "model_context" }) { return this.invoke("participant.view.admit", () => this.client.admitParticipantView(input)); }
  createCase(input: { tenant_id: string; case_ref: string }) { return this.invoke("case.create", () => this.client.createCase(input)); }
  addParticipantRole(input: { case_ref: string; participant_ref: string; role: string }) { return this.invoke("participant.role.add", () => this.client.addParticipantRole(input)); }
  linkCurrentPrincipal(input: { case_ref: string; participant_ref: string }) { return this.invoke("participant.principal.link", () => this.client.linkCurrentPrincipal(input)); }
  caseLifecycle(action: "close" | "cancel", input: { case_ref: string; reason: string }) { return this.invoke(`case.${action}`, () => this.client.caseLifecycle(action, input)); }
  resolveReview(action: "approve" | "deny" | "defer", input: { case_ref: string; review_ref: string; participant_ref: string; reason: string }) { return this.invoke(`review.${action}`, () => this.client.resolveReview(action, input)); }
  recordWorkflowInput(input: { case_ref: string; node_ref: string; value: string }) { return this.invoke("workflow.input.record", () => this.client.recordWorkflowInput(input)); }
  declareSource(input: SourceDeclarationInput) { return this.invoke("source.declare", () => this.client.declareSource(input)); }
  revokeSource(input: { case_ref: string; source_ref: string; reason: string }) { return this.invoke("source.revoke", () => this.client.revokeSource(input)); }
  bindPolicy(input: CasePolicyBindingInput) { return this.invoke("policy.case.bind", () => this.client.bindPolicy(input)); }
  replacePolicy(input: CasePolicyReplacementInput) { return this.invoke("policy.case.replace", () => this.client.replacePolicy(input)); }
  unbindPolicy(input: CasePolicyUnbindingInput) { return this.invoke("policy.case.unbind", () => this.client.unbindPolicy(input)); }
  ingestPolicy(input: { tenant_id: string; source_bytes: number[] }) { return this.invoke("policy.ingest", () => this.client.ingestPolicy(input)); }
  policyLifecycle(action: PolicyLifecycleAction, input: PolicyLifecycleInput) { return this.invoke(`policy.${action}`, () => this.client.policyLifecycle(action, input)); }
  dispose() { this.epoch++; this.stop.dispose(); this.listeners.clear(); }
}
