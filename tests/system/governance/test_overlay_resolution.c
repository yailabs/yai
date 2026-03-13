#include <stdio.h>
#include <string.h>

#include <yai/policy/governance/policy_effects.h>
#include <yai/policy/governance/resolver.h>

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
          "ws-ov1",
          "{\"command\":\"payment.authorize\",\"resource\":\"ledger\",\"provider\":\"untrusted-gateway\",\"contract\":\"true\"}",
          "trace-ov1",
          &out,
          err,
          sizeof(err)) != 0) {
    fprintf(stderr, "ov1 resolve failed: %s\n", err);
    return 1;
  }
  if (strcmp(out.decision.domain_id, "D5-economic") != 0) return 1;
  if (strcmp(out.decision.family_id, "economic") != 0) return 1;
  if (strcmp(out.decision.specialization_id, "payments") != 0) return 1;
  if (out.decision.final_effect != YAI_GOVERNANCE_EFFECT_QUARANTINE) return 1;
  if (out.decision.stack.sector_overlay_count < 1 || out.decision.stack.regulatory_overlay_count < 1) return 1;
  if (!has_req(out.decision.authority_requirements, out.decision.authority_requirement_count, "escalation_required")) return 1;
  if (!has_req(out.decision.evidence_requirements, out.decision.evidence_requirement_count, "dependency_chain_ref")) return 1;

  if (yai_governance_resolve_control_call(
          "ws-ov2",
          "{\"command\":\"github.publish.personal-data\",\"resource\":\"personal_profile\",\"provider\":\"github-api\",\"contract\":\"true\"}",
          "trace-ov2",
          &out,
          err,
          sizeof(err)) != 0) {
    fprintf(stderr, "ov2 resolve failed: %s\n", err);
    return 1;
  }
  if (strcmp(out.decision.domain_id, "D1-digital") != 0) return 1;
  if (strcmp(out.decision.family_id, "digital") != 0) return 1;
  if (strcmp(out.decision.specialization_id, "remote-publication") != 0) return 1;
  if (out.decision.final_effect != YAI_GOVERNANCE_EFFECT_REVIEW_REQUIRED) return 1;
  if (out.decision.stack.regulatory_overlay_count < 1) return 1;
  if (!has_req(out.decision.evidence_requirements, out.decision.evidence_requirement_count, "lawful_basis_trace")) return 1;
  if (out.evidence.lawful_basis_required != 1) return 1;

  if (yai_governance_resolve_control_call(
          "ws-ov3",
          "{\"command\":\"experiment.run.high-risk\",\"params_hash\":\"abc\",\"dataset\":\"d1\",\"provider\":\"experiment-runner\"}",
          "trace-ov3",
          &out,
          err,
          sizeof(err)) != 0) {
    fprintf(stderr, "ov3 resolve failed: %s\n", err);
    return 1;
  }
  if (strcmp(out.decision.domain_id, "D8-scientific") != 0) return 1;
  if (strcmp(out.decision.family_id, "scientific") != 0) return 1;
  if (strcmp(out.decision.specialization_id, "parameter-governance") != 0) return 1;
  if (out.decision.final_effect != YAI_GOVERNANCE_EFFECT_REVIEW_REQUIRED) return 1;
  if (!has_req(out.decision.authority_requirements, out.decision.authority_requirement_count, "human_oversight_required")) return 1;
  if (!has_req(out.decision.evidence_requirements, out.decision.evidence_requirement_count, "oversight_trace_required")) return 1;
  if (out.evidence.oversight_trace_required != 1) return 1;
  if (!strstr(out.trace_json, "regulatory_overlay_count")) return 1;

  puts("integration_overlay_resolution: ok");
  return 0;
}
