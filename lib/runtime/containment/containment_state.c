/* SPDX-License-Identifier: Apache-2.0 */

#include <yai/runtime/containment.h>

#include <stdio.h>
#include <string.h>
#include <time.h>

static yai_runtime_containment_state_t g_containment_state;
static int g_containment_started = 0;

static int copy_string(char *dst, size_t cap, const char *src)
{
  if (!dst || cap == 0 || !src || !src[0]) return -1;
  if (snprintf(dst, cap, "%s", src) >= (int)cap) return -1;
  return 0;
}

static int64_t now_epoch(void)
{
  return (int64_t)time(NULL);
}

int yai_runtime_containment_state_init(yai_runtime_containment_state_t *state, const char *workspace_id)
{
  if (!state || !workspace_id || !workspace_id[0]) return -1;
  memset(state, 0, sizeof(*state));
  if (copy_string(state->workspace_id, sizeof(state->workspace_id), workspace_id) != 0) return -1;
  if (copy_string(state->mode, sizeof(state->mode), YAI_RUNTIME_CONTAINMENT_MODE_OBSERVE_ONLY) != 0) return -1;
  if (copy_string(state->isolation_state, sizeof(state->isolation_state), "bootstrap") != 0) return -1;
  if (copy_string(state->last_reason, sizeof(state->last_reason), "containment_init") != 0) return -1;
  state->updated_at_epoch = now_epoch();
  return 0;
}

int yai_runtime_containment_state_apply(yai_runtime_containment_state_t *state,
                                        const yai_runtime_policy_state_t *policy,
                                        const yai_runtime_grants_state_t *grants,
                                        const char *reason,
                                        int64_t now_epoch_value)
{
  int64_t now = now_epoch_value > 0 ? now_epoch_value : now_epoch();
  if (!state) return -1;

  state->policy_active = (policy && policy->active) ? 1 : 0;
  state->valid_grants = grants ? grants->valid_count : 0u;

  if (!state->policy_active) {
    (void)copy_string(state->mode, sizeof(state->mode), YAI_RUNTIME_CONTAINMENT_MODE_OBSERVE_ONLY);
    (void)copy_string(state->isolation_state, sizeof(state->isolation_state), "policy_inactive");
  } else if (state->valid_grants == 0u) {
    (void)copy_string(state->mode, sizeof(state->mode), YAI_RUNTIME_CONTAINMENT_MODE_RESTRICTED);
    (void)copy_string(state->isolation_state, sizeof(state->isolation_state), "grants_missing");
  } else {
    (void)copy_string(state->mode, sizeof(state->mode), YAI_RUNTIME_CONTAINMENT_MODE_NORMAL);
    (void)copy_string(state->isolation_state, sizeof(state->isolation_state), "policy_and_grants_ok");
  }

  (void)copy_string(state->last_reason,
                    sizeof(state->last_reason),
                    (reason && reason[0]) ? reason : "containment_evaluated");
  state->updated_at_epoch = now;
  return 0;
}

int yai_runtime_containment_state_allows(const yai_runtime_containment_state_t *state,
                                         const char *action_class)
{
  if (!state || !action_class || !action_class[0]) return 0;
  if (strcmp(state->mode, YAI_RUNTIME_CONTAINMENT_MODE_NORMAL) == 0) return 1;
  if (strcmp(state->mode, YAI_RUNTIME_CONTAINMENT_MODE_RESTRICTED) == 0) {
    return strcmp(action_class, "read") == 0 || strcmp(action_class, "inspect") == 0;
  }
  return strcmp(action_class, "inspect") == 0;
}

int yai_runtime_containment_state_json(const yai_runtime_containment_state_t *state,
                                       char *out,
                                       size_t out_cap)
{
  if (!state || !out || out_cap == 0) return -1;
  if (snprintf(out,
               out_cap,
               "{\n"
               "  \"workspace_id\": \"%s\",\n"
               "  \"mode\": \"%s\",\n"
               "  \"isolation_state\": \"%s\",\n"
               "  \"policy_active\": %s,\n"
               "  \"valid_grants\": %zu,\n"
               "  \"last_reason\": \"%s\",\n"
               "  \"updated_at_epoch\": %lld\n"
               "}\n",
               state->workspace_id,
               state->mode,
               state->isolation_state,
               state->policy_active ? "true" : "false",
               state->valid_grants,
               state->last_reason,
               (long long)state->updated_at_epoch) >= (int)out_cap) {
    return -1;
  }
  return 0;
}

int yai_runtime_containment_start(const char *workspace_id, char *err, size_t err_cap)
{
  if (err && err_cap > 0) err[0] = '\0';
  if (g_containment_started) return 0;
  if (yai_runtime_containment_state_init(&g_containment_state,
                                         workspace_id ? workspace_id : "system") != 0) {
    if (err && err_cap > 0) snprintf(err, err_cap, "%s", "runtime_containment_init_failed");
    return -1;
  }
  g_containment_started = 1;
  return 0;
}

int yai_runtime_containment_evaluate(const char *reason, int64_t now_epoch_value)
{
  if (!g_containment_started) return -1;
  return yai_runtime_containment_state_apply(&g_containment_state,
                                             yai_runtime_policy_current(),
                                             yai_runtime_grants_current(),
                                             reason,
                                             now_epoch_value);
}

int yai_runtime_containment_stop(void)
{
  if (!g_containment_started) return 0;
  memset(&g_containment_state, 0, sizeof(g_containment_state));
  g_containment_started = 0;
  return 0;
}

const yai_runtime_containment_state_t *yai_runtime_containment_current(void)
{
  return g_containment_started ? &g_containment_state : NULL;
}
