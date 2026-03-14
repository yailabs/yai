#include "../internal.h"

int yai_governance_effective_stack_finalize(yai_governance_effective_stack_t *stack) {
  if (!stack) return -1;
  if (stack->authority_contributor_count == 0) {
    (void)yai_governance_safe_snprintf(stack->authority_contributors[0], sizeof(stack->authority_contributors[0]), "%s", "baseline_authority");
    stack->authority_contributor_count = 1;
  }
  if (stack->evidence_contributor_count == 0) {
    (void)yai_governance_safe_snprintf(stack->evidence_contributors[0], sizeof(stack->evidence_contributors[0]), "%s", "resolution_trace");
    stack->evidence_contributor_count = 1;
  }
  if (stack->authority_profile[0] == '\0') {
    (void)yai_governance_safe_snprintf(stack->authority_profile, sizeof(stack->authority_profile), "%s", "baseline_authority");
  }
  if (stack->evidence_profile[0] == '\0') {
    (void)yai_governance_safe_snprintf(stack->evidence_profile, sizeof(stack->evidence_profile), "%s", "resolution_trace");
  }
  return 0;
}
