/* SPDX-License-Identifier: Apache-2.0 */

#include <yai/pol/grants_state.h>

#include <stdio.h>
#include <string.h>
#include <time.h>

static yai_runtime_grants_state_t g_grants_state;
static int g_grants_started = 0;

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

static const char *validity_for(const yai_runtime_grant_entry_t *entry, int64_t now)
{
  if (!entry) return "invalid";
  if (entry->revoked) return "revoked";
  if (entry->expires_at_epoch > 0 && now >= entry->expires_at_epoch) return "expired";
  if (entry->refresh_after_epoch > 0 && now >= entry->refresh_after_epoch) return "refresh_required";
  return "valid";
}

static yai_runtime_grant_entry_t *find_entry_mut(yai_runtime_grants_state_t *state, const char *grant_id)
{
  size_t i;
  if (!state || !grant_id || !grant_id[0]) return NULL;
  for (i = 0; i < state->count; ++i) {
    if (strcmp(state->grants[i].grant_id, grant_id) == 0) return &state->grants[i];
  }
  return NULL;
}

int yai_runtime_grants_state_init(yai_runtime_grants_state_t *state, const char *workspace_id)
{
  if (!state || !workspace_id || !workspace_id[0]) return -1;
  memset(state, 0, sizeof(*state));
  if (copy_string(state->workspace_id, sizeof(state->workspace_id), workspace_id) != 0) return -1;
  state->updated_at_epoch = now_epoch();
  return 0;
}

int yai_runtime_grants_state_upsert(yai_runtime_grants_state_t *state,
                                    const char *grant_id,
                                    const char *scope,
                                    int64_t issued_at_epoch,
                                    int64_t refresh_after_epoch,
                                    int64_t expires_at_epoch,
                                    int revoked)
{
  yai_runtime_grant_entry_t *entry;
  if (!state || !grant_id || !grant_id[0] || !scope || !scope[0]) return -1;

  entry = find_entry_mut(state, grant_id);
  if (!entry) {
    if (state->count >= YAI_RUNTIME_GRANT_CAPACITY) return -1;
    entry = &state->grants[state->count++];
    memset(entry, 0, sizeof(*entry));
    if (copy_string(entry->grant_id, sizeof(entry->grant_id), grant_id) != 0) return -1;
  }

  if (copy_string(entry->scope, sizeof(entry->scope), scope) != 0) return -1;
  entry->issued_at_epoch = issued_at_epoch;
  entry->refresh_after_epoch = refresh_after_epoch;
  entry->expires_at_epoch = expires_at_epoch;
  entry->revoked = revoked ? 1 : 0;
  (void)copy_string(entry->validity_state, sizeof(entry->validity_state), validity_for(entry, now_epoch()));
  state->updated_at_epoch = now_epoch();
  return 0;
}

int yai_runtime_grants_state_refresh_validity(yai_runtime_grants_state_t *state, int64_t now_epoch_value)
{
  size_t i;
  int64_t now = now_epoch_value > 0 ? now_epoch_value : now_epoch();
  if (!state) return -1;
  state->valid_count = 0;
  for (i = 0; i < state->count; ++i) {
    const char *v = validity_for(&state->grants[i], now);
    (void)copy_string(state->grants[i].validity_state, sizeof(state->grants[i].validity_state), v);
    if (strcmp(v, "valid") == 0) state->valid_count += 1u;
  }
  state->updated_at_epoch = now;
  return 0;
}

int yai_runtime_grants_state_has_valid_scope(const yai_runtime_grants_state_t *state, const char *scope_prefix)
{
  size_t i;
  size_t n;
  if (!state || !scope_prefix || !scope_prefix[0]) return 0;
  n = strlen(scope_prefix);
  for (i = 0; i < state->count; ++i) {
    const yai_runtime_grant_entry_t *entry = &state->grants[i];
    if (strncmp(entry->scope, scope_prefix, n) == 0 && strcmp(entry->validity_state, "valid") == 0) return 1;
  }
  return 0;
}

int yai_runtime_grants_state_json(const yai_runtime_grants_state_t *state,
                                  char *out,
                                  size_t out_cap)
{
  if (!state || !out || out_cap == 0) return -1;
  if (snprintf(out,
               out_cap,
               "{\n"
               "  \"workspace_id\": \"%s\",\n"
               "  \"count\": %zu,\n"
               "  \"valid_count\": %zu,\n"
               "  \"updated_at_epoch\": %lld\n"
               "}\n",
               state->workspace_id,
               state->count,
               state->valid_count,
               (long long)state->updated_at_epoch) >= (int)out_cap) {
    return -1;
  }
  return 0;
}

int yai_runtime_grants_start(const char *workspace_id, char *err, size_t err_cap)
{
  if (err && err_cap > 0) err[0] = '\0';
  if (g_grants_started) return 0;
  if (yai_runtime_grants_state_init(&g_grants_state, workspace_id ? workspace_id : "user") != 0) {
    if (err && err_cap > 0) snprintf(err, err_cap, "%s", "runtime_grants_init_failed");
    return -1;
  }
  g_grants_started = 1;
  return 0;
}

int yai_runtime_grants_upsert(const char *grant_id,
                              const char *scope,
                              int64_t issued_at_epoch,
                              int64_t refresh_after_epoch,
                              int64_t expires_at_epoch,
                              int revoked)
{
  if (!g_grants_started) return -1;
  return yai_runtime_grants_state_upsert(&g_grants_state,
                                         grant_id,
                                         scope,
                                         issued_at_epoch,
                                         refresh_after_epoch,
                                         expires_at_epoch,
                                         revoked);
}

int yai_runtime_grants_refresh(int64_t now_epoch_value)
{
  if (!g_grants_started) return -1;
  return yai_runtime_grants_state_refresh_validity(&g_grants_state, now_epoch_value);
}

int yai_runtime_grants_stop(void)
{
  if (!g_grants_started) return 0;
  memset(&g_grants_state, 0, sizeof(g_grants_state));
  g_grants_started = 0;
  return 0;
}

const yai_runtime_grants_state_t *yai_runtime_grants_current(void)
{
  return g_grants_started ? &g_grants_state : NULL;
}
