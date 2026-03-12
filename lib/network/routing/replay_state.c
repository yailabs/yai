/* SPDX-License-Identifier: Apache-2.0 */

#include <yai/network/routing/replay.h>

#include <stdio.h>
#include <string.h>

int yai_mesh_replay_init(yai_mesh_replay_state_t *replay, int64_t now_epoch)
{
  if (!replay) return -1;
  memset(replay, 0, sizeof(*replay));
  replay->updated_at_epoch = now_epoch;
  (void)snprintf(replay->status, sizeof(replay->status), "%s", "idle");
  return 0;
}

int yai_mesh_replay_record_recovery(yai_mesh_replay_state_t *replay,
                                    uint32_t recovered_now,
                                    int64_t now_epoch)
{
  if (!replay) return -1;
  replay->recovered_items += recovered_now;
  replay->backlog_items = (replay->backlog_items > recovered_now) ? (replay->backlog_items - recovered_now) : 0u;
  replay->updated_at_epoch = now_epoch;
  (void)snprintf(replay->status,
                 sizeof(replay->status),
                 "%s",
                 replay->backlog_items == 0u ? "idle" : "recovering");
  return 0;
}
