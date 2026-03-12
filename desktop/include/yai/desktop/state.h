/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include "surface.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef struct yai_desktop_state {
  int running;
  yai_desktop_surface_id_t focused_surface;
  unsigned long frame_counter;
} yai_desktop_state_t;

void yai_desktop_state_init(yai_desktop_state_t *state);

#ifdef __cplusplus
}
#endif
