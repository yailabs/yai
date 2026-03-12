#include <stdio.h>
#include <string.h>

#include <yai/policy/governance/resolver.h>
#include <yai/policy/governance/policy_effects.h>

int main(void) {
  yai_governance_resolution_output_t out;
  char err[256] = {0};

  if (yai_governance_resolve_control_call("ws-a", "{\"command\":\"curl\",\"resource\":\"endpoint\"}", "trace-a", &out, err, sizeof(err)) != 0) {
    fprintf(stderr, "resolution failed: %s\n", err);
    return 1;
  }
  if (out.decision.final_effect != YAI_GOVERNANCE_EFFECT_DENY && out.decision.final_effect != YAI_GOVERNANCE_EFFECT_REVIEW_REQUIRED) return 1;
  if (out.decision.family_id[0] == '\0' || out.decision.specialization_id[0] == '\0') return 1;
  if (strstr(out.trace_json, "\"routing_mode\":\"family-specialization\"") == NULL) return 1;
  if (strstr(out.trace_json, "\"family_candidates\"") == NULL) return 1;

  if (yai_governance_resolve_control_call("ws-b", "{\"command\":\"experiment.run\",\"params_hash\":\"ok\",\"dataset\":\"d\"}", "trace-b", &out, err, sizeof(err)) != 0) return 1;
  if (strcmp(out.decision.domain_id, "D8-scientific") != 0) return 1;
  if (strcmp(out.decision.family_id, "scientific") != 0) return 1;
  if (strcmp(out.decision.specialization_id, "parameter-governance") != 0) return 1;
  if (out.decision.stack.regulatory_overlay_count < 1) return 1;
  if (out.decision.stack.contextual_overlay_count < 1) return 1;
  if (out.decision.stack.authority_contributor_count < 1) return 1;
  if (out.decision.stack.evidence_contributor_count < 2) return 1;
  if (out.decision.authority_requirement_count < 1) return 1;
  if (out.decision.evidence_requirement_count < 2) return 1;

  puts("resolution: ok");
  return 0;
}
