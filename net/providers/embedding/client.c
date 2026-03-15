/* SPDX-License-Identifier: Apache-2.0 */

#include <yai/net/providers/embedding.h>

static int resolve_embedding_provider(const char *provider_name, yai_provider_t **provider_out)
{
  yai_provider_registry_t *registry = yai_knowledge_provider_registry();
  yai_provider_selection_request_t request = {
    .provider_name = provider_name,
    .capability = YAI_PROVIDER_CAPABILITY_EMBEDDING,
    .min_trust = YAI_PROVIDER_TRUST_SANDBOXED,
    .allow_mock_override = 0,
  };

  if (!registry || !provider_out) return YAI_MIND_ERR_PROVIDER;
  return yai_provider_select(registry, &request, NULL, provider_out);
}

int yai_client_embedding(const char *provider_name,
                         const char *text,
                         float *vector_out,
                         size_t vector_dim)
{
  yai_provider_t *provider = NULL;
  int rc;

  if (!text || !text[0] || !vector_out || vector_dim == 0) return YAI_MIND_ERR_INVALID_ARG;

  rc = resolve_embedding_provider(provider_name, &provider);
  if (rc != YAI_MIND_OK || !provider || !provider->vtable || !provider->vtable->embedding) {
    return YAI_MIND_ERR_PROVIDER;
  }

  return provider->vtable->embedding(provider, text, vector_out, vector_dim);
}
