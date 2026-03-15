#pragma once

#include <stdint.h>

typedef struct {
  uint64_t grants_view_handle;
  uint64_t active_grants;
  uint64_t suspended_grants;
  uint64_t revoked_grants;
} yai_container_grants_view_t;

int yai_container_grants_view_get(const char *container_id,
                                  yai_container_grants_view_t *out_view);
