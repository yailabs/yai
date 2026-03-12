#include <stdio.h>
#include <string.h>

#include <yai/policy/governance/resolver.h>
#include <yai/policy/governance/policy_effects.h>

int main(void) {
  yai_governance_resolution_output_t out;
  char err[256] = {0};

  if (yai_governance_resolve_control_call(
          "ws-econ",
          "{\"command\":\"payment.authorize\",\"resource\":\"ledger\",\"provider\":\"payment-gateway\",\"contract\":\"true\"}",
          "trace-econ",
          &out,
          err,
          sizeof(err)) != 0) {
    fprintf(stderr, "economic resolution error: %s\n", err);
    return 1;
  }

  if (strcmp(out.decision.domain_id, "D5-economic") != 0) {
    fprintf(stderr, "economic resolution unexpected domain: %s\n", out.decision.domain_id);
    return 1;
  }
  if (strcmp(out.decision.family_id, "economic") != 0) return 1;
  if (strcmp(out.decision.specialization_id, "payments") != 0) return 1;
  if (out.decision.stack.sector_overlay_count < 1) return 1;
  if (out.decision.stack.contextual_overlay_count < 1) return 1;
  if (out.decision.stack.authority_contributor_count < 1) return 1;
  if (out.decision.stack.evidence_contributor_count < 2) return 1;

  if (out.decision.final_effect == YAI_GOVERNANCE_EFFECT_UNKNOWN) return 1;
  if (strstr(out.trace_json, "D5-economic") == NULL) return 1;
  if (strstr(out.trace_json, "\"family_candidates\"") == NULL) return 1;

  puts("integration_economic_resolution: ok");
  return 0;
}
