/* SPDX-License-Identifier: Apache-2.0 */

#include <yai/core/lifecycle.h>
#include <yai/knowledge/runtime.h>

#include <stdio.h>
#include <string.h>

static yai_mind_runtime_t g_runtime = {0};

static void clear_runtime(void)
{
  memset(&g_runtime, 0, sizeof(g_runtime));
}

int yai_mind_init(const yai_mind_config_t *config)
{
  if (g_runtime.initialized) return YAI_MIND_OK;
  clear_runtime();
  if (config) g_runtime.config = *config;

  if (yai_runtime_capabilities_start("system",
                                     g_runtime.config.runtime_name,
                                     g_runtime.config.enable_mock_provider,
                                     NULL,
                                     0) != 0) {
    clear_runtime();
    return YAI_MIND_ERR_STATE;
  }

  {
    const yai_runtime_capability_state_t *caps = yai_runtime_capabilities_state();
    g_runtime.initialized = caps->initialized;
    g_runtime.transport_ready = caps->transport_ready;
    g_runtime.providers_ready = caps->providers_ready;
    g_runtime.memory_ready = caps->memory_ready;
    g_runtime.cognition_ready = caps->cognition_ready;
  }
  return YAI_MIND_OK;
}

int yai_mind_shutdown(void)
{
  if (!g_runtime.initialized) return YAI_MIND_OK;
  (void)yai_runtime_capabilities_stop(NULL, 0);
  clear_runtime();
  return YAI_MIND_OK;
}

int yai_mind_is_initialized(void)
{
  return yai_runtime_capabilities_is_ready();
}

const yai_mind_runtime_t *yai_mind_runtime_state(void)
{
  const yai_runtime_capability_state_t *caps = yai_runtime_capabilities_state();
  g_runtime.initialized = caps->initialized;
  g_runtime.transport_ready = caps->transport_ready;
  g_runtime.providers_ready = caps->providers_ready;
  g_runtime.memory_ready = caps->memory_ready;
  g_runtime.cognition_ready = caps->cognition_ready;
  g_runtime.config.runtime_name = caps->runtime_name;
  g_runtime.config.enable_mock_provider = caps->enable_mock_provider;
  return &g_runtime;
}
