#pragma once

#include <stdint.h>

typedef struct {
  uint64_t policy_view_handle;
  uint64_t active_rules;
  uint64_t denied_rules;
  uint64_t deferred_rules;
} yai_container_policy_view_t;

int yai_container_policy_view_get(const char *container_id,
                                  yai_container_policy_view_t *out_view);
