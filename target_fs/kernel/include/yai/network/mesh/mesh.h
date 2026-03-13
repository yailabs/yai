#pragma once

#include <stdint.h>

#include <yai/network/discovery/discovery.h>
#include <yai/network/discovery/enrollment.h>
#include <yai/network/routing/conflict.h>
#include <yai/network/routing/coordination.h>
#include <yai/network/routing/replay.h>
#include <yai/network/transport/client.h>
#include <yai/network/transport/overlay.h>

typedef struct {
  int initialized;
  yai_mesh_peer_registry_t registry;
  yai_mesh_enrollment_state_t enrollment;
  yai_mesh_conflict_state_t conflict;
  yai_mesh_coordination_state_t coordination;
  yai_mesh_replay_state_t replay;
  yai_mesh_transport_state_t transport;
} yai_mesh_state_t;
