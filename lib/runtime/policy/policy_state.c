/* SPDX-License-Identifier: Apache-2.0 */

#include <yai/runtime/policy.h>

#include <stdio.h>
#include <string.h>
#include <time.h>

static yai_runtime_policy_state_t g_policy_state;
static int g_policy_started = 0;

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

int yai_runtime_policy_state_init(yai_runtime_policy_state_t *state, const char *workspace_id)
{
  if (!state || !workspace_id || !workspace_id[0]) return -1;
  memset(state, 0, sizeof(*state));
  if (copy_string(state->workspace_id, sizeof(state->workspace_id), workspace_id) != 0) return -1;
  if (copy_string(state->mode, sizeof(state->mode), "pending") != 0) return -1;
  if (copy_string(state->last_reason, sizeof(state->last_reason), "not_applied") != 0) return -1;
  return 0;
}

int yai_runtime_policy_state_apply(yai_runtime_policy_state_t *state,
                                   const char *snapshot_id,
                                   uint32_t effect_count,
                                   int64_t applied_at_epoch,
                                   const char *reason)
{
  if (!state || !snapshot_id || !snapshot_id[0]) return -1;
  if (copy_string(state->active_snapshot_id, sizeof(state->active_snapshot_id), snapshot_id) != 0) return -1;
  if (copy_string(state->mode, sizeof(state->mode), "active") != 0) return -1;
  state->active = 1;
  state->generation += 1u;
  state->effect_count = effect_count;
  state->applied_at_epoch = applied_at_epoch > 0 ? applied_at_epoch : now_epoch();
  if (reason && reason[0]) (void)copy_string(state->last_reason, sizeof(state->last_reason), reason);
  return 0;
}

int yai_runtime_policy_state_clear(yai_runtime_policy_state_t *state, const char *reason)
{
  if (!state) return -1;
  state->active = 0;
  state->effect_count = 0u;
  state->active_snapshot_id[0] = '\0';
  (void)copy_string(state->mode, sizeof(state->mode), "pending");
  (void)copy_string(state->last_reason,
                    sizeof(state->last_reason),
                    (reason && reason[0]) ? reason : "cleared");
  state->applied_at_epoch = now_epoch();
  return 0;
}

int yai_runtime_policy_state_json(const yai_runtime_policy_state_t *state,
                                  char *out,
                                  size_t out_cap)
{
  if (!state || !out || out_cap == 0) return -1;
  if (snprintf(out,
               out_cap,
               "{\n"
               "  \"workspace_id\": \"%s\",\n"
               "  \"mode\": \"%s\",\n"
               "  \"active_snapshot_id\": \"%s\",\n"
               "  \"generation\": %u,\n"
               "  \"effect_count\": %u,\n"
               "  \"active\": %s,\n"
               "  \"applied_at_epoch\": %lld,\n"
               "  \"last_reason\": \"%s\"\n"
               "}\n",
               state->workspace_id,
               state->mode,
               state->active_snapshot_id,
               state->generation,
               state->effect_count,
               state->active ? "true" : "false",
               (long long)state->applied_at_epoch,
               state->last_reason) >= (int)out_cap) {
    return -1;
  }
  return 0;
}

int yai_runtime_policy_start(const char *workspace_id, char *err, size_t err_cap)
{
  if (err && err_cap > 0) err[0] = '\0';
  if (g_policy_started) return 0;
  if (yai_runtime_policy_state_init(&g_policy_state, workspace_id ? workspace_id : "system") != 0) {
    if (err && err_cap > 0) snprintf(err, err_cap, "%s", "runtime_policy_init_failed");
    return -1;
  }
  g_policy_started = 1;
  return 0;
}

int yai_runtime_policy_apply_effective(const char *snapshot_id,
                                       uint32_t effect_count,
                                       int64_t applied_at_epoch,
                                       const char *reason)
{
  if (!g_policy_started) return -1;
  return yai_runtime_policy_state_apply(&g_policy_state,
                                        snapshot_id,
                                        effect_count,
                                        applied_at_epoch,
                                        reason);
}

int yai_runtime_policy_stop(void)
{
  if (!g_policy_started) return 0;
  (void)yai_runtime_policy_state_clear(&g_policy_state, "runtime_stop");
  g_policy_started = 0;
  return 0;
}

const yai_runtime_policy_state_t *yai_runtime_policy_current(void)
{
  return g_policy_started ? &g_policy_state : NULL;
}
