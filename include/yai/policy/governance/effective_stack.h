#pragma once

#include <yai/policy/governance/discovery.h>

#define YAI_GOVERNANCE_RULE_MAX 12
#define YAI_GOVERNANCE_COMPLIANCE_MAX 8
#define YAI_GOVERNANCE_CONTRIBUTOR_MAX 12

typedef struct yai_governance_effective_stack {
  char stack_id[64];
  char domain_id[64];
  char family_id[64];
  char specialization_id[96];
  char applied_rules[YAI_GOVERNANCE_RULE_MAX][96];
  int applied_rule_count;
  char skipped_rules[YAI_GOVERNANCE_RULE_MAX][96];
  int skipped_rule_count;
  char compliance_layers[YAI_GOVERNANCE_COMPLIANCE_MAX][64];
  int compliance_count;
  char regulatory_overlays[YAI_GOVERNANCE_COMPLIANCE_MAX][64];
  int regulatory_overlay_count;
  char sector_overlays[YAI_GOVERNANCE_COMPLIANCE_MAX][64];
  int sector_overlay_count;
  char contextual_overlays[YAI_GOVERNANCE_COMPLIANCE_MAX][64];
  int contextual_overlay_count;
  char overlay_layers[YAI_GOVERNANCE_COMPLIANCE_MAX][64];
  int overlay_count;
  char authority_contributors[YAI_GOVERNANCE_CONTRIBUTOR_MAX][64];
  int authority_contributor_count;
  char evidence_contributors[YAI_GOVERNANCE_CONTRIBUTOR_MAX][64];
  int evidence_contributor_count;
  char authority_profile[128];
  char evidence_profile[128];
  char precedence_trace[192];
} yai_governance_effective_stack_t;
