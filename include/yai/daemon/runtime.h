#pragma once

#include <yai/daemon/config.h>
#include <yai/daemon/edge_services.h>
#include <yai/daemon/edge_state.h>
#include <yai/daemon/local_runtime.h>
#include <yai/daemon/paths.h>

typedef struct yai_daemon_runtime {
  yai_daemon_config_t config;
  yai_daemon_paths_t paths;
  yai_daemon_edge_state_t edge_state;
  yai_daemon_edge_services_t services;
  yai_daemon_local_runtime_t *local;
  char instance_id[96];
  char edge_state_file[512];
  int running;
  unsigned int tick_count;
} yai_daemon_runtime_t;

int yai_daemon_runtime_init(yai_daemon_runtime_t *rt,
                            const yai_daemon_config_t *cfg);
int yai_daemon_runtime_start(yai_daemon_runtime_t *rt);
int yai_daemon_runtime_tick(yai_daemon_runtime_t *rt);
int yai_daemon_runtime_shutdown(yai_daemon_runtime_t *rt);

void yai_daemon_log(const yai_daemon_runtime_t *rt,
                    const char *level,
                    const char *message);
void yai_daemon_logf(const yai_daemon_runtime_t *rt,
                     const char *level,
                     const char *fmt,
                     ...);
