#pragma once

#include <stddef.h>
#include <stdint.h>

#define YAI_RUNTIME_POLICY_ID_MAX 128
#define YAI_RUNTIME_POLICY_WORKSPACE_MAX 64
#define YAI_RUNTIME_POLICY_MODE_MAX 32

typedef struct yai_runtime_policy_state {
  char workspace_id[YAI_RUNTIME_POLICY_WORKSPACE_MAX];
  char active_snapshot_id[YAI_RUNTIME_POLICY_ID_MAX];
  char mode[YAI_RUNTIME_POLICY_MODE_MAX];
  uint32_t generation;
  uint32_t effect_count;
  int active;
  int64_t applied_at_epoch;
  char last_reason[96];
} yai_runtime_policy_state_t;

int yai_runtime_policy_state_init(yai_runtime_policy_state_t *state, const char *workspace_id);
int yai_runtime_policy_state_apply(yai_runtime_policy_state_t *state,
                                   const char *snapshot_id,
                                   uint32_t effect_count,
                                   int64_t applied_at_epoch,
                                   const char *reason);
int yai_runtime_policy_state_clear(yai_runtime_policy_state_t *state, const char *reason);
int yai_runtime_policy_state_json(const yai_runtime_policy_state_t *state,
                                  char *out,
                                  size_t out_cap);

int yai_runtime_policy_start(const char *workspace_id, char *err, size_t err_cap);
int yai_runtime_policy_apply_effective(const char *snapshot_id,
                                       uint32_t effect_count,
                                       int64_t applied_at_epoch,
                                       const char *reason);
int yai_runtime_policy_stop(void);
const yai_runtime_policy_state_t *yai_runtime_policy_current(void);
