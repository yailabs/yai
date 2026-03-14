#pragma once

#include <stdint.h>

typedef enum {
  YAI_CONTAINER_LIFECYCLE_CREATED = 0,
  YAI_CONTAINER_LIFECYCLE_INITIALIZED,
  YAI_CONTAINER_LIFECYCLE_OPEN,
  YAI_CONTAINER_LIFECYCLE_ACTIVE,
  YAI_CONTAINER_LIFECYCLE_DEGRADED,
  YAI_CONTAINER_LIFECYCLE_SEALED,
  YAI_CONTAINER_LIFECYCLE_RECOVERY,
  YAI_CONTAINER_LIFECYCLE_DESTROYED,
  YAI_CONTAINER_LIFECYCLE_ARCHIVED,
} yai_container_lifecycle_state_t;

typedef struct {
  yai_container_lifecycle_state_t current;
  yai_container_lifecycle_state_t previous;
  int64_t created_at;
  int64_t updated_at;
  int64_t sealed_at;
  int64_t destroyed_at;
} yai_container_lifecycle_t;

int yai_container_lifecycle_transition_allowed(yai_container_lifecycle_state_t from,
                                               yai_container_lifecycle_state_t to);
const char *yai_container_lifecycle_name(yai_container_lifecycle_state_t value);
