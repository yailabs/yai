#include <stdio.h>
#include <string.h>

#include <yai/policy/governance/resolver.h>

int main(void) {
  yai_governance_resolution_output_t out;
  char err[256] = {0};

  if (yai_governance_resolve_control_call("ws-d8", "{\"command\":\"experiment.run\",\"params_hash\":\"abc\",\"dataset\":\"d1\"}", "trace-d8", &out, err, sizeof(err)) != 0) {
    fprintf(stderr, "d8 resolution error: %s\n", err);
    return 1;
  }

  if (strcmp(out.decision.domain_id, "D8-scientific") != 0) return 1;
  if (strcmp(out.decision.family_id, "scientific") != 0) return 1;
  if (strcmp(out.decision.specialization_id, "parameter-governance") != 0) return 1;
  if (strstr(out.trace_json, "D8-scientific") == NULL) return 1;
  if (strstr(out.trace_json, "\"specialization_candidates\"") == NULL) return 1;

  puts("integration_d8_resolution: ok");
  return 0;
}
