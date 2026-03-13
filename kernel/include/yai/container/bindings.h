#pragma once

#include <stdint.h>

typedef struct {
  uint64_t daemon_binding_handle;
  uint64_t orchestration_binding_handle;
  uint64_t network_binding_handle;
} yai_container_bindings_t;

int yai_container_bindings_set(const char *container_id,
                               const yai_container_bindings_t *bindings);
