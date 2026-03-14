#pragma once

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <string.h>

#include <yai/abi/protocol/vault.h>
#include <yai/ipc/message_types.h>
#include <yai/ipc/ids.h>

#ifdef __cplusplus
extern "C" {
#endif

#define SHM_VAULT_PREFIX "/yai_vault_"
#define MAX_WS_ID 64
#define MAX_TRACE_ID 64
#define MAX_ERR_MSG 256
#define YAI_VAULT_MAGIC 0x59414956u
#define YAI_VAULT_VERSION 1u

typedef enum {
  YAI_STATE_HALT = 0,
  YAI_STATE_PREBOOT,
  YAI_STATE_READY,
  YAI_STATE_HANDOFF_COMPLETE,
  YAI_STATE_RUNNING,
  YAI_STATE_SUSPENDED,
  YAI_STATE_ERROR
} yai_state_t;

#pragma pack(push, 1)
typedef struct {
  uint32_t status;
  uint32_t energy_quota;
  uint32_t energy_consumed;
  char workspace_id[MAX_WS_ID];
  char trace_id[MAX_TRACE_ID];
  bool authority_lock;
  uint8_t _pad0[3];
  uint32_t last_command_id;
  uint32_t command_seq;
  uint32_t last_processed_seq;
  uint32_t last_result;
  char response_buffer[1024];
  char last_error[MAX_ERR_MSG];
  uint64_t logical_clock;
} yai_vault_t;
#pragma pack(pop)

static inline uint32_t yai_resolve_command_class(uint32_t cmd) {
  uint32_t cls_prefix = cmd & 0xFF00u;
  if (cls_prefix == YAI_CMD_CLASS_PROVIDER || cls_prefix == 0x0400u) {
    return 0x02u;
  }
  return 0x01u;
}

static inline bool yai_vault_allows_command(const yai_vault_t *v, uint32_t cmd) {
  if (!v) return false;
  if (v->authority_lock && yai_resolve_command_class(cmd) == 0x02u) {
    return false;
  }
  return true;
}

static inline void yai_vault_bootstrap_defaults(yai_vault_t *v, const char *ws_id_opt) {
  if (!v) return;
  memset(v, 0, sizeof(*v));
  v->status = YAI_STATE_PREBOOT;
  v->energy_quota = 1000;
  v->authority_lock = true;
  if (ws_id_opt) {
    strncpy(v->workspace_id, ws_id_opt, MAX_WS_ID - 1);
  }
}

static inline void yai_vault_set_error(yai_vault_t *v, const char *msg) {
  if (!v) return;
  v->last_result = 1;
  strncpy(v->last_error, msg ? msg : "unknown_err", MAX_ERR_MSG - 1);
}

#ifdef __cplusplus
}
#endif
