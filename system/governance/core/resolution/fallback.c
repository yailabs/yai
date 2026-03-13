#include "../internal.h"

int yai_governance_apply_fallback(const yai_governance_discovery_result_t *discovery, yai_governance_effect_t *effect) {
  if (!discovery || !effect) return -1;
  if (discovery->ambiguous && *effect == YAI_GOVERNANCE_EFFECT_ALLOW) {
    *effect = YAI_GOVERNANCE_EFFECT_REVIEW_REQUIRED;
  }
  return 0;
}
