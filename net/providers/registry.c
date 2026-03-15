/* SPDX-License-Identifier: Apache-2.0 */

#include <yai/net/providers/registry.h>

#include <string.h>

static yai_provider_descriptor_t yai_default_descriptor_for(const yai_provider_t *provider)
{
  yai_provider_descriptor_t descriptor;
  memset(&descriptor, 0, sizeof(descriptor));
  descriptor.provider_id = provider ? provider->name : NULL;
  descriptor.provider_class = YAI_PROVIDER_CLASS_HYBRID;
  descriptor.capability_mask = YAI_PROVIDER_CAPABILITY_EMBEDDING | YAI_PROVIDER_CAPABILITY_INFERENCE;
  descriptor.trust_level = YAI_PROVIDER_TRUST_SANDBOXED;
  descriptor.is_mock = 0;
  descriptor.invocation_mode = "sync";
  return descriptor;
}

int yai_provider_registry_init(yai_provider_registry_t *registry)
{
  if (!registry) return YAI_MIND_ERR_INVALID_ARG;
  memset(registry, 0, sizeof(*registry));
  return YAI_MIND_OK;
}

int yai_provider_registry_register_with_descriptor(yai_provider_registry_t *registry,
                                                   yai_provider_t *provider,
                                                   const yai_provider_descriptor_t *descriptor,
                                                   int make_default)
{
  size_t i;
  yai_provider_descriptor_t normalized;

  if (!registry || !provider || !provider->name || !provider->vtable) return YAI_MIND_ERR_INVALID_ARG;

  for (i = 0; i < registry->count; i++) {
    yai_provider_t *existing = registry->providers[i];
    if (existing && existing->name && strcmp(existing->name, provider->name) == 0) {
      return YAI_MIND_ERR_STATE;
    }
  }

  if (registry->count >= (sizeof(registry->providers) / sizeof(registry->providers[0]))) {
    return YAI_MIND_ERR_STATE;
  }

  normalized = descriptor ? *descriptor : yai_default_descriptor_for(provider);
  if (!normalized.provider_id || !normalized.provider_id[0]) normalized.provider_id = provider->name;
  if (!normalized.capability_mask) {
    normalized.capability_mask = YAI_PROVIDER_CAPABILITY_EMBEDDING | YAI_PROVIDER_CAPABILITY_INFERENCE;
  }
  if (!normalized.invocation_mode || !normalized.invocation_mode[0]) normalized.invocation_mode = "sync";

  registry->providers[registry->count] = provider;
  registry->descriptors[registry->count] = normalized;
  registry->count++;

  if (make_default || !registry->default_provider) {
    registry->default_provider = provider;
  }

  return YAI_MIND_OK;
}

int yai_provider_registry_register(yai_provider_registry_t *registry,
                                   yai_provider_t *provider,
                                   int make_default)
{
  yai_provider_descriptor_t descriptor = yai_default_descriptor_for(provider);
  return yai_provider_registry_register_with_descriptor(registry, provider, &descriptor, make_default);
}

yai_provider_t *yai_provider_registry_get(yai_provider_registry_t *registry,
                                          const char *name)
{
  size_t i;
  if (!registry || !name || !name[0]) return NULL;

  for (i = 0; i < registry->count; i++) {
    yai_provider_t *provider = registry->providers[i];
    if (provider && provider->name && strcmp(provider->name, name) == 0) return provider;
  }

  return NULL;
}

const yai_provider_descriptor_t *yai_provider_registry_get_descriptor(yai_provider_registry_t *registry,
                                                                      const char *name)
{
  size_t i;
  if (!registry || !name || !name[0]) return NULL;

  for (i = 0; i < registry->count; i++) {
    yai_provider_t *provider = registry->providers[i];
    if (provider && provider->name && strcmp(provider->name, name) == 0) {
      return &registry->descriptors[i];
    }
  }

  return NULL;
}

const yai_provider_descriptor_t *yai_provider_registry_descriptor_for(yai_provider_registry_t *registry,
                                                                      const yai_provider_t *provider)
{
  size_t i;
  if (!registry || !provider) return NULL;

  for (i = 0; i < registry->count; i++) {
    if (registry->providers[i] == provider) return &registry->descriptors[i];
  }

  return NULL;
}

yai_provider_t *yai_provider_registry_default(yai_provider_registry_t *registry)
{
  if (!registry) return NULL;
  return registry->default_provider;
}

void yai_provider_registry_shutdown(yai_provider_registry_t *registry)
{
  size_t i;
  if (!registry) return;

  for (i = 0; i < registry->count; i++) {
    yai_provider_t *provider = registry->providers[i];
    if (provider && provider->vtable && provider->vtable->destroy) {
      provider->vtable->destroy(provider);
    }
    registry->providers[i] = NULL;
    memset(&registry->descriptors[i], 0, sizeof(registry->descriptors[i]));
  }

  registry->count = 0;
  registry->default_provider = NULL;
}

int yai_provider_completion(yai_provider_t *provider,
                            const yai_provider_request_t *request,
                            yai_provider_response_t *response)
{
  if (!provider || !provider->vtable || !provider->vtable->completion) {
    return YAI_MIND_ERR_PROVIDER;
  }
  return provider->vtable->completion(provider, request, response);
}
