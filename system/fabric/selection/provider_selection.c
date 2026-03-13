/* SPDX-License-Identifier: Apache-2.0 */

#include <yai/network/providers/selection.h>

static yai_provider_selection_request_t yai_normalize_request(const yai_provider_selection_request_t *request)
{
  yai_provider_selection_request_t normalized = {0};
  if (request) normalized = *request;
  if (!normalized.capability) normalized.capability = YAI_PROVIDER_CAPABILITY_INFERENCE;
  return normalized;
}

static int yai_select_named_provider(yai_provider_registry_t *registry,
                                     const yai_provider_selection_request_t *request,
                                     const yai_provider_policy_t *policy,
                                     yai_provider_t **provider_out)
{
  yai_provider_t *named = yai_provider_registry_get(registry, request->provider_name);
  const yai_provider_descriptor_t *descriptor = named ? yai_provider_registry_descriptor_for(registry, named) : NULL;
  if (!named || !descriptor) return YAI_MIND_ERR_NOT_FOUND;
  if (descriptor->trust_level < request->min_trust) return YAI_MIND_ERR_PROVIDER;

  if (request->allow_mock_override) {
    yai_provider_policy_t override_policy = *policy;
    override_policy.allow_mock_providers = 1;
    if (!yai_provider_policy_admits(descriptor, request->capability, &override_policy)) {
      return YAI_MIND_ERR_PROVIDER;
    }
  } else if (!yai_provider_policy_admits(descriptor, request->capability, policy)) {
    return YAI_MIND_ERR_PROVIDER;
  }

  *provider_out = named;
  return YAI_MIND_OK;
}

int yai_provider_select(yai_provider_registry_t *registry,
                        const yai_provider_selection_request_t *request,
                        const yai_provider_policy_t *policy,
                        yai_provider_t **provider_out)
{
  yai_provider_selection_request_t normalized;
  yai_provider_policy_t loaded_policy;
  const yai_provider_policy_t *effective_policy = policy;
  size_t i;

  if (!registry || !provider_out) return YAI_MIND_ERR_INVALID_ARG;
  *provider_out = NULL;

  normalized = yai_normalize_request(request);

  if (!effective_policy) {
    if (yai_provider_get_policy(&loaded_policy) != YAI_MIND_OK) return YAI_MIND_ERR_STATE;
    effective_policy = &loaded_policy;
  }

  if (normalized.provider_name && normalized.provider_name[0]) {
    return yai_select_named_provider(registry, &normalized, effective_policy, provider_out);
  }

  for (i = 0; i < registry->count; i++) {
    yai_provider_t *candidate = registry->providers[i];
    const yai_provider_descriptor_t *descriptor = yai_provider_registry_descriptor_for(registry, candidate);
    if (!candidate || !descriptor) continue;
    if (descriptor->trust_level < normalized.min_trust) continue;
    if (yai_provider_policy_admits(descriptor, normalized.capability, effective_policy)) {
      *provider_out = candidate;
      return YAI_MIND_OK;
    }
  }

  return YAI_MIND_ERR_PROVIDER;
}
