#pragma once

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define YAI_FS_MOUNT_MAX 128u

typedef uint64_t yai_mount_handle_t;

typedef enum {
  YAI_FS_MOUNT_NONE = 0,
  YAI_FS_MOUNT_ROOT,
  YAI_FS_MOUNT_CONTAINER,
  YAI_FS_MOUNT_RUNTIME,
  YAI_FS_MOUNT_EXTERNAL,
} yai_fs_mount_class_t;

typedef struct {
  yai_mount_handle_t handle;
  yai_fs_mount_class_t mount_class;
  uint64_t owner_handle;
  uint64_t flags;
  char source[256];
  char target[256];
} yai_mount_entry_t;

typedef struct {
  yai_mount_entry_t entries[YAI_FS_MOUNT_MAX];
  size_t len;
} yai_mount_table_t;

void yai_mount_table_defaults(yai_mount_table_t *table);
int yai_mount_table_attach(yai_mount_table_t *table,
                           const yai_mount_entry_t *entry);
int yai_mount_table_lookup(const yai_mount_table_t *table,
                           const char *target,
                           yai_mount_entry_t *out_entry);

#ifdef __cplusplus
}
#endif
