/* SPDX-License-Identifier: Apache-2.0 */
#include "yai/desktop/backend.h"
#include "yai/desktop/compositor.h"
#include "yai/desktop/desktop.h"
#include "yai/desktop/renderer.h"

int yai_desktop_run(yai_desktop_session_t *session)
{
  yai_desktop_state_t state;
  yai_desktop_input_event_t ev;
  const yai_desktop_backend_ops_t *backend = yai_desktop_backend_current();
  if (!session || !backend || !backend->init) return -1;
  if (yai_desktop_session_start(session) != 0) return -1;
  yai_desktop_state_init(&state);
  if (backend->init(&state) != 0) return -1;

  while (state.running) {
    if (backend->poll_input && backend->poll_input(&ev) == 0) {
      if (ev.type == YAI_DESKTOP_INPUT_KEY && ev.code == 27) state.running = 0;
    }
    if (yai_desktop_compositor_compose(&state) != 0) break;
    if (yai_desktop_renderer_draw(&state) != 0) break;
    if (state.frame_counter > 2) state.running = 0;
  }

  if (backend->shutdown) backend->shutdown();
  return yai_desktop_session_stop(session);
}
