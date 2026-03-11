/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <stdint.h>

#include <yai/mesh/membership.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct yai_mesh_coordination_state {
  char leader_peer_id[YAI_MESH_PEER_MAX];
  char mode[32];
  uint32_t quorum_size;
  uint32_t participants;
  int64_t updated_at_epoch;
} yai_mesh_coordination_state_t;

int yai_mesh_coordination_evaluate(yai_mesh_coordination_state_t *coordination,
                                   const yai_mesh_peer_registry_t *registry,
                                   const yai_mesh_membership_state_t *membership,
                                   int64_t now_epoch);

#ifdef __cplusplus
}
#endif
