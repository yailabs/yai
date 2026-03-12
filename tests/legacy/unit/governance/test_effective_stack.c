#include <stdio.h>

#include <yai/policy/governance/resolver.h>

int main(void) {
  yai_governance_resolution_output_t out;
  char err[256] = {0};

  if (yai_governance_resolve_control_call("ws", "{\"command\":\"github.issues.comment.create\",\"contract\":\"true\"}", "trace-s", &out, err, sizeof(err)) != 0) return 1;
  if (out.decision.stack.applied_rule_count <= 0) return 1;
  if (out.decision.stack.compliance_count <= 0) return 1;
  if (out.decision.stack.overlay_count <= 0) return 1;
  if (out.decision.stack.regulatory_overlay_count <= 0) return 1;
  if (out.decision.stack.contextual_overlay_count <= 0) return 1;
  if (out.decision.stack.authority_contributor_count <= 0) return 1;
  if (out.decision.stack.evidence_contributor_count <= 0) return 1;
  if (out.decision.stack.authority_profile[0] == '\0') return 1;
  if (out.decision.stack.evidence_profile[0] == '\0') return 1;

  puts("effective_stack: ok");
  return 0;
}
