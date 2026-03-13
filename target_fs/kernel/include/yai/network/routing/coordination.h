#pragma once

#include <stdint.h>
#include <stddef.h>

#include <yai/network/discovery/discovery.h>

typedef struct {
  int self_joined;
} yai_mesh_membership_state_t;

typedef struct {
  char leader_peer_id[64];
  uint32_t participants;
  uint32_t quorum_size;
  int64_t updated_at_epoch;
} yai_mesh_coordination_state_t;

int yai_mesh_coordination_evaluate(yai_mesh_coordination_state_t *coordination,
                                   const yai_mesh_peer_registry_t *registry,
                                   const yai_mesh_membership_state_t *membership,
                                   int64_t now_epoch);
