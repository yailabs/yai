/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <stdint.h>

#include <yai/network/discovery/peer_registry.h>

#ifdef __cplusplus
extern "C" {
#endif

#define YAI_MESH_WORKSPACE_ID_MAX 64

typedef struct yai_mesh_membership_state {
  char workspace_id[YAI_MESH_WORKSPACE_ID_MAX];
  uint32_t generation;
  uint32_t active_peers;
  int self_joined;
  int64_t updated_at_epoch;
} yai_mesh_membership_state_t;

int yai_mesh_membership_init(yai_mesh_membership_state_t *membership,
                             const char *workspace_id,
                             int64_t now_epoch);
int yai_mesh_membership_join_self(yai_mesh_membership_state_t *membership,
                                  int64_t now_epoch);
int yai_mesh_membership_refresh(yai_mesh_membership_state_t *membership,
                                const yai_mesh_peer_registry_t *registry,
                                int64_t now_epoch);

#ifdef __cplusplus
}
#endif
