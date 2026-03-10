/* SPDX-License-Identifier: Apache-2.0 */

#include <yai/knowledge/providers.h>

static int g_providers_initialized = 0;
static yai_mind_provider_registry_t g_registry;

int yai_mind_providers_init(void)
{
  int rc;
  yai_mind_provider_t *mock = NULL;

  if (g_providers_initialized) return YAI_MIND_OK;

  rc = yai_mind_provider_registry_init(&g_registry);
  if (rc != YAI_MIND_OK) return rc;

  rc = yai_mind_mock_provider_create(&mock);
  if (rc != YAI_MIND_OK) {
    yai_mind_provider_registry_shutdown(&g_registry);
    return rc;
  }

  rc = yai_mind_provider_registry_register(&g_registry, mock, 1);
  if (rc != YAI_MIND_OK) {
    if (mock->vtable && mock->vtable->destroy) mock->vtable->destroy(mock);
    yai_mind_provider_registry_shutdown(&g_registry);
    return rc;
  }

  g_providers_initialized = 1;
  return YAI_MIND_OK;
}

int yai_mind_providers_shutdown(void)
{
  if (!g_providers_initialized) return YAI_MIND_OK;
  yai_mind_provider_registry_shutdown(&g_registry);
  g_providers_initialized = 0;
  return YAI_MIND_OK;
}

yai_mind_provider_registry_t *yai_mind_providers_registry(void)
{
  if (!g_providers_initialized) return NULL;
  return &g_registry;
}

int yai_knowledge_providers_start(void)
{
  return yai_mind_providers_init();
}

int yai_knowledge_providers_stop(void)
{
  return yai_mind_providers_shutdown();
}

yai_mind_provider_registry_t *yai_knowledge_provider_registry(void)
{
  return yai_mind_providers_registry();
}
