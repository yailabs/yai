#pragma once

#include <stddef.h>
#include <stdint.h>

#include <yai/pol/policy_state.h>
#include <yai/pol/grants_state.h>

#define YAI_RUNTIME_CONTAINMENT_MODE_NORMAL "normal"
#define YAI_RUNTIME_CONTAINMENT_MODE_RESTRICTED "restricted"
#define YAI_RUNTIME_CONTAINMENT_MODE_OBSERVE_ONLY "observe_only"

typedef struct yai_runtime_containment_state {
  char workspace_id[64];
  char mode[32];
  char isolation_state[48];
  char last_reason[128];
  int policy_active;
  size_t valid_grants;
  int64_t updated_at_epoch;
} yai_runtime_containment_state_t;

int yai_runtime_containment_state_init(yai_runtime_containment_state_t *state, const char *workspace_id);
int yai_runtime_containment_state_apply(yai_runtime_containment_state_t *state,
                                        const yai_runtime_policy_state_t *policy,
                                        const yai_runtime_grants_state_t *grants,
                                        const char *reason,
                                        int64_t now_epoch);
int yai_runtime_containment_state_allows(const yai_runtime_containment_state_t *state,
                                         const char *action_class);
int yai_runtime_containment_state_json(const yai_runtime_containment_state_t *state,
                                       char *out,
                                       size_t out_cap);

int yai_runtime_containment_start(const char *workspace_id, char *err, size_t err_cap);
int yai_runtime_containment_evaluate(const char *reason, int64_t now_epoch);
int yai_runtime_containment_stop(void);
const yai_runtime_containment_state_t *yai_runtime_containment_current(void);
