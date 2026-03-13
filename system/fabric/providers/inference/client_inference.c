/* SPDX-License-Identifier: Apache-2.0 */

#include <yai/network/providers/inference.h>

#include <stdio.h>
#include <string.h>

static int resolve_inference_provider(const char *provider_name, yai_provider_t **provider_out)
{
  yai_provider_registry_t *registry = yai_knowledge_provider_registry();
  yai_provider_selection_request_t request = {
    .provider_name = provider_name,
    .capability = YAI_PROVIDER_CAPABILITY_INFERENCE,
    .min_trust = YAI_PROVIDER_TRUST_SANDBOXED,
    .allow_mock_override = 0,
  };

  if (!registry || !provider_out) return YAI_MIND_ERR_PROVIDER;
  return yai_provider_select(registry, &request, NULL, provider_out);
}

int yai_client_completion(const char *provider_name,
                          const char *payload,
                          yai_provider_response_t *response_out)
{
  yai_provider_t *provider = NULL;
  yai_provider_request_t req;
  int rc;

  if (!payload || !payload[0] || !response_out) return YAI_MIND_ERR_INVALID_ARG;

  rc = resolve_inference_provider(provider_name, &provider);
  if (rc != YAI_MIND_OK || !provider) return YAI_MIND_ERR_PROVIDER;

  memset(&req, 0, sizeof(req));
  snprintf(req.request_id, sizeof(req.request_id), "providers-inference");
  snprintf(req.model, sizeof(req.model), "%s", provider->name ? provider->name : "default");
  snprintf(req.payload, sizeof(req.payload), "%.511s", payload);

  return yai_provider_completion(provider, &req, response_out);
}
