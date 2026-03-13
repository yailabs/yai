#include <stdio.h>
#include <string.h>

#include <yai/policy/governance/resolver.h>
#include <yai/policy/governance/policy_effects.h>

int main(void) {
  yai_governance_resolution_output_t out;
  char err[256] = {0};

  if (yai_governance_resolve_control_call("ws-d1", "{\"command\":\"curl\",\"resource\":\"endpoint\"}", "trace-d1", &out, err, sizeof(err)) != 0) {
    fprintf(stderr, "d1 resolution error: %s\n", err);
    return 1;
  }

  if (strcmp(out.decision.domain_id, "D1-digital") != 0) return 1;
  if (strcmp(out.decision.family_id, "digital") != 0) return 1;
  if (strcmp(out.decision.specialization_id, "network-egress") != 0) return 1;
  if (strstr(out.trace_json, "\"family_candidates\"") == NULL) return 1;
  if (out.decision.final_effect == YAI_GOVERNANCE_EFFECT_UNKNOWN) return 1;

  puts("integration_d1_resolution: ok");
  return 0;
}
