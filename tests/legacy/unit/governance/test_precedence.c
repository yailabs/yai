#include <stdio.h>
#include <string.h>

#include <yai/policy/governance/resolver.h>
#include <yai/policy/governance/policy_effects.h>

int main(void) {
  yai_governance_resolution_output_t out;
  char err[256] = {0};

  if (yai_governance_resolve_control_call("ws", "{\"command\":\"experiment.run\"}", "trace-p", &out, err, sizeof(err)) != 0) return 1;
  if (out.decision.final_effect != YAI_GOVERNANCE_EFFECT_DENY) {
    fprintf(stderr, "expected deny precedence for missing params lock\n");
    return 1;
  }

  puts("precedence: ok");
  return 0;
}
