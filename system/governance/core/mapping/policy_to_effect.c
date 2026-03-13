#include "../internal.h"

int yai_governance_map_policy_to_effect(yai_governance_effect_t in_effect, yai_governance_effect_t *out_effect) {
  if (!out_effect) return -1;
  switch (in_effect) {
    case YAI_GOVERNANCE_EFFECT_DENY:
    case YAI_GOVERNANCE_EFFECT_QUARANTINE:
    case YAI_GOVERNANCE_EFFECT_REVIEW_REQUIRED:
    case YAI_GOVERNANCE_EFFECT_ALLOW:
    case YAI_GOVERNANCE_EFFECT_DEGRADE:
      *out_effect = in_effect;
      break;
    case YAI_GOVERNANCE_EFFECT_REQUIRE_JUSTIFICATION:
      *out_effect = YAI_GOVERNANCE_EFFECT_REVIEW_REQUIRED;
      break;
    case YAI_GOVERNANCE_EFFECT_UNKNOWN:
    default:
      *out_effect = YAI_GOVERNANCE_EFFECT_REVIEW_REQUIRED;
      break;
  }
  return 0;
}
