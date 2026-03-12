/* SPDX-License-Identifier: Apache-2.0 */

#include <yai/cognition/memory.h>
#include <yai/graph/internal/backend.h>
#include <yai/graph/internal/state_reset.h>

static int g_memory_initialized = 0;

int yai_memory_init(void)
{
  int rc;
  if (g_memory_initialized) return YAI_MIND_OK;

  rc = yai_graph_backend_select_inmemory();
  if (rc != YAI_MIND_OK) return rc;
  yai_graph_state_reset();

  g_memory_initialized = 1;
  return YAI_MIND_OK;
}

int yai_memory_shutdown(void)
{
  if (!g_memory_initialized) return YAI_MIND_OK;
  yai_graph_state_reset();
  yai_graph_backend_shutdown_internal();
  g_memory_initialized = 0;
  return YAI_MIND_OK;
}

int yai_knowledge_memory_start(void)
{
  return yai_memory_init();
}

int yai_knowledge_memory_stop(void)
{
  return yai_memory_shutdown();
}
