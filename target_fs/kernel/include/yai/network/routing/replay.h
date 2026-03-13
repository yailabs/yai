#pragma once

#include <stdint.h>

typedef struct {
  int initialized;
  char status[32];
  uint32_t backlog_items;
  uint32_t recovered_items;
  int64_t updated_at_epoch;
} yai_mesh_replay_state_t;

int yai_mesh_replay_init(yai_mesh_replay_state_t *state, int64_t now_epoch);
int yai_mesh_replay_record(yai_mesh_replay_state_t *state,
                           uint32_t recovered_now,
                           int64_t now_epoch);
