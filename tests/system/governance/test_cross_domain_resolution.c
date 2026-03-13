#include <stdio.h>
#include <string.h>

#include <yai/policy/governance/resolver.h>

int main(void) {
  yai_governance_resolution_output_t out;
  char err[256] = {0};

  if (yai_governance_resolve_control_call("ws-x", "{\"command\":\"github.issues.comment.create\",\"contract\":\"true\"}", "trace-x1", &out, err, sizeof(err)) != 0) return 1;
  if (strcmp(out.decision.domain_id, "D1-digital") != 0) return 1;
  if (strcmp(out.decision.family_id, "digital") != 0) return 1;
  if (strcmp(out.decision.specialization_id, "external-commentary") != 0) return 1;

  if (yai_governance_resolve_control_call("ws-x", "{\"command\":\"experiment.run\",\"black-box\":true}", "trace-x2", &out, err, sizeof(err)) != 0) return 1;
  if (strcmp(out.decision.domain_id, "D8-scientific") != 0) return 1;
  if (strcmp(out.decision.family_id, "scientific") != 0) return 1;
  if (strcmp(out.decision.specialization_id, "black-box-evaluation") != 0) return 1;

  puts("integration_cross_domain_resolution: ok");
  return 0;
}
