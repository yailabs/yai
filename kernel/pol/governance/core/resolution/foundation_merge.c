#include "../internal.h"

int yai_governance_foundation_merge_apply(yai_governance_effective_stack_t *stack,
                                   yai_governance_effect_t *effect) {
  if (!stack || !effect) return -1;
  if (stack->authority_contributor_count == 0) {
    (void)yai_governance_safe_snprintf(stack->authority_contributors[0],
                                sizeof(stack->authority_contributors[0]),
                                "%s",
                                "baseline_authority");
    stack->authority_contributor_count = 1;
  }
  if (stack->evidence_contributor_count == 0) {
    (void)yai_governance_safe_snprintf(stack->evidence_contributors[0],
                                sizeof(stack->evidence_contributors[0]),
                                "%s",
                                "resolution_trace");
    stack->evidence_contributor_count = 1;
  }
  if (stack->authority_profile[0] == '\0') {
    (void)yai_governance_safe_snprintf(stack->authority_profile,
                                sizeof(stack->authority_profile),
                                "%s",
                                stack->authority_contributors[0]);
  }
  if (stack->evidence_profile[0] == '\0') {
    (void)yai_governance_safe_snprintf(stack->evidence_profile,
                                sizeof(stack->evidence_profile),
                                "%s",
                                stack->evidence_contributors[0]);
  }
  if (*effect == YAI_GOVERNANCE_EFFECT_UNKNOWN) {
    *effect = YAI_GOVERNANCE_EFFECT_REVIEW_REQUIRED;
  }
  return 0;
}
