#include <stdio.h>
#include <string.h>

#include <yai/policy/governance/resolver.h>
#include <yai/policy/governance/policy_effects.h>

static int has_req(char reqs[][64], int n, const char *id) {
  for (int i = 0; i < n; ++i) {
    if (strcmp(reqs[i], id) == 0) return 1;
  }
  return 0;
}

int main(void) {
  yai_governance_resolution_output_t out;
  char err[256] = {0};

  if (yai_governance_resolve_control_call(
          "ws-ov-ae",
          "{\"command\":\"payment.transfer.personal-data\",\"resource\":\"personal_ledger\",\"provider\":\"payment-gateway\",\"contract\":\"true\"}",
          "trace-ov-ae",
          &out,
          err,
          sizeof(err)) != 0) {
    fprintf(stderr, "resolve failed: %s\n", err);
    return 1;
  }

  if (strcmp(out.decision.family_id, "economic") != 0) return 1;
  if (out.decision.stack.regulatory_overlay_count < 1) return 1;
  if (out.decision.stack.sector_overlay_count < 1) return 1;
  if (out.decision.stack.contextual_overlay_count < 1) return 1;

  if (!has_req(out.decision.authority_requirements, out.decision.authority_requirement_count, "dual_control_required")) return 1;
  if (!has_req(out.decision.evidence_requirements, out.decision.evidence_requirement_count, "approval_chain_required")) return 1;
  if (!has_req(out.decision.evidence_requirements, out.decision.evidence_requirement_count, "retention_required")) return 1;

  if (out.evidence.approval_chain_required != 1) return 1;
  if (out.evidence.retention_required != 1) return 1;
  if (out.evidence.provenance_required != 1) return 1;

  if (!strstr(out.trace_json, "regulatory_overlay_count")) return 1;
  if (!strstr(out.trace_json, "authority_contributor_count")) return 1;

  puts("overlay_authority_evidence: ok");
  return 0;
}
