/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <yai/network/topologies/mesh/mesh.h>

#ifdef __cplusplus
extern "C" {
#endif

int yai_mesh_awareness_refresh(yai_mesh_awareness_state_t *awareness,
                               const yai_mesh_peer_registry_t *registry,
                               int64_t now_epoch,
                               int64_t aging_after_sec,
                               int64_t stale_after_sec);

#ifdef __cplusplus
}
#endif
