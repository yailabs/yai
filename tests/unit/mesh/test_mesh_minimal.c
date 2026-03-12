/* SPDX-License-Identifier: Apache-2.0 */

#include <assert.h>
#include <stdio.h>
#include <string.h>

#include <yai/network/identity/identity.h>
#include <yai/network/discovery/peer_registry.h>
#include <yai/network/discovery/membership.h>
#include <yai/network/discovery/discovery.h>
#include <yai/network/topologies/sovereign_overlay/awareness.h>
#include <yai/network/topologies/sovereign_overlay/coordination.h>
#include <yai/network/transport/session.h>
#include <yai/network/transport/replay.h>
#include <yai/network/topologies/sovereign_overlay/conflict.h>
#include <yai/network/overlay/containment.h>
#include <yai/network/identity/enrollment.h>

int main(void)
{
  yai_mesh_identity_t self;
  yai_mesh_peer_registry_t registry;
  yai_mesh_membership_state_t membership;
  yai_mesh_awareness_state_t awareness;
  yai_mesh_coordination_state_t coordination;
  yai_mesh_transport_state_t transport;
  yai_mesh_replay_state_t replay;
  yai_mesh_conflict_state_t conflict;
  yai_mesh_containment_state_t containment;
  yai_mesh_enrollment_state_t enrollment;
  yai_mesh_discovery_event_t event = {0};
  const yai_mesh_peer_t *peer = NULL;

  assert(yai_mesh_identity_init(&self, "mesh-a", "node-1", "edge", 1000) == 0);
  assert(yai_mesh_identity_is_valid(&self) == 1);
  assert(yai_mesh_peer_registry_init(&registry, &self) == 0);

  (void)snprintf(event.peer_id, sizeof(event.peer_id), "%s", "peer-a");
  (void)snprintf(event.endpoint, sizeof(event.endpoint), "%s", "unix:///tmp/peer-a.sock");
  (void)snprintf(event.role, sizeof(event.role), "%s", "edge");
  event.connected = 1;
  event.observed_at_epoch = 1010;
  assert(yai_mesh_discovery_apply(&registry, &event) == 0);

  peer = yai_mesh_peer_registry_find(&registry, "peer-a");
  assert(peer != NULL);
  assert(peer->connected == 1);

  assert(yai_mesh_membership_init(&membership, "ws-1", 1011) == 0);
  assert(yai_mesh_membership_join_self(&membership, 1012) == 0);
  assert(yai_mesh_membership_refresh(&membership, &registry, 1013) == 0);
  assert(membership.active_peers == 1u);

  assert(yai_mesh_awareness_refresh(&awareness, &registry, 1014, 10, 30) == 0);
  assert(awareness.peers_total == 1u);
  assert(strcmp(awareness.freshness_state, YAI_MESH_FRESHNESS_FRESH) == 0);

  assert(yai_mesh_coordination_evaluate(&coordination, &registry, &membership, 1015) == 0);
  assert(coordination.participants >= 2u);
  assert(coordination.leader_peer_id[0] != '\0');

  assert(yai_mesh_transport_refresh(&transport, &registry, 1016) == 0);
  assert(strcmp(transport.reachability_state, "connected") == 0);

  assert(yai_mesh_replay_init(&replay, 1017) == 0);
  replay.backlog_items = 3u;
  assert(yai_mesh_replay_record_recovery(&replay, 2u, 1018) == 0);
  assert(replay.backlog_items == 1u);

  assert(yai_mesh_conflict_init(&conflict, 1019) == 0);
  assert(yai_mesh_conflict_record(&conflict, 0, 1020) == 0);
  assert(conflict.conflicts_open == 1u);

  assert(yai_mesh_containment_init(&containment, 1021) == 0);
  assert(yai_mesh_containment_set_mode(&containment, "quarantine", 1u, 1022) == 0);
  assert(strcmp(containment.mode, "quarantine") == 0);

  assert(yai_mesh_enrollment_init(&enrollment, 1023) == 0);
  assert(yai_mesh_enrollment_record(&enrollment, 1, 1024) == 0);
  assert(enrollment.accepted == 1u);

  return 0;
}
