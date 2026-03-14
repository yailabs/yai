/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct yai_mesh_containment_state {
  char mode[24];
  uint32_t quarantined_peers;
  int64_t updated_at_epoch;
} yai_mesh_containment_state_t;

int yai_mesh_containment_init(yai_mesh_containment_state_t *state, int64_t now_epoch);
int yai_mesh_containment_set_mode(yai_mesh_containment_state_t *state,
                                  const char *mode,
                                  uint32_t quarantined_peers,
                                  int64_t now_epoch);

#ifdef __cplusplus
}
#endif
