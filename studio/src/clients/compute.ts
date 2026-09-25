/** Published provider governance payloads. Endpoint configuration is not runtime identity. */
export interface ProviderRegistration {
  tenant_id: string; provider_key: string; adapter: "open_ai_compatible";
  endpoint: string; model_id: string; credential_ref: string;
  locality: "loopback" | "private_network" | "remote"; extension_adapter_id?: "yvex.http.v1";
}
export interface ProviderTarget extends ProviderRegistration { target_id: string; integrity_digest: string }
export interface ProviderProbeEvidence {
  run_id: string; target_id: string; started_at_unix_ms: number; completed_at_unix_ms: number;
  transport_connected: boolean; exact_model_addressed: boolean; chat_text_envelope_valid: boolean;
  structured_json_object_valid: boolean; usage_accounting_observed: boolean;
  health_endpoint_observed: boolean; extension_telemetry_observed: boolean;
  text_embedding_envelope_valid?: boolean; embedding_dimension?: number;
  realization_shapes?: string[]; failure_codes: string[];
}
export interface ProviderQualificationInput { target_ref: string; evidence: ProviderProbeEvidence; suite_ref: string; valid_until_unix_ms?: number }
export interface ProviderQualification { qualification_id: string; target_id: string; suite_id: string; run_id: string; evidence: ProviderProbeEvidence; capabilities: Array<{ capability: string; provenance: string; evidence_refs: string[] }> }
export interface ProviderBindingInput { case_ref: string; participant_ref: string; ordered_target_refs: string[]; failover_policy: "none" | "safe_only"; max_attempts_per_turn: number }
export interface ProviderPosture {
  qualification?: { id: string; suite_id: string; run_id: string; qualified_at_unix_ms: number; valid_until_unix_ms?: number; capabilities: Array<{ capability: string; provenance: string; evidence_refs: string[] }> };
  trust?: { event_ref: string; posture: string; recorded_at_unix_ms: number };
  health: { posture: string; circuit: string; consecutive_failures: number; observed_at_unix_ms?: number; failure_class?: string };
}

/** Read an operator-supplied, already measured record; never manufacture successful probes. */
export function readProbeEvidence(value: unknown): ProviderProbeEvidence | undefined {
  if (!value || typeof value !== "object") return;
  const record = value as Record<string, unknown>;
  const candidate = (record.evidence ?? record) as ProviderProbeEvidence;
  if (!candidate || typeof candidate !== "object" || typeof candidate.run_id !== "string" || typeof candidate.target_id !== "string") return;
  if (![candidate.started_at_unix_ms, candidate.completed_at_unix_ms].every(Number.isSafeInteger)) return;
  if (![candidate.transport_connected, candidate.exact_model_addressed, candidate.chat_text_envelope_valid, candidate.structured_json_object_valid, candidate.usage_accounting_observed, candidate.health_endpoint_observed, candidate.extension_telemetry_observed].every(item => typeof item === "boolean")) return;
  if (!Array.isArray(candidate.failure_codes) || !candidate.failure_codes.every(item => typeof item === "string")) return;
  return candidate;
}

export interface SemanticEvidence { evidence_id: string; target_id: string; capability: string; posture: string; suite_id: string; run_id: string }
export interface CognitiveBinding { binding_id: string; participant_id: string; role: string; capability: string; target_id: string; semantic_evidence_id: string }
export interface SuitabilityInput { target_ref: string; capability: "primary_conversation"; suite_ref: string; run_ref: string; evidence_refs: string[] }
export interface CognitiveBindingInput { case_ref: string; participant_ref: string; role: "primary"; capability: "primary_conversation"; candidates: Array<{ target_ref: string; semantic_evidence_ref: string }>; replace: boolean }

export interface ProviderConnectionModelsInput { tenant_id: string; endpoint: string; locality: ProviderRegistration["locality"]; credential_ref: string }
export type ProviderModelsInput = ProviderConnectionModelsInput | { tenant_id: string; target_ref: string };
export interface ProviderModels { target_ref?: string | null; observed_at_unix_ms?: number; models: string[]; capacity?: ProviderCatalogCapacity | null; scope: "currently_exposed"; authority: "provider_metadata_only" }

/** A dated public catalog claim for one exact deployment; not a reservation,
 * successful request or proof that the engine remains resident. */
export interface ProviderCatalogCapacity {
  public_contract: "yvex.openai.compat.v3"; observation_kind: "public_model_catalog";
  model_id: string; engine_generation: number;
  runtime_binding_identity: string; runtime_model_identity: string; capacity_plan_identity: string;
  input_capacity_tokens: number; sequence_capacity_tokens: number; http_body_limit_bytes: number;
  resource_reservation: false; execution_or_resources_qualified: false;
}
export function readProviderCatalogCapacity(value: unknown, models: string[]): ProviderCatalogCapacity | undefined {
  if (!value || typeof value !== "object") return;
  const item = value as Record<string, unknown>;
  if (item.public_contract !== "yvex.openai.compat.v3" || item.observation_kind !== "public_model_catalog"
    || typeof item.model_id !== "string" || !models.includes(item.model_id)
    || item.resource_reservation !== false || item.execution_or_resources_qualified !== false) return;
  if (![item.engine_generation, item.input_capacity_tokens, item.sequence_capacity_tokens, item.http_body_limit_bytes]
    .every(number => Number.isSafeInteger(number) && (number as number) > 0)) return;
  if (![item.runtime_binding_identity, item.runtime_model_identity, item.capacity_plan_identity]
    .every(identity => typeof identity === "string" && identity.length > 0 && identity.length <= 256)) return;
  return item as unknown as ProviderCatalogCapacity;
}

/** Window-local, timestamped catalog metadata. Host loss invalidates it. */
export type ProviderCatalogObservation =
  | { state: "checking" }
  | { state: "observed"; models: string[]; at: number; capacity?: ProviderCatalogCapacity }
  | { state: "unavailable"; reason: string; empty: boolean };
export function providerCatalogKey(tenant: string, target: string): string {
  return JSON.stringify([tenant, target]);
}

/** Exact retained synthetic qualification; never Case content or administrative trust. */
export interface ProviderProbeInput {
  target_ref: string; submission_ref: string; embedding: boolean;
  realization_shapes: Array<"text_to_text" | "text_functions_to_text_or_call" | "text_to_json_object">;
  qualify: boolean; valid_for_ms?: number;
}
export interface ProviderProbeExecution {
  schema: "yai.provider_probe_execution.v1"; target_ref: string; submission_ref: string;
  created: boolean; posture: "running" | "completed" | "failed" | "interrupted";
  run: {
    request: { target_id: string; submission_ref: string; embedding: boolean; realization_shapes: string[]; qualify: boolean; valid_for_ms?: number | null };
    evidence?: ProviderProbeEvidence | null; qualification?: ProviderQualification | null;
    failure_code?: string | null;
    owner: { started_at_unix_ms: number };
  };
}

export interface ProviderProbeList { schema: "yai.provider_probe_list.v1"; target_ref: string; runs: ProviderProbeExecution[] }
