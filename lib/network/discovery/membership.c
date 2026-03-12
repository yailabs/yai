/* SPDX-License-Identifier: Apache-2.0 */

#include <yai/network/discovery/membership.h>

#include <stdio.h>
#include <string.h>

int yai_mesh_membership_init(yai_mesh_membership_state_t *membership,
                             const char *workspace_id,
                             int64_t now_epoch)
{
  if (!membership || !workspace_id || !workspace_id[0]) return -1;
  memset(membership, 0, sizeof(*membership));
  if (snprintf(membership->workspace_id, sizeof(membership->workspace_id), "%s", workspace_id) >=
      (int)sizeof(membership->workspace_id)) {
    return -1;
  }
  membership->updated_at_epoch = now_epoch;
  return 0;
}

int yai_mesh_membership_join_self(yai_mesh_membership_state_t *membership,
                                  int64_t now_epoch)
{
  if (!membership) return -1;
  membership->self_joined = 1;
  membership->generation += 1u;
  membership->updated_at_epoch = now_epoch;
  return 0;
}

int yai_mesh_membership_refresh(yai_mesh_membership_state_t *membership,
                                const yai_mesh_peer_registry_t *registry,
                                int64_t now_epoch)
{
  if (!membership || !registry) return -1;
  membership->active_peers = (uint32_t)yai_mesh_peer_registry_connected_count(registry);
  membership->updated_at_epoch = now_epoch;
  return 0;
}
