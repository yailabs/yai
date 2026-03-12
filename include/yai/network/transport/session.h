/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <stdint.h>

#include <yai/network/discovery/peer_registry.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct yai_mesh_transport_state {
  char reachability_state[24];
  uint32_t path_count;
  uint32_t degraded_paths;
  int64_t updated_at_epoch;
} yai_mesh_transport_state_t;

int yai_mesh_transport_refresh(yai_mesh_transport_state_t *transport,
                               const yai_mesh_peer_registry_t *registry,
                               int64_t now_epoch);

#ifdef __cplusplus
}
#endif
