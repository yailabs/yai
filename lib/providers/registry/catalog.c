/* SPDX-License-Identifier: Apache-2.0 */

#include <yai/providers/catalog.h>

static int g_providers_initialized = 0;
static yai_provider_registry_t g_registry;

int yai_providers_init(void)
{
  int rc;
  yai_provider_t *mock = NULL;
  yai_provider_descriptor_t mock_descriptor = {
    .provider_id = "mock",
    .provider_class = YAI_PROVIDER_CLASS_HYBRID,
    .capability_mask = YAI_PROVIDER_CAPABILITY_EMBEDDING | YAI_PROVIDER_CAPABILITY_INFERENCE,
    .trust_level = YAI_PROVIDER_TRUST_SANDBOXED,
    .is_mock = 1,
    .invocation_mode = "sync",
  };

  if (g_providers_initialized) return YAI_MIND_OK;

  rc = yai_provider_registry_init(&g_registry);
  if (rc != YAI_MIND_OK) return rc;

  rc = yai_mock_provider_create(&mock);
  if (rc != YAI_MIND_OK) {
    yai_provider_registry_shutdown(&g_registry);
    return rc;
  }

  rc = yai_provider_registry_register_with_descriptor(&g_registry, mock, &mock_descriptor, 1);
  if (rc != YAI_MIND_OK) {
    if (mock->vtable && mock->vtable->destroy) mock->vtable->destroy(mock);
    yai_provider_registry_shutdown(&g_registry);
    return rc;
  }

  g_providers_initialized = 1;
  return YAI_MIND_OK;
}

int yai_providers_shutdown(void)
{
  if (!g_providers_initialized) return YAI_MIND_OK;
  yai_provider_registry_shutdown(&g_registry);
  g_providers_initialized = 0;
  return YAI_MIND_OK;
}

yai_provider_registry_t *yai_providers_registry(void)
{
  if (!g_providers_initialized) return NULL;
  return &g_registry;
}

int yai_knowledge_providers_start(void)
{
  return yai_providers_init();
}

int yai_knowledge_providers_stop(void)
{
  return yai_providers_shutdown();
}

yai_provider_registry_t *yai_knowledge_provider_registry(void)
{
  return yai_providers_registry();
}
