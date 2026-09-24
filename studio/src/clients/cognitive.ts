/** Existing engine-owned plan identity; Studio never builds requirement hashes. */
export type CognitiveCapability = "primary_conversation" | "speech_to_text" | "image_understanding";
export interface CognitivePrepareInput {
  case_ref: string; participant_ref: string; source_turn_ref: string;
  source_part_refs: string[]; capability: CognitiveCapability;
}
export interface CognitiveRealizeInput extends CognitivePrepareInput { plan_ref: string }
export interface CognitivePlan {
  plan_id: string; case_id: string; participant_id: string; case_generation: number;
  capability: CognitiveCapability; route: "native" | "derived" | "unresolved";
  role: "primary" | "auxiliary" | "none"; selected_target_id?: string;
  selected_binding_id?: string; semantic_evidence_id?: string; unresolved_reason?: string;
  arbitration?: { candidates: Array<{ target_id: string; role: string; exclusions: string[] }> };
}

/** Explicit intent for an existing Turn; source parts must be exact disclosed identities. */
export interface CognitiveComposeInput {
  case_ref: string; participant_ref: string; source_turn_ref: string;
  source_part_refs: string[]; expected_generation: number;
  prerequisite: { capability: Exclude<CognitiveCapability, "primary_conversation">; source_part_ids: string[] } | null;
}
