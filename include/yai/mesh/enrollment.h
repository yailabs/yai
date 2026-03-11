/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct yai_mesh_enrollment_state {
  uint32_t pending;
  uint32_t accepted;
  uint32_t rejected;
  int64_t updated_at_epoch;
} yai_mesh_enrollment_state_t;

int yai_mesh_enrollment_init(yai_mesh_enrollment_state_t *state, int64_t now_epoch);
int yai_mesh_enrollment_record(yai_mesh_enrollment_state_t *state,
                               int accepted,
                               int64_t now_epoch);

#ifdef __cplusplus
}
#endif
