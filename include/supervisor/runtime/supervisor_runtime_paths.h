#pragma once

#include <yai/dmn/runtime/runtime_config.h>

typedef struct yai_edge_paths {
  char home[YAI_EDGE_PATH_MAX];
  char config_dir[YAI_EDGE_PATH_MAX];
  char state_dir[YAI_EDGE_PATH_MAX];
  char log_dir[YAI_EDGE_PATH_MAX];
  char spool_dir[YAI_EDGE_PATH_MAX];
  char bindings_dir[YAI_EDGE_PATH_MAX];
  char identity_dir[YAI_EDGE_PATH_MAX];
  char run_dir[YAI_EDGE_PATH_MAX];
  char pid_file[YAI_EDGE_PATH_MAX];
  char health_file[YAI_EDGE_PATH_MAX];
  char instance_file[YAI_EDGE_PATH_MAX];
} yai_edge_paths_t;

int yai_edge_paths_build(const yai_edge_config_t *cfg,
                           yai_edge_paths_t *paths);
int yai_edge_paths_ensure(const yai_edge_paths_t *paths);
