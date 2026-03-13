/* SPDX-License-Identifier: Apache-2.0 */
#include "internal.h"

static yai_desktop_backend_ops_t g_backend;
static const yai_desktop_backend_ops_t *g_active;

int yai_desktop_backend_register(const yai_desktop_backend_ops_t *ops)
{
  if (!ops) return -1;
  g_active = ops;
  return 0;
}

const yai_desktop_backend_ops_t *yai_desktop_backend_current(void)
{
  return g_active;
}

static int sdl_init(yai_desktop_state_t *state)
{
  (void)state;
  return 0;
}

static int sdl_poll(yai_desktop_input_event_t *event_out)
{
  if (!event_out) return -1;
  event_out->type = YAI_DESKTOP_INPUT_NONE;
  event_out->code = 0;
  event_out->value = 0;
  event_out->x = 0;
  event_out->y = 0;
  return 0;
}

static int sdl_present(const yai_desktop_state_t *state)
{
  (void)state;
  return 0;
}

static void sdl_shutdown(void)
{
}

int yai_desktop_backend_sdl_install(void)
{
  g_backend.init = sdl_init;
  g_backend.poll_input = sdl_poll;
  g_backend.present_frame = sdl_present;
  g_backend.shutdown = sdl_shutdown;
  return yai_desktop_backend_register(&g_backend);
}
