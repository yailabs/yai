#pragma once

#include <stddef.h>
#include <stdint.h>

#define YAI_RUNTIME_GRANT_CAPACITY 16
#define YAI_RUNTIME_GRANT_ID_MAX 128
#define YAI_RUNTIME_GRANT_SCOPE_MAX 128
#define YAI_RUNTIME_GRANT_STATE_MAX 32
#define YAI_RUNTIME_GRANTS_WORKSPACE_MAX 64

typedef struct yai_runtime_grant_entry {
  char grant_id[YAI_RUNTIME_GRANT_ID_MAX];
  char scope[YAI_RUNTIME_GRANT_SCOPE_MAX];
  char validity_state[YAI_RUNTIME_GRANT_STATE_MAX];
  int64_t issued_at_epoch;
  int64_t refresh_after_epoch;
  int64_t expires_at_epoch;
  int revoked;
} yai_runtime_grant_entry_t;

typedef struct yai_runtime_grants_state {
  char workspace_id[YAI_RUNTIME_GRANTS_WORKSPACE_MAX];
  yai_runtime_grant_entry_t grants[YAI_RUNTIME_GRANT_CAPACITY];
  size_t count;
  size_t valid_count;
  int64_t updated_at_epoch;
} yai_runtime_grants_state_t;

int yai_runtime_grants_state_init(yai_runtime_grants_state_t *state, const char *workspace_id);
int yai_runtime_grants_state_upsert(yai_runtime_grants_state_t *state,
                                    const char *grant_id,
                                    const char *scope,
                                    int64_t issued_at_epoch,
                                    int64_t refresh_after_epoch,
                                    int64_t expires_at_epoch,
                                    int revoked);
int yai_runtime_grants_state_refresh_validity(yai_runtime_grants_state_t *state, int64_t now_epoch);
int yai_runtime_grants_state_has_valid_scope(const yai_runtime_grants_state_t *state, const char *scope_prefix);
int yai_runtime_grants_state_json(const yai_runtime_grants_state_t *state,
                                  char *out,
                                  size_t out_cap);

int yai_runtime_grants_start(const char *workspace_id, char *err, size_t err_cap);
int yai_runtime_grants_upsert(const char *grant_id,
                              const char *scope,
                              int64_t issued_at_epoch,
                              int64_t refresh_after_epoch,
                              int64_t expires_at_epoch,
                              int revoked);
int yai_runtime_grants_refresh(int64_t now_epoch);
int yai_runtime_grants_stop(void);
const yai_runtime_grants_state_t *yai_runtime_grants_current(void);
