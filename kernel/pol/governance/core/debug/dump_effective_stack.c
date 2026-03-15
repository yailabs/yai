#include "../internal.h"

int yai_governance_dump_effective_stack(const yai_governance_effective_stack_t *stack, char *out, size_t out_cap) {
  if (!stack || !out || out_cap == 0) return -1;
  return yai_governance_safe_snprintf(out,
                               out_cap,
                               "stack=%s family=%s domain=%s specialization=%s rules=%d compliance=%d overlays=%d (reg=%d sec=%d ctx=%d) auth_contrib=%d ev_contrib=%d authority=%s evidence=%s precedence=%s",
                               stack->stack_id,
                               stack->family_id,
                               stack->domain_id,
                               stack->specialization_id,
                               stack->applied_rule_count,
                               stack->compliance_count,
                               stack->overlay_count,
                               stack->regulatory_overlay_count,
                               stack->sector_overlay_count,
                               stack->contextual_overlay_count,
                               stack->authority_contributor_count,
                               stack->evidence_contributor_count,
                               stack->authority_profile,
                               stack->evidence_profile,
                               stack->precedence_trace);
}
