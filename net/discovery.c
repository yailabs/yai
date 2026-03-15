/* SPDX-License-Identifier: Apache-2.0 */

#include <yai/net/discovery.h>

int yai_mesh_discovery_apply(yai_mesh_peer_registry_t *registry,
                             const yai_mesh_discovery_event_t *event)
{
  if (!registry || !event || !event->peer_id[0]) return -1;
  return yai_mesh_peer_registry_upsert(registry,
                                       event->peer_id,
                                       event->endpoint,
                                       event->role,
                                       event->connected,
                                       event->observed_at_epoch);
}
