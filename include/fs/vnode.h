#pragma once

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef uint64_t yai_vnode_handle_t;

typedef enum {
  YAI_VNODE_NONE = 0,
  YAI_VNODE_FILE,
  YAI_VNODE_DIRECTORY,
  YAI_VNODE_SOCKET,
  YAI_VNODE_DEVICE,
  YAI_VNODE_SYMLINK,
} yai_vnode_class_t;

typedef struct {
  yai_vnode_handle_t handle;
  yai_vnode_class_t vnode_class;
  uint64_t fs_handle;
  uint64_t parent_handle;
  uint64_t flags;
  char name[128];
} yai_vnode_t;

void yai_vnode_defaults(yai_vnode_t *vnode);
int yai_vnode_create(yai_vnode_class_t vnode_class,
                     const char *name,
                     uint64_t parent_handle,
                     yai_vnode_t *out_vnode);

#ifdef __cplusplus
}
#endif
