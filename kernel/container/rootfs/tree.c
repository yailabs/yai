#include <string.h>

#include <yai/con/tree.h>

void yai_container_tree_defaults(yai_container_tree_t *tree) {
  if (!tree) {
    return;
  }
  memset(tree, 0, sizeof(*tree));
}

int yai_container_tree_project_defaults(const yai_container_root_t *root,
                                        yai_container_tree_t *tree) {
  if (!root || !tree) {
    return -1;
  }

  memset(tree, 0, sizeof(*tree));
  tree->tree_handle = root->root_projection_handle;
  tree->node_count = 8u;
  tree->leaf_count = 8u;
  return 0;
}
