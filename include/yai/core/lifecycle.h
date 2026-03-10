#pragma once

#include <stddef.h>

int yai_run_preboot_checks(void);
int yai_ensure_runtime_layout(const char *ws_id);
int yai_init_system_shm(void);

typedef struct yai_runtime_capability_state {
  int initialized;
  int transport_ready;
  int providers_ready;
  int memory_ready;
  int cognition_ready;
  char workspace_id[64];
  char runtime_name[64];
  int enable_mock_provider;
} yai_runtime_capability_state_t;

int yai_runtime_capabilities_start(const char *workspace_id,
                                   const char *runtime_name,
                                   int enable_mock_provider,
                                   char *err,
                                   size_t err_cap);
int yai_runtime_capabilities_bind_workspace(const char *workspace_id,
                                            char *err,
                                            size_t err_cap);
int yai_runtime_capabilities_stop(char *err, size_t err_cap);
int yai_runtime_capabilities_is_ready(void);
const yai_runtime_capability_state_t *yai_runtime_capabilities_state(void);
