#include "../internal.h"

int yai_governance_apply_precedence(yai_governance_effect_t *effect) {
  if (!effect) return -1;
  /* Deterministic precedence hardening for the runtime handshake. */
  if (*effect == YAI_GOVERNANCE_EFFECT_REQUIRE_JUSTIFICATION) {
    *effect = YAI_GOVERNANCE_EFFECT_REVIEW_REQUIRED;
  }
  return 0;
}
