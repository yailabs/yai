#pragma once

#include <stddef.h>
#include <stdint.h>

#include <yai/kernel/handles.h>

#define YAI_CONTAINER_MOUNT_MAX 127u
#define YAI_CONTAINER_MOUNT_PATH_MAX 255u

typedef enum {
  YAI_CONTAINER_MOUNT_RO = 0,
  YAI_CONTAINER_MOUNT_RW,
  YAI_CONTAINER_MOUNT_PRIVILEGED,
} yai_container_mount_policy_t;

typedef enum {
  YAI_CONTAINER_MOUNT_INTERNAL_ROOT = 0,
  YAI_CONTAINER_MOUNT_ATTACHED_EXTERNAL,
  YAI_CONTAINER_MOUNT_PRIVILEGED_DIAGNOSTIC,
  YAI_CONTAINER_MOUNT_EPHEMERAL,
} yai_container_mount_class_t;

typedef enum {
  YAI_CONTAINER_VISIBILITY_INTERNAL = 0,
  YAI_CONTAINER_VISIBILITY_ATTACHED,
  YAI_CONTAINER_VISIBILITY_READ_ONLY,
  YAI_CONTAINER_VISIBILITY_READ_WRITE,
  YAI_CONTAINER_VISIBILITY_HIDDEN,
  YAI_CONTAINER_VISIBILITY_PRIVILEGED_ONLY,
} yai_container_visibility_class_t;

typedef enum {
  YAI_CONTAINER_ATTACHABILITY_NONE = 0,
  YAI_CONTAINER_ATTACHABILITY_CONTROLLED = 1,
  YAI_CONTAINER_ATTACHABILITY_PRIVILEGED = 2,
} yai_container_attachability_class_t;

/* Low-level kernel policy class kept for substrate validation. */
enum yai_mount_policy_class {
  YAI_MOUNT_POLICY_READONLY = 0,
  YAI_MOUNT_POLICY_SCOPED_RW = 1,
  YAI_MOUNT_POLICY_PRIVILEGED_RW = 2
};

typedef struct {
  uint64_t mount_id;
  char source[YAI_CONTAINER_MOUNT_PATH_MAX + 1u];
  char target[YAI_CONTAINER_MOUNT_PATH_MAX + 1u];
  yai_container_mount_policy_t policy;
  yai_container_mount_class_t mount_class;
  yai_container_visibility_class_t visibility_class;
  yai_container_attachability_class_t attachability_class;
  uint64_t flags;
} yai_container_mount_t;

typedef struct {
  yai_container_mount_t entries[YAI_CONTAINER_MOUNT_MAX + 1u];
  size_t len;
} yai_container_mount_set_t;

int yai_kernel_mounts_validate_policy(enum yai_mount_policy_class mount_policy,
                                      uint64_t flags);

int yai_container_mounts_add(const char *container_id,
                             const yai_container_mount_t *mount);
int yai_container_attach_mount(const char *container_id,
                               const yai_container_mount_t *mount);
int yai_container_get_mount_set(const char *container_id,
                                yai_container_mount_set_t *out_set);
int yai_container_is_path_visible(const char *container_id,
                                  const char *container_path,
                                  int privileged_access);
