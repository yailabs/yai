/* SPDX-License-Identifier: Apache-2.0 */

#include <yai/exec/transport.h>

static int g_transport_initialized = 0;

int yai_mind_transport_init(void)
{
  if (g_transport_initialized) return YAI_MIND_OK;
  g_transport_initialized = 1;
  return YAI_MIND_OK;
}

int yai_mind_transport_shutdown(void)
{
  if (!g_transport_initialized) return YAI_MIND_OK;
  g_transport_initialized = 0;
  return YAI_MIND_OK;
}

int yai_mind_transport_is_initialized(void)
{
  return g_transport_initialized;
}

int yai_exec_transport_start(void)
{
  return yai_mind_transport_init();
}

int yai_exec_transport_stop(void)
{
  return yai_mind_transport_shutdown();
}

int yai_exec_transport_is_ready(void)
{
  return yai_mind_transport_is_initialized();
}
