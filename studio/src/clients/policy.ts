/** Bounded presentation of the published policy.* Application results.
 * The compiler, validation, lifecycle and effective composition remain YAI-owned.
 */
export interface PolicyRule {
  kind: "operation_restriction" | "review_requirement" | "evidence_obligation" | "authority_requirement";
  rule_id: string; operation_kind: string; resource_kind?: string | null; reason: string;
  effect?: "allow" | "deny"; required?: boolean; obligation?: string;
  subject?: "proposer" | "reviewer"; required_role?: string;
  provenance: { source_id: string; fact_refs: string[]; source_locations: string[] };
}
export interface PolicyArtifactView {
  artifact: {
    artifact_id: string; policy_key: string; artifact_version: string; owner_ref: string;
    tenant_id?: string; source_id: string; source_digest: string;
    validity: { mode: string; valid_from_unix_ms?: number | null; refresh_after_unix_ms?: number | null; expires_at_unix_ms?: number | null };
    policy_ir: { compiler_version: string; ir_digest: string; rules: PolicyRule[]; unresolved: unknown[]; conflicts: Array<{ code: string; selector: string; rule_refs: string[] }> };
    validation: { status: "qualified" | "blocked"; blockers: string[]; validator_version: string };
  };
  lifecycle: "candidate" | "validated" | "published" | "superseded" | "retired" | "revoked";
  runtime_consumable: boolean; superseded_by?: string | null;
  lifecycle_events: Array<{ event_id: string; action: string; reason: string; committed_at_unix_ms: number; actor_ref: string }>;
}
export interface PolicyIngestResult { source_created: boolean; artifact_created: boolean; view: PolicyArtifactView }
export interface PolicyLifecycleResult { changed: boolean; view: PolicyArtifactView }
export type PolicyLifecycleAction = "validate" | "publish" | "retire" | "revoke";
export interface PolicyLifecycleInput { artifact_ref: string; reason: string }
