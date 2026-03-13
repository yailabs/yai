#include "../internal.h"

int yai_governance_decision_to_audit_blob(const yai_governance_decision_t *decision, char *out, size_t out_cap) {
  if (!decision || !out || out_cap == 0) return -1;
  return yai_governance_safe_snprintf(out,
                               out_cap,
                               "{\"type\":\"yai.decision_record.v1\",\"schema_version\":\"v1\","
                               "\"decision_id\":\"%s\",\"family_id\":\"%s\",\"domain_id\":\"%s\","
                               "\"specialization_id\":\"%s\",\"effect\":\"%s\",\"rationale\":\"%s\","
                               "\"stack_ref\":\"%s\",\"precedence_trace\":\"%s\","
                               "\"regulatory_overlay_count\":%d,\"sector_overlay_count\":%d,\"contextual_overlay_count\":%d,"
                               "\"authority_profile\":\"%s\",\"evidence_profile\":\"%s\","
                               "\"authority_contributor_count\":%d,\"evidence_contributor_count\":%d,"
                               "\"lifecycle\":{\"data_class\":\"decision_record\",\"tier\":\"hot\","
                               "\"retention_profile\":\"decision.default.v1\",\"partition_scope\":\"workspace_id,time_window\","
                               "\"lineage_required\":true,\"compactable\":true,\"archive_eligible\":true}}",
                               decision->decision_id,
                               decision->family_id,
                               decision->domain_id,
                               decision->specialization_id,
                               yai_governance_effect_name(decision->final_effect),
                               decision->rationale,
                               decision->stack.stack_id,
                               decision->stack.precedence_trace,
                               decision->stack.regulatory_overlay_count,
                               decision->stack.sector_overlay_count,
                               decision->stack.contextual_overlay_count,
                               decision->stack.authority_profile,
                               decision->stack.evidence_profile,
                               decision->stack.authority_contributor_count,
                               decision->stack.evidence_contributor_count);
}
