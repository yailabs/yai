/* SPDX-License-Identifier: Apache-2.0 */

#include <yai/net/coordination.h>

#include <stdio.h>
#include <string.h>

static const char *pick_leader(const yai_mesh_peer_registry_t *registry)
{
  size_t i;
  if (!registry) return NULL;

  for (i = 0; i < registry->count; ++i) {
    if (registry->peers[i].connected) return registry->peers[i].peer_id;
  }

  return registry->self.node_id[0] ? registry->self.node_id : NULL;
}

int yai_mesh_coordination_evaluate(yai_mesh_coordination_state_t *coordination,
                                   const yai_mesh_peer_registry_t *registry,
                                   const yai_mesh_membership_state_t *membership,
                                   int64_t now_epoch)
{
  const char *leader;
  uint32_t participants;

  if (!coordination || !registry || !membership) return -1;

  memset(coordination, 0, sizeof(*coordination));

  participants = (uint32_t)yai_mesh_peer_registry_connected_count(registry);
  if (membership->self_joined) participants += 1u;

  leader = pick_leader(registry);
  if (leader) {
    (void)snprintf(coordination->leader_peer_id,
                   sizeof(coordination->leader_peer_id),
                   "%s",
                   leader);
  }

  coordination->participants = participants;
  coordination->quorum_size = (participants / 2u) + 1u;
  coordination->updated_at_epoch = now_epoch;
  return 0;
}
