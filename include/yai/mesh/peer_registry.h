/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <stddef.h>
#include <stdint.h>

#include <yai/mesh/identity.h>

#ifdef __cplusplus
extern "C" {
#endif

#define YAI_MESH_PEER_MAX 96
#define YAI_MESH_ENDPOINT_MAX 256
#define YAI_MESH_MEMBERSHIP_STATE_MAX 24
#define YAI_MESH_PEER_CAPACITY 64

typedef struct yai_mesh_peer {
  char peer_id[YAI_MESH_PEER_MAX];
  char endpoint[YAI_MESH_ENDPOINT_MAX];
  char role[YAI_MESH_ROLE_MAX];
  char membership_state[YAI_MESH_MEMBERSHIP_STATE_MAX];
  int64_t discovered_at_epoch;
  int64_t last_seen_at_epoch;
  int connected;
} yai_mesh_peer_t;

typedef struct yai_mesh_peer_registry {
  yai_mesh_identity_t self;
  yai_mesh_peer_t peers[YAI_MESH_PEER_CAPACITY];
  size_t count;
} yai_mesh_peer_registry_t;

int yai_mesh_peer_registry_init(yai_mesh_peer_registry_t *registry,
                                const yai_mesh_identity_t *self_identity);
int yai_mesh_peer_registry_upsert(yai_mesh_peer_registry_t *registry,
                                  const char *peer_id,
                                  const char *endpoint,
                                  const char *role,
                                  int connected,
                                  int64_t observed_at_epoch);
const yai_mesh_peer_t *yai_mesh_peer_registry_find(const yai_mesh_peer_registry_t *registry,
                                                   const char *peer_id);
int yai_mesh_peer_registry_mark_seen(yai_mesh_peer_registry_t *registry,
                                     const char *peer_id,
                                     int64_t seen_at_epoch,
                                     int connected);
size_t yai_mesh_peer_registry_connected_count(const yai_mesh_peer_registry_t *registry);

#ifdef __cplusplus
}
#endif
