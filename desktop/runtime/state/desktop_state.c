/* SPDX-License-Identifier: Apache-2.0 */
#include "yai/desktop/state.h"

void yai_desktop_state_init(yai_desktop_state_t *state)
{
  if (!state) return;
  state->running = 1;
  state->focused_surface = 0;
  state->frame_counter = 0;
}
