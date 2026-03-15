#include "../internal.h"

int yai_governance_compliance_merge_apply(yai_governance_effective_stack_t *stack,
                                   yai_governance_effect_t *effect) {
  if (!stack || !effect) return -1;
  if (stack->compliance_count == 0) {
    (void)yai_governance_safe_snprintf(stack->compliance_layers[stack->compliance_count],
                                sizeof(stack->compliance_layers[stack->compliance_count]),
                                "%s",
                                "baseline.compliance");
    stack->compliance_count++;
  }
  if (stack->regulatory_overlay_count > 0 && *effect == YAI_GOVERNANCE_EFFECT_ALLOW) {
    *effect = YAI_GOVERNANCE_EFFECT_REVIEW_REQUIRED;
  }
  return 0;
}
