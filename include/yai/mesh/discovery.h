/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <stdint.h>

#include <yai/mesh/peer_registry.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct yai_mesh_discovery_event {
  char peer_id[YAI_MESH_PEER_MAX];
  char endpoint[YAI_MESH_ENDPOINT_MAX];
  char role[YAI_MESH_ROLE_MAX];
  int connected;
  int64_t observed_at_epoch;
} yai_mesh_discovery_event_t;

int yai_mesh_discovery_apply(yai_mesh_peer_registry_t *registry,
                             const yai_mesh_discovery_event_t *event);

#ifdef __cplusplus
}
#endif
