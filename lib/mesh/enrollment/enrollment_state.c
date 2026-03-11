/* SPDX-License-Identifier: Apache-2.0 */

#include <yai/mesh/enrollment.h>

#include <string.h>

int yai_mesh_enrollment_init(yai_mesh_enrollment_state_t *state, int64_t now_epoch)
{
  if (!state) return -1;
  memset(state, 0, sizeof(*state));
  state->updated_at_epoch = now_epoch;
  return 0;
}

int yai_mesh_enrollment_record(yai_mesh_enrollment_state_t *state,
                               int accepted,
                               int64_t now_epoch)
{
  if (!state) return -1;
  if (accepted > 0) state->accepted += 1u;
  else if (accepted < 0) state->rejected += 1u;
  else state->pending += 1u;
  state->updated_at_epoch = now_epoch;
  return 0;
}
