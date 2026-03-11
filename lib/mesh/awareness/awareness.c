/* SPDX-License-Identifier: Apache-2.0 */

#include <yai/mesh/awareness.h>

#include <stdio.h>
#include <string.h>

int yai_mesh_awareness_refresh(yai_mesh_awareness_state_t *awareness,
                               const yai_mesh_peer_registry_t *registry,
                               int64_t now_epoch,
                               int64_t aging_after_sec,
                               int64_t stale_after_sec)
{
  size_t i;
  int64_t max_age = -1;

  if (!awareness || !registry) return -1;
  if (aging_after_sec <= 0) aging_after_sec = 15;
  if (stale_after_sec <= aging_after_sec) stale_after_sec = aging_after_sec * 2;

  memset(awareness, 0, sizeof(*awareness));
  awareness->peers_total = (uint32_t)registry->count;
  awareness->updated_at_epoch = now_epoch;

  for (i = 0; i < registry->count; ++i) {
    const yai_mesh_peer_t *peer = &registry->peers[i];
    int64_t age = now_epoch - peer->last_seen_at_epoch;
    if (peer->connected) awareness->peers_connected += 1u;
    if (age > max_age) max_age = age;
    if (age > stale_after_sec) awareness->peers_stale += 1u;
  }

  if (registry->count == 0) {
    (void)snprintf(awareness->freshness_state, sizeof(awareness->freshness_state), "%s", YAI_MESH_FRESHNESS_UNKNOWN);
  } else if (awareness->peers_stale > 0) {
    (void)snprintf(awareness->freshness_state, sizeof(awareness->freshness_state), "%s", YAI_MESH_FRESHNESS_STALE);
  } else if (max_age >= aging_after_sec) {
    (void)snprintf(awareness->freshness_state, sizeof(awareness->freshness_state), "%s", YAI_MESH_FRESHNESS_AGING);
  } else {
    (void)snprintf(awareness->freshness_state, sizeof(awareness->freshness_state), "%s", YAI_MESH_FRESHNESS_FRESH);
  }

  return 0;
}
