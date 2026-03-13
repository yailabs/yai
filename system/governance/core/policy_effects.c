#include <yai/policy/governance/policy_effects.h>

const char *yai_governance_effect_name(yai_governance_effect_t effect) {
  switch (effect) {
    case YAI_GOVERNANCE_EFFECT_ALLOW: return "allow";
    case YAI_GOVERNANCE_EFFECT_DENY: return "deny";
    case YAI_GOVERNANCE_EFFECT_QUARANTINE: return "quarantine";
    case YAI_GOVERNANCE_EFFECT_REVIEW_REQUIRED: return "review_required";
    case YAI_GOVERNANCE_EFFECT_DEGRADE: return "degrade";
    case YAI_GOVERNANCE_EFFECT_REQUIRE_JUSTIFICATION: return "require_justification";
    default: return "unknown";
  }
}
