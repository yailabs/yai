#pragma once

#include <stdint.h>

typedef struct {
  char conflict_key[128];
  uint32_t conflicts_open;
  uint32_t conflicts_resolved;
  int64_t updated_at_epoch;
} yai_mesh_conflict_state_t;

int yai_mesh_conflict_init(yai_mesh_conflict_state_t *state, int64_t now_epoch);
int yai_mesh_conflict_record(yai_mesh_conflict_state_t *state,
                             const char *conflict_key,
                             int resolved,
                             int64_t now_epoch);
