#include <yai/pol/governance/resolver.h>

#include "../internal.h"

#include <string.h>
#include <time.h>

int yai_governance_apply_precedence(yai_governance_effect_t *effect);
int yai_governance_apply_fallback(const yai_governance_discovery_result_t *discovery, yai_governance_effect_t *effect);
int yai_governance_resolve_conflicts(yai_governance_effect_t *effect);
int yai_governance_effective_stack_finalize(yai_governance_effective_stack_t *stack);
int yai_governance_map_policy_to_effect(yai_governance_effect_t in_effect, yai_governance_effect_t *out_effect);
int yai_governance_decision_to_audit_blob(const yai_governance_decision_t *decision, char *out, size_t out_cap);

static void append_requirement(char arr[][64], int *count, int cap, const char *id) {
  int i;
  if (!arr || !count || !id || id[0] == '\0') return;
  for (i = 0; i < *count; ++i) {
    if (strcmp(arr[i], id) == 0) return;
  }
  if (*count >= cap) return;
  (void)yai_governance_safe_snprintf(arr[*count], sizeof(arr[0]), "%s", id);
  (*count)++;
}

int yai_governance_resolve_control_call(const char *ws_id,
                                 const char *payload,
                                 const char *trace_id,
                                 yai_governance_resolution_output_t *out,
                                 char *err,
                                 size_t err_cap) {
  yai_governance_runtime_t runtime;
  yai_governance_classification_ctx_t ctx;
  yai_governance_discovery_result_t discovery;
  yai_governance_effect_t base_effect = YAI_GOVERNANCE_EFFECT_UNKNOWN;
  yai_governance_effect_t final_effect = YAI_GOVERNANCE_EFFECT_UNKNOWN;
  char rationale[192] = {0};

  if (!payload || !out) return -1;
  memset(out, 0, sizeof(*out));

  if (yai_governance_load_runtime(&runtime, err, err_cap) != 0) {
    return -1;
  }
  if (yai_governance_classify_event(ws_id, payload, &ctx) != 0) {
    if (err && err_cap) (void)yai_governance_safe_snprintf(err, err_cap, "%s", "classification_failed");
    return -1;
  }
  if (yai_governance_discover_domain(&ctx, &discovery) != 0) {
    if (err && err_cap) (void)yai_governance_safe_snprintf(err, err_cap, "%s", "discovery_failed");
    return -1;
  }
  if (yai_governance_domain_merge_apply(&discovery, err, err_cap) != 0) {
    if (err && err_cap && !err[0]) (void)yai_governance_safe_snprintf(err, err_cap, "%s", "domain_merge_failed");
    return -1;
  }

  if (yai_governance_stack_build(&runtime, &discovery, &ctx, &out->decision.stack, &base_effect, rationale, sizeof(rationale)) != 0) {
    if (err && err_cap) (void)yai_governance_safe_snprintf(err, err_cap, "%s", "stack_build_failed");
    return -1;
  }
  (void)yai_governance_foundation_merge_apply(&out->decision.stack, &base_effect);
  (void)yai_governance_overlay_merge_apply(&out->decision.stack);
  (void)yai_governance_compliance_merge_apply(&out->decision.stack, &base_effect);

  if (yai_governance_map_policy_to_effect(base_effect, &final_effect) != 0) {
    if (err && err_cap) (void)yai_governance_safe_snprintf(err, err_cap, "%s", "effect_mapping_failed");
    return -1;
  }

  (void)yai_governance_apply_precedence(&final_effect);
  (void)yai_governance_apply_fallback(&discovery, &final_effect);
  (void)yai_governance_resolve_conflicts(&final_effect);
  (void)yai_governance_effective_stack_finalize(&out->decision.stack);

  (void)yai_governance_safe_snprintf(out->decision.decision_id, sizeof(out->decision.decision_id), "dec-%ld", (long)time(NULL));
  (void)yai_governance_safe_snprintf(out->decision.domain_id, sizeof(out->decision.domain_id), "%s", discovery.domain_id);
  (void)yai_governance_safe_snprintf(out->decision.family_id, sizeof(out->decision.family_id), "%s", discovery.family_id);
  (void)yai_governance_safe_snprintf(out->decision.specialization_id, sizeof(out->decision.specialization_id), "%s", discovery.specialization_id);
  out->decision.final_effect = final_effect;
  (void)yai_governance_safe_snprintf(out->decision.rationale, sizeof(out->decision.rationale), "%s", rationale[0] ? rationale : discovery.rationale);

  out->decision.evidence_requirement_count = 0;
  append_requirement(out->decision.evidence_requirements,
                     &out->decision.evidence_requirement_count,
                     8,
                     "resolution_trace");
  append_requirement(out->decision.evidence_requirements,
                     &out->decision.evidence_requirement_count,
                     8,
                     "decision_record");
  for (int i = 0; i < out->decision.stack.evidence_contributor_count; ++i) {
    append_requirement(out->decision.evidence_requirements,
                       &out->decision.evidence_requirement_count,
                       8,
                       out->decision.stack.evidence_contributors[i]);
  }

  out->decision.authority_requirement_count = 0;
  append_requirement(out->decision.authority_requirements,
                     &out->decision.authority_requirement_count,
                     8,
                     "baseline_authority");
  for (int i = 0; i < out->decision.stack.authority_contributor_count; ++i) {
    append_requirement(out->decision.authority_requirements,
                       &out->decision.authority_requirement_count,
                       8,
                       out->decision.stack.authority_contributors[i]);
  }

  if (yai_governance_decision_to_evidence(&out->decision,
                                   (trace_id && trace_id[0]) ? trace_id : "trace-missing",
                                   ctx.provider,
                                   ctx.resource,
                                   ctx.has_authority_contract ? "contracted" : "missing_contract",
                                   &out->evidence) != 0) {
    if (err && err_cap) (void)yai_governance_safe_snprintf(err, err_cap, "%s", "evidence_mapping_failed");
    return -1;
  }

  if (yai_governance_build_trace_json(&ctx, &discovery, &out->decision, out->trace_json, sizeof(out->trace_json)) != 0) {
    if (err && err_cap) (void)yai_governance_safe_snprintf(err, err_cap, "%s", "trace_build_failed");
    return -1;
  }

  return 0;
}
