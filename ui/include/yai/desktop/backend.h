/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include "input.h"
#include "state.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef struct yai_desktop_backend_ops {
  int (*init)(yai_desktop_state_t *state);
  int (*poll_input)(yai_desktop_input_event_t *event_out);
  int (*present_frame)(const yai_desktop_state_t *state);
  void (*shutdown)(void);
} yai_desktop_backend_ops_t;

int yai_desktop_backend_register(const yai_desktop_backend_ops_t *ops);
const yai_desktop_backend_ops_t *yai_desktop_backend_current(void);

#ifdef __cplusplus
}
#endif
