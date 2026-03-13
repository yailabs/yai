#pragma once

#include <stddef.h>
#include <stdint.h>

#include "root.h"

typedef struct {
  char container_id[64];
  char projected_root[YAI_CONTAINER_PATH_MAX + 1u];
  char backing_store[YAI_CONTAINER_PATH_MAX + 1u];
  uint64_t root_handle;
  uint64_t backing_store_handle;
} yai_container_path_context_t;

int yai_container_paths_join(const char *root,
                             const char *relative,
                             char *out,
                             size_t out_cap);
int yai_container_path_context_load(const char *container_id,
                                    yai_container_path_context_t *out_context);
int yai_container_resolve_path(const yai_container_path_context_t *context,
                               const char *container_path,
                               char *out_host_path,
                               size_t out_cap);
int yai_container_can_traverse(const char *container_id, const char *container_path);
