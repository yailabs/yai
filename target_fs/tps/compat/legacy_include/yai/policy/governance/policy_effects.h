#pragma once

typedef enum yai_governance_effect {
  YAI_GOVERNANCE_EFFECT_ALLOW = 0,
  YAI_GOVERNANCE_EFFECT_DENY,
  YAI_GOVERNANCE_EFFECT_QUARANTINE,
  YAI_GOVERNANCE_EFFECT_REVIEW_REQUIRED,
  YAI_GOVERNANCE_EFFECT_DEGRADE,
  YAI_GOVERNANCE_EFFECT_REQUIRE_JUSTIFICATION,
  YAI_GOVERNANCE_EFFECT_UNKNOWN
} yai_governance_effect_t;

const char *yai_governance_effect_name(yai_governance_effect_t effect);
