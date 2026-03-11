/* SPDX-License-Identifier: Apache-2.0 */

#include <yai/mesh/transport.h>

#include <stdio.h>
#include <string.h>

int yai_mesh_transport_refresh(yai_mesh_transport_state_t *transport,
                               const yai_mesh_peer_registry_t *registry,
                               int64_t now_epoch)
{
  size_t connected;
  if (!transport || !registry) return -1;

  memset(transport, 0, sizeof(*transport));
  connected = yai_mesh_peer_registry_connected_count(registry);
  transport->path_count = (uint32_t)registry->count;
  transport->degraded_paths = (uint32_t)(registry->count >= connected ? (registry->count - connected) : 0u);
  transport->updated_at_epoch = now_epoch;

  if (registry->count == 0) {
    (void)snprintf(transport->reachability_state, sizeof(transport->reachability_state), "%s", "isolated");
  } else if (transport->degraded_paths > 0u) {
    (void)snprintf(transport->reachability_state, sizeof(transport->reachability_state), "%s", "degraded");
  } else {
    (void)snprintf(transport->reachability_state, sizeof(transport->reachability_state), "%s", "connected");
  }

  return 0;
}
