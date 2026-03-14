#pragma once

#include <stddef.h>
#include <stdint.h>

#include <yai/vault.h>

typedef enum yai_authority_decision {
  YAI_AUTHORITY_ALLOW = 0,
  YAI_AUTHORITY_REVIEW_REQUIRED = 1,
  YAI_AUTHORITY_DENY = 2
} yai_authority_decision_t;

typedef struct yai_authority_evaluation {
  yai_authority_decision_t decision;
  uint32_t command_id;
  uint32_t command_class;
  int operator_armed;
  int authority_lock;
  char reason[96];
} yai_authority_evaluation_t;

int yai_authority_command_gate(const yai_vault_t *vault,
                               uint32_t command_id,
                               uint16_t role,
                               uint16_t arming,
                               yai_authority_evaluation_t *out,
                               char *err,
                               size_t err_cap);

int yai_authority_policy_gate(const char *workspace_id,
                              const char *effect_name,
                              const char *authority_profile,
                              const char *authority_context,
                              yai_authority_evaluation_t *out,
                              char *err,
                              size_t err_cap);
