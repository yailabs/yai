#include <yai/policy/governance/evidence_map.h>

#include "../internal.h"

#include <string.h>

int yai_governance_decision_to_evidence(const yai_governance_decision_t *decision,
                                 const char *trace_id,
                                 const char *provider,
                                 const char *resource,
                                 const char *authority_context,
                                 yai_governance_evidence_envelope_t *out) {
  if (!decision || !trace_id || !out) return -1;
  memset(out, 0, sizeof(*out));

  (void)yai_governance_safe_snprintf(out->trace_id, sizeof(out->trace_id), "%s", trace_id);
  (void)yai_governance_safe_snprintf(out->decision_id, sizeof(out->decision_id), "%s", decision->decision_id);
  (void)yai_governance_safe_snprintf(out->domain_id, sizeof(out->domain_id), "%s", decision->domain_id);
  (void)yai_governance_safe_snprintf(out->family_id, sizeof(out->family_id), "%s", decision->family_id);
  (void)yai_governance_safe_snprintf(out->specialization_id, sizeof(out->specialization_id), "%s", decision->specialization_id);
  (void)yai_governance_safe_snprintf(out->final_effect, sizeof(out->final_effect), "%s", yai_governance_effect_name(decision->final_effect));
  (void)yai_governance_safe_snprintf(out->provider, sizeof(out->provider), "%s", provider ? provider : "unknown");
  (void)yai_governance_safe_snprintf(out->resource, sizeof(out->resource), "%s", resource ? resource : "unknown");
  (void)yai_governance_safe_snprintf(out->authority_context, sizeof(out->authority_context), "%s", authority_context ? authority_context : "unknown");
  out->review_trace_required = decision->final_effect == YAI_GOVERNANCE_EFFECT_REVIEW_REQUIRED ? 1 : 0;
  out->retention_required = 0;
  out->provenance_required = 0;
  out->approval_chain_required = 0;
  out->dependency_chain_required = 0;
  out->lawful_basis_required = 0;
  out->oversight_trace_required = 0;
  for (int i = 0; i < decision->evidence_requirement_count; ++i) {
    if (strstr(decision->evidence_requirements[i], "retention")) out->retention_required = 1;
    if (strstr(decision->evidence_requirements[i], "provenance")) out->provenance_required = 1;
    if (strstr(decision->evidence_requirements[i], "review_trace")) out->review_trace_required = 1;
    if (strstr(decision->evidence_requirements[i], "approval_chain")) out->approval_chain_required = 1;
    if (strstr(decision->evidence_requirements[i], "dependency_chain")) out->dependency_chain_required = 1;
    if (strstr(decision->evidence_requirements[i], "lawful_basis")) out->lawful_basis_required = 1;
    if (strstr(decision->evidence_requirements[i], "oversight_trace")) out->oversight_trace_required = 1;
  }

  return 0;
}

int yai_governance_evidence_to_record_blob(const yai_governance_decision_t *decision,
                                    const yai_governance_evidence_envelope_t *evidence,
                                    char *out,
                                    size_t out_cap) {
  if (!decision || !evidence || !out || out_cap == 0) return -1;
  return yai_governance_safe_snprintf(
      out,
      out_cap,
      "{\"type\":\"yai.evidence_record.v1\",\"schema_version\":\"v1\","
      "\"decision_ref\":\"%s\",\"trace_ref\":\"%s\",\"workspace_scope\":\"workspace\","
      "\"domain_id\":\"%s\",\"family_id\":\"%s\",\"specialization_id\":\"%s\","
      "\"final_effect\":\"%s\",\"provider\":\"%s\",\"resource\":\"%s\","
      "\"authority_context\":\"%s\","
      "\"evidence_profile\":\"%s\","
      "\"requirements\":{\"review_trace_required\":%s,\"retention_required\":%s,"
      "\"provenance_required\":%s,\"approval_chain_required\":%s,"
      "\"dependency_chain_required\":%s,\"lawful_basis_required\":%s,"
      "\"oversight_trace_required\":%s},"
      "\"lifecycle\":{\"data_class\":\"evidence_record\",\"tier\":\"hot\","
      "\"retention_profile\":\"evidence.default.v1\",\"partition_scope\":\"workspace_id,time_window\","
      "\"lineage_required\":true,\"compactable\":true,\"archive_eligible\":true}}",
      evidence->decision_id,
      evidence->trace_id,
      evidence->domain_id,
      evidence->family_id,
      evidence->specialization_id,
      evidence->final_effect,
      evidence->provider,
      evidence->resource,
      evidence->authority_context,
      decision->stack.evidence_profile,
      evidence->review_trace_required ? "true" : "false",
      evidence->retention_required ? "true" : "false",
      evidence->provenance_required ? "true" : "false",
      evidence->approval_chain_required ? "true" : "false",
      evidence->dependency_chain_required ? "true" : "false",
      evidence->lawful_basis_required ? "true" : "false",
      evidence->oversight_trace_required ? "true" : "false");
}
