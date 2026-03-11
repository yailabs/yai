/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct yai_mesh_replay_state {
  uint32_t backlog_items;
  uint32_t recovered_items;
  char status[24];
  int64_t updated_at_epoch;
} yai_mesh_replay_state_t;

int yai_mesh_replay_init(yai_mesh_replay_state_t *replay, int64_t now_epoch);
int yai_mesh_replay_record_recovery(yai_mesh_replay_state_t *replay,
                                    uint32_t recovered_now,
                                    int64_t now_epoch);

#ifdef __cplusplus
}
#endif
