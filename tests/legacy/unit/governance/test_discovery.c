#include <stdio.h>
#include <string.h>

#include <yai/policy/governance/classifier.h>
#include <yai/policy/governance/discovery.h>

int main(void) {
  yai_governance_classification_ctx_t ctx;
  yai_governance_discovery_result_t res;

  if (yai_governance_classify_event("ws", "{\"command\":\"curl\",\"resource\":\"endpoint\"}", &ctx) != 0) return 1;
  if (yai_governance_discover_domain(&ctx, &res) != 0) return 1;
  if (strcmp(res.domain_id, "D1-digital") != 0) return 1;
  if (strcmp(res.family_id, "digital") != 0) return 1;
  if (strcmp(res.specialization_id, "network-egress") != 0) return 1;
  if (res.family_candidate_count < 1) return 1;
  if (strcmp(res.family_candidates[0], "digital") != 0) return 1;
  if (res.specialization_candidate_count < 1) return 1;

  if (yai_governance_classify_event("ws", "{\"command\":\"experiment.run\",\"params_hash\":\"abc\",\"dataset\":\"d1\"}", &ctx) != 0) return 1;
  if (yai_governance_discover_domain(&ctx, &res) != 0) return 1;
  if (strcmp(res.domain_id, "D8-scientific") != 0) return 1;
  if (strcmp(res.family_id, "scientific") != 0) return 1;
  if (strcmp(res.specialization_id, "parameter-governance") != 0) return 1;

  if (yai_governance_classify_event("ws", "{\"command\":\"payment.authorize\",\"resource\":\"ledger\",\"provider\":\"payment-gateway\"}", &ctx) != 0) return 1;
  if (yai_governance_discover_domain(&ctx, &res) != 0) return 1;
  if (strcmp(res.domain_id, "D5-economic") != 0) return 1;
  if (strcmp(res.family_id, "economic") != 0) return 1;
  if (strcmp(res.specialization_id, "payments") != 0) return 1;
  if (strcmp(res.family_candidates[0], "economic") != 0) return 1;

  puts("discovery: ok");
  return 0;
}
