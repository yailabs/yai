#pragma once

#include <stdint.h>

#include "root.h"

typedef struct {
  uint64_t tree_handle;
  uint64_t node_count;
  uint64_t leaf_count;
} yai_container_tree_t;

void yai_container_tree_defaults(yai_container_tree_t *tree);
int yai_container_tree_project_defaults(const yai_container_root_t *root,
                                        yai_container_tree_t *tree);
