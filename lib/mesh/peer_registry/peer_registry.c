/* SPDX-License-Identifier: Apache-2.0 */

#include <yai/mesh/peer_registry.h>

#include <stdio.h>
#include <string.h>

static int copy_string(char *dst, size_t cap, const char *src)
{
  if (!dst || cap == 0 || !src || !src[0]) return -1;
  if (snprintf(dst, cap, "%s", src) >= (int)cap) return -1;
  return 0;
}

static yai_mesh_peer_t *find_peer_mut(yai_mesh_peer_registry_t *registry, const char *peer_id)
{
  size_t i;
  if (!registry || !peer_id || !peer_id[0]) return NULL;
  for (i = 0; i < registry->count; ++i) {
    if (strcmp(registry->peers[i].peer_id, peer_id) == 0) return &registry->peers[i];
  }
  return NULL;
}

int yai_mesh_peer_registry_init(yai_mesh_peer_registry_t *registry,
                                const yai_mesh_identity_t *self_identity)
{
  if (!registry || !self_identity || !yai_mesh_identity_is_valid(self_identity)) return -1;
  memset(registry, 0, sizeof(*registry));
  registry->self = *self_identity;
  return 0;
}

int yai_mesh_peer_registry_upsert(yai_mesh_peer_registry_t *registry,
                                  const char *peer_id,
                                  const char *endpoint,
                                  const char *role,
                                  int connected,
                                  int64_t observed_at_epoch)
{
  yai_mesh_peer_t *peer;
  if (!registry || !peer_id || !peer_id[0]) return -1;

  peer = find_peer_mut(registry, peer_id);
  if (!peer) {
    if (registry->count >= YAI_MESH_PEER_CAPACITY) return -1;
    peer = &registry->peers[registry->count++];
    memset(peer, 0, sizeof(*peer));
    if (copy_string(peer->peer_id, sizeof(peer->peer_id), peer_id) != 0) return -1;
    if (copy_string(peer->membership_state, sizeof(peer->membership_state), "joined") != 0) return -1;
    peer->discovered_at_epoch = observed_at_epoch;
  }

  if (endpoint && endpoint[0]) (void)copy_string(peer->endpoint, sizeof(peer->endpoint), endpoint);
  if (role && role[0]) (void)copy_string(peer->role, sizeof(peer->role), role);
  peer->connected = connected ? 1 : 0;
  peer->last_seen_at_epoch = observed_at_epoch;
  return 0;
}

const yai_mesh_peer_t *yai_mesh_peer_registry_find(const yai_mesh_peer_registry_t *registry,
                                                   const char *peer_id)
{
  size_t i;
  if (!registry || !peer_id || !peer_id[0]) return NULL;
  for (i = 0; i < registry->count; ++i) {
    if (strcmp(registry->peers[i].peer_id, peer_id) == 0) return &registry->peers[i];
  }
  return NULL;
}

int yai_mesh_peer_registry_mark_seen(yai_mesh_peer_registry_t *registry,
                                     const char *peer_id,
                                     int64_t seen_at_epoch,
                                     int connected)
{
  yai_mesh_peer_t *peer = find_peer_mut(registry, peer_id);
  if (!peer) return -1;
  peer->last_seen_at_epoch = seen_at_epoch;
  peer->connected = connected ? 1 : 0;
  return 0;
}

size_t yai_mesh_peer_registry_connected_count(const yai_mesh_peer_registry_t *registry)
{
  size_t i;
  size_t count = 0;
  if (!registry) return 0;
  for (i = 0; i < registry->count; ++i) {
    if (registry->peers[i].connected) count++;
  }
  return count;
}
