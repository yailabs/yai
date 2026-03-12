#pragma once

#include <yai/daemon/config.h>
#include <yai/runtime/local/services.h>
#include <yai/runtime/local/state.h>
#include <yai/daemon/network_binding.h>
#include <yai/runtime/local/paths.h>

typedef struct yai_edge_runtime {
  yai_edge_config_t config;
  yai_edge_paths_t paths;
  yai_edge_edge_state_t edge_state;
  yai_edge_edge_services_t services;
  yai_edge_local_runtime_t *local;
  char instance_id[96];
  char edge_state_file[512];
  int running;
  unsigned int tick_count;
} yai_edge_runtime_t;

typedef yai_edge_runtime_t yai_daemon_runtime_t;

int yai_edge_runtime_init(yai_edge_runtime_t *rt,
                            const yai_edge_config_t *cfg);
int yai_edge_runtime_start(yai_edge_runtime_t *rt);
int yai_edge_runtime_tick(yai_edge_runtime_t *rt);
int yai_edge_runtime_shutdown(yai_edge_runtime_t *rt);

void yai_edge_log(const yai_edge_runtime_t *rt,
                    const char *level,
                    const char *message);
void yai_edge_logf(const yai_edge_runtime_t *rt,
                     const char *level,
                     const char *fmt,
                     ...);
