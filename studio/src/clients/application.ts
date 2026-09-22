import { APPLICATION_PROTOCOL, type LiveClient, type OperationResult } from "./live";
import type { HostServices } from "../platform/host";
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
export interface ApplicationAvailability {
  state: "checking" | "available" | "unavailable";
  catalog?: ApplicationCatalog; reason?: string;
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
  tenants() { return this.invoke("tenant.list", () => this.client.listTenants()); }
  createCase(input: { tenant_id: string; case_ref: string }) { return this.invoke("case.create", () => this.client.createCase(input)); }
  addParticipantRole(input: { case_ref: string; participant_ref: string; role: string }) { return this.invoke("participant.role.add", () => this.client.addParticipantRole(input)); }
  linkCurrentPrincipal(input: { case_ref: string; participant_ref: string }) { return this.invoke("participant.principal.link", () => this.client.linkCurrentPrincipal(input)); }
  caseLifecycle(action: "close" | "cancel", input: { case_ref: string; reason: string }) { return this.invoke(`case.${action}`, () => this.client.caseLifecycle(action, input)); }
  resolveReview(action: "approve" | "deny" | "defer", input: { case_ref: string; review_ref: string; participant_ref: string; reason: string }) { return this.invoke(`review.${action}`, () => this.client.resolveReview(action, input)); }
  recordWorkflowInput(input: { case_ref: string; node_ref: string; value: string }) { return this.invoke("workflow.input.record", () => this.client.recordWorkflowInput(input)); }
  dispose() { this.epoch++; this.stop.dispose(); this.listeners.clear(); }
}
