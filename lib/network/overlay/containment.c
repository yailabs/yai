/* SPDX-License-Identifier: Apache-2.0 */

#include <yai/network/overlay/containment.h>

#include <stdio.h>
#include <string.h>

int yai_mesh_containment_init(yai_mesh_containment_state_t *state, int64_t now_epoch)
{
  if (!state) return -1;
  memset(state, 0, sizeof(*state));
  (void)snprintf(state->mode, sizeof(state->mode), "%s", "normal");
  state->updated_at_epoch = now_epoch;
  return 0;
}

int yai_mesh_containment_set_mode(yai_mesh_containment_state_t *state,
                                  const char *mode,
                                  uint32_t quarantined_peers,
                                  int64_t now_epoch)
{
  if (!state || !mode || !mode[0]) return -1;
  if (snprintf(state->mode, sizeof(state->mode), "%s", mode) >= (int)sizeof(state->mode)) return -1;
  state->quarantined_peers = quarantined_peers;
  state->updated_at_epoch = now_epoch;
  return 0;
}
