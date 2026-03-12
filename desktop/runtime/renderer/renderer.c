/* SPDX-License-Identifier: Apache-2.0 */
#include "yai/desktop/backend.h"
#include "yai/desktop/renderer.h"

int yai_desktop_renderer_draw(const yai_desktop_state_t *state)
{
  const yai_desktop_backend_ops_t *backend = yai_desktop_backend_current();
  if (!state || !backend || !backend->present_frame) return -1;
  return backend->present_frame(state);
}
