/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <stdint.h>

#include <yai/network/discovery/peer_registry.h>

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

int yai_mesh_awareness_refresh(yai_mesh_awareness_state_t *awareness,
                               const yai_mesh_peer_registry_t *registry,
                               int64_t now_epoch,
                               int64_t aging_after_sec,
                               int64_t stale_after_sec);

#ifdef __cplusplus
}
#endif
