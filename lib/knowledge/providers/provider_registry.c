/* SPDX-License-Identifier: Apache-2.0 */

#include <yai/knowledge/providers.h>

#include <string.h>

int yai_mind_provider_registry_init(yai_mind_provider_registry_t *registry)
{
  if (!registry) return YAI_MIND_ERR_INVALID_ARG;
  memset(registry, 0, sizeof(*registry));
  return YAI_MIND_OK;
}

int yai_mind_provider_registry_register(yai_mind_provider_registry_t *registry,
                                        yai_mind_provider_t *provider,
                                        int make_default)
{
  size_t i;
  if (!registry || !provider || !provider->name || !provider->vtable) return YAI_MIND_ERR_INVALID_ARG;
  for (i = 0; i < registry->count; i++) {
    yai_mind_provider_t *existing = registry->providers[i];
    if (existing && existing->name && strcmp(existing->name, provider->name) == 0) {
      return YAI_MIND_ERR_STATE;
    }
  }
  if (registry->count >= (sizeof(registry->providers) / sizeof(registry->providers[0]))) {
    return YAI_MIND_ERR_STATE;
  }
  registry->providers[registry->count++] = provider;
  if (make_default || !registry->default_provider) {
    registry->default_provider = provider;
  }
  return YAI_MIND_OK;
}

yai_mind_provider_t *yai_mind_provider_registry_get(yai_mind_provider_registry_t *registry,
                                                     const char *name)
{
  size_t i;
  if (!registry || !name || !name[0]) return NULL;
  for (i = 0; i < registry->count; i++) {
    yai_mind_provider_t *provider = registry->providers[i];
    if (provider && provider->name && strcmp(provider->name, name) == 0) return provider;
  }
  return NULL;
}

yai_mind_provider_t *yai_mind_provider_registry_default(yai_mind_provider_registry_t *registry)
{
  if (!registry) return NULL;
  return registry->default_provider;
}

void yai_mind_provider_registry_shutdown(yai_mind_provider_registry_t *registry)
{
  size_t i;
  if (!registry) return;
  for (i = 0; i < registry->count; i++) {
    yai_mind_provider_t *provider = registry->providers[i];
    if (provider && provider->vtable && provider->vtable->destroy) {
      provider->vtable->destroy(provider);
    }
    registry->providers[i] = NULL;
  }
  registry->count = 0;
  registry->default_provider = NULL;
}

int yai_mind_provider_completion(yai_mind_provider_t *provider,
                                 const yai_mind_provider_request_t *request,
                                 yai_mind_provider_response_t *response)
{
  if (!provider || !provider->vtable || !provider->vtable->completion) {
    return YAI_MIND_ERR_PROVIDER;
  }
  return provider->vtable->completion(provider, request, response);
}
