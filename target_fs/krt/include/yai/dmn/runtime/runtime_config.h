#pragma once

#include <stddef.h>

#define YAI_EDGE_PATH_MAX 512
#define YAI_EDGE_LABEL_MAX 128
#define YAI_EDGE_LEVEL_MAX 32
#define YAI_EDGE_MODE_MAX 32

typedef struct yai_edge_config {
  char home[YAI_EDGE_PATH_MAX];
  char config_path[YAI_EDGE_PATH_MAX];
  char owner_ref[YAI_EDGE_PATH_MAX];
  char source_label[YAI_EDGE_LABEL_MAX];
  char log_level[YAI_EDGE_LEVEL_MAX];
  char mode[YAI_EDGE_MODE_MAX];
  char bindings_manifest[YAI_EDGE_PATH_MAX];
  unsigned int tick_ms;
  int max_ticks;
} yai_edge_config_t;

int yai_edge_config_defaults(yai_edge_config_t *cfg);
int yai_edge_config_apply_env(yai_edge_config_t *cfg);
int yai_edge_config_apply_file(yai_edge_config_t *cfg, const char *path);
int yai_edge_config_validate(const yai_edge_config_t *cfg);

int yai_edge_config_set_string(char *dst,
                                 size_t dst_cap,
                                 const char *value);

int yai_edge_config_parse_uint(const char *raw, unsigned int *out);
int yai_edge_config_parse_int(const char *raw, int *out);
