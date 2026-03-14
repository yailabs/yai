#pragma once

#include <yai/pol/governance/classifier.h>

typedef struct yai_governance_discovery_result {
  char domain_id[64];
  char family_id[64];
  char specialization_id[96];
  char family_candidates[4][64];
  double family_candidate_scores[4];
  int family_candidate_count;
  char specialization_candidates[6][96];
  int specialization_candidate_count;
  double confidence;
  int ambiguous;
  char rationale[192];
} yai_governance_discovery_result_t;

int yai_governance_discover_domain(const yai_governance_classification_ctx_t *ctx,
                            yai_governance_discovery_result_t *out);
