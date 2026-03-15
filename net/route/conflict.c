/* SPDX-License-Identifier: Apache-2.0 */

#include <yai/net/conflict.h>

#include <stdio.h>
#include <string.h>

int yai_mesh_conflict_init(yai_mesh_conflict_state_t *state, int64_t now_epoch)
{
  if (!state) return -1;
  memset(state, 0, sizeof(*state));
  state->updated_at_epoch = now_epoch;
  return 0;
}

int yai_mesh_conflict_record(yai_mesh_conflict_state_t *state,
                             const char *conflict_key,
                             int resolved,
                             int64_t now_epoch)
{
  if (!state) return -1;

  if (conflict_key && conflict_key[0]) {
    (void)snprintf(state->conflict_key, sizeof(state->conflict_key), "%s", conflict_key);
  }

  if (resolved) {
    if (state->conflicts_open > 0u) state->conflicts_open -= 1u;
    state->conflicts_resolved += 1u;
  } else {
    state->conflicts_open += 1u;
  }

  state->updated_at_epoch = now_epoch;
  return 0;
}
