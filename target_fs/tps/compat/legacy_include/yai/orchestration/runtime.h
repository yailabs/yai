#pragma once

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

typedef enum yai_exec_runtime_state {
  YAI_EXEC_OFFLINE = 0,
  YAI_EXEC_READY = 1,
  YAI_EXEC_BUSY = 2,
  YAI_EXEC_ERROR = 3
} yai_exec_runtime_state_t;

#define YAI_EXEC_RPC_BUFFER_MAX 4096

typedef struct yai_exec_config {
  char storage_backend[32];
  uint16_t max_parallel_agents;
  bool enforce_tla_safety;
} yai_exec_config_t;

/* YD-1 boundary lock:
 * - exec mediates owner/edge source-plane operations
 * - exec is not canonical workspace truth
 * - final truth persists only in owner runtime/data/graph planes
 */
const char *yai_exec_runtime_state_name(yai_exec_runtime_state_t state);
int yai_exec_runtime_probe(void);
int yai_exec_config_load_initial(const char *config_path, yai_exec_config_t *out_cfg);
bool yai_exec_config_enforce_limits(yai_exec_config_t *cfg);

int yai_exec_source_ingest_operation_known(const char *command_id);
int yai_exec_source_ingest_handle(const char *workspace_id,
                                  const char *command_id,
                                  const char *payload_json,
                                  char *out_json,
                                  size_t out_cap,
                                  char *out_reason,
                                  size_t reason_cap);
