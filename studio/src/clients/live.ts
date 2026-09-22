import type { ApplicationCatalog, TenantPresentation } from "./application";
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
    sources: Array<{ id: string; label: string; kind: string; perimeter: string; media_type: string; roles: string[]; resource_ref: string; posture?: string; revision_ref?: string; items?: number }>;
    files: Array<{ id: string; source_ref: string; source_label: string; revision_ref: string; path: string; digest: string; bytes: number; media_type: string; backing: unknown }>;
    resources: Array<{ id: string; label?: string; kind: string; policy_ref: string; review_requirement: string; allowed_write_prefix: string; max_write_bytes: number; operations: string[]; read_prefixes: string[]; names: string[]; max_output_bytes?: number; max_items?: number }>;
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
    definition?: { nodes?: unknown[] };
    resolution?: { nodes: Array<{ node_id: string; node_kind: string; posture: string; reason: string; evidence_refs: string[] }> };
    nodes: Array<{ node_id: string; node_kind: string; posture: string; reason: string }>;
    edges: LiveEdge[];
  };
  compute: { status: string; message: string; targets: Array<{ id: string; provider_key: string; adapter: string; model_id: string; locality: string; endpoint: string; posture?: unknown; management: string }> };
  conversation: { read_only: boolean; turns: Array<{ id: string; thread_ref: string; participant_ref: string; generation: number; parts: Array<{ modality: string; media_type: string; text?: string }> }> };
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

  applicationCapabilities() { return this.call<ApplicationCatalog>("application.capabilities"); }
  listTenants() { return this.call<TenantPresentation[]>("tenant.list"); }
  createCase(input: { tenant_id: string; case_ref: string }) { return this.call<unknown>("case.create", input); }
  addParticipantRole(input: { case_ref: string; participant_ref: string; role: string }) { return this.call<unknown>("participant.role.add", input); }
  linkCurrentPrincipal(input: { case_ref: string; participant_ref: string }) { return this.call<unknown>("participant.principal.link", { ...input, principal_ref: "self" }); }
  caseLifecycle(action: "close" | "cancel", input: { case_ref: string; reason: string }) { return this.call<unknown>(`case.${action}`, input); }
  resolveReview(action: "approve" | "deny" | "defer", input: { case_ref: string; review_ref: string; participant_ref: string; reason: string }) { return this.call<unknown>(`review.${action}`, input); }
  recordWorkflowInput(input: { case_ref: string; node_ref: string; value: string }) { return this.call<unknown>("workflow.input.record", input); }
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
