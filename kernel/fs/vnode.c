#include <stdio.h>
#include <string.h>

#include <yai/fs/vnode.h>

static yai_vnode_handle_t g_next_vnode_handle = 1u;

void yai_vnode_defaults(yai_vnode_t *vnode) {
  if (!vnode) {
    return;
  }
  memset(vnode, 0, sizeof(*vnode));
}

int yai_vnode_create(yai_vnode_class_t vnode_class,
                     const char *name,
                     uint64_t parent_handle,
                     yai_vnode_t *out_vnode) {
  if (!out_vnode || !name || name[0] == '\0' || vnode_class == YAI_VNODE_NONE) {
    return -1;
  }

  yai_vnode_defaults(out_vnode);
  out_vnode->handle = g_next_vnode_handle++;
  out_vnode->vnode_class = vnode_class;
  out_vnode->parent_handle = parent_handle;

  if (snprintf(out_vnode->name, sizeof(out_vnode->name), "%s", name) >= (int)sizeof(out_vnode->name)) {
    return -1;
  }

  return 0;
}
