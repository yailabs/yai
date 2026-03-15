#include "../internal.h"

int yai_governance_build_trace_json(const yai_governance_classification_ctx_t *ctx,
                             const yai_governance_discovery_result_t *disc,
                             const yai_governance_decision_t *decision,
                             char *out,
                             size_t out_cap) {
  const char *f1 = "";
  const char *f2 = "";
  const char *f3 = "";
  const char *s1 = "";
  const char *s2 = "";
  const char *s3 = "";
  const char *r0 = "";
  const char *sec0 = "";
  const char *ctx0 = "";
  if (!ctx || !disc || !decision || !out || out_cap == 0) return -1;
  if (disc->family_candidate_count > 0) f1 = disc->family_candidates[0];
  if (disc->family_candidate_count > 1) f2 = disc->family_candidates[1];
  if (disc->family_candidate_count > 2) f3 = disc->family_candidates[2];
  if (disc->specialization_candidate_count > 0) s1 = disc->specialization_candidates[0];
  if (disc->specialization_candidate_count > 1) s2 = disc->specialization_candidates[1];
  if (disc->specialization_candidate_count > 2) s3 = disc->specialization_candidates[2];
  if (decision->stack.regulatory_overlay_count > 0) r0 = decision->stack.regulatory_overlays[0];
  if (decision->stack.sector_overlay_count > 0) sec0 = decision->stack.sector_overlays[0];
  if (decision->stack.contextual_overlay_count > 0) ctx0 = decision->stack.contextual_overlays[0];
  return yai_governance_safe_snprintf(
      out,
      out_cap,
      "{\"type\":\"yai.governance.resolution_trace.v1\",\"routing_mode\":\"family-specialization\",\"action\":\"%s\",\"provider\":\"%s\",\"resource\":\"%s\",\"protocol\":\"%s\",\"matched_domain\":\"%s\",\"compat_domain_id\":\"%s\",\"matched_family\":\"%s\",\"matched_specialization\":\"%s\",\"family_candidates\":[\"%s\",\"%s\",\"%s\"],\"specialization_candidates\":[\"%s\",\"%s\",\"%s\"],\"confidence\":%.3f,\"ambiguous\":%s,\"regulatory_overlay_count\":%d,\"sector_overlay_count\":%d,\"contextual_overlay_count\":%d,\"regulatory_overlay_head\":\"%s\",\"sector_overlay_head\":\"%s\",\"contextual_overlay_head\":\"%s\",\"authority_contributor_count\":%d,\"evidence_contributor_count\":%d,\"final_effect\":\"%s\",\"rationale\":\"%s\",\"precedence_trace\":\"%s\",\"authority_profile\":\"%s\",\"evidence_profile\":\"%s\",\"scientific_flags\":{\"params_hash\":%s,\"locked_parameters\":%s,\"repro_context\":%s,\"dataset_ref\":%s,\"publication_intent\":%s,\"result_ref\":%s},\"digital_flags\":{\"retrieve_intent\":%s,\"egress_intent\":%s,\"commentary_intent\":%s,\"distribution_intent\":%s,\"sink_ref\":%s,\"sink_trusted\":%s,\"sink_external\":%s}}",
      ctx->action,
      ctx->provider,
      ctx->resource,
      ctx->protocol,
      disc->domain_id,
      disc->domain_id,
      disc->family_id,
      disc->specialization_id,
      f1, f2, f3,
      s1, s2, s3,
      disc->confidence,
      disc->ambiguous ? "true" : "false",
      decision->stack.regulatory_overlay_count,
      decision->stack.sector_overlay_count,
      decision->stack.contextual_overlay_count,
      r0, sec0, ctx0,
      decision->stack.authority_contributor_count,
      decision->stack.evidence_contributor_count,
      yai_governance_effect_name(decision->final_effect),
      decision->rationale,
      decision->stack.precedence_trace,
      decision->stack.authority_profile,
      decision->stack.evidence_profile,
      ctx->has_params_hash ? "true" : "false",
      ctx->has_locked_parameters ? "true" : "false",
      ctx->has_repro_context ? "true" : "false",
      ctx->has_dataset_ref ? "true" : "false",
      ctx->has_publication_intent ? "true" : "false",
      ctx->has_result_ref ? "true" : "false",
      ctx->has_retrieve_intent ? "true" : "false",
      ctx->has_egress_intent ? "true" : "false",
      ctx->has_commentary_intent ? "true" : "false",
      ctx->has_distribution_intent ? "true" : "false",
      ctx->has_sink_ref ? "true" : "false",
      ctx->sink_trusted ? "true" : "false",
      ctx->sink_external ? "true" : "false");
}
