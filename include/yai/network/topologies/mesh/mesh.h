/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <stdint.h>

#include <yai/network/topology/registry.h>

#ifdef __cplusplus
extern "C" {
#endif

#define YAI_MESH_FRESHNESS_UNKNOWN "unknown"
#define YAI_MESH_FRESHNESS_FRESH "fresh"
#define YAI_MESH_FRESHNESS_AGING "aging"
#define YAI_MESH_FRESHNESS_STALE "stale"

typedef struct yai_mesh_awareness_state {
  uint32_t peers_total;
  uint32_t peers_connected;
  uint32_t peers_stale;
  char freshness_state[16];
  int64_t updated_at_epoch;
} yai_mesh_awareness_state_t;

int yai_network_sovereign_overlay_topology_ready(void);

#ifdef __cplusplus
}
#endif
