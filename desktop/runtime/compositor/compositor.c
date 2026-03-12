/* SPDX-License-Identifier: Apache-2.0 */
#include "yai/desktop/compositor.h"

int yai_desktop_compositor_compose(yai_desktop_state_t *state)
{
  if (!state) return -1;
  state->frame_counter += 1;
  return 0;
}
