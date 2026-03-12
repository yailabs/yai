#pragma once

#include <yai/policy/governance/effective_stack.h>
#include <yai/policy/governance/policy_effects.h>

typedef struct yai_governance_decision {
  char decision_id[64];
  char domain_id[64];
  char family_id[64];
  char specialization_id[96];
  yai_governance_effect_t final_effect;
  char rationale[192];
  yai_governance_effective_stack_t stack;
  char evidence_requirements[8][64];
  int evidence_requirement_count;
  char authority_requirements[8][64];
  int authority_requirement_count;
} yai_governance_decision_t;
