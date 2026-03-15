#pragma once

#include <stdint.h>

#define YAI_CONTAINER_ID_MAX 63u
#define YAI_CONTAINER_PROFILE_MAX 63u
#define YAI_CONTAINER_SOURCE_MAX 63u

typedef enum {
  YAI_CONTAINER_CLASS_INTERACTIVE = 0,
  YAI_CONTAINER_CLASS_MANAGED,
  YAI_CONTAINER_CLASS_SERVICE,
  YAI_CONTAINER_CLASS_SYSTEM,
  YAI_CONTAINER_CLASS_RECOVERY,
} yai_container_class_t;

typedef struct {
  char container_id[YAI_CONTAINER_ID_MAX + 1u];
  yai_container_class_t container_class;
  char container_profile[YAI_CONTAINER_PROFILE_MAX + 1u];
  char creation_source[YAI_CONTAINER_SOURCE_MAX + 1u];
  uint64_t owner_handle;
  uint64_t state_handle;
} yai_container_identity_t;

const char *yai_container_class_name(yai_container_class_t value);
