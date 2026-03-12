/* SPDX-License-Identifier: Apache-2.0 */

#include <yai/network/providers/policy.h>

#include <string.h>

static yai_provider_policy_t g_provider_policy = {
  .allow_mock_providers = 1,
  .min_trust_for_embedding = YAI_PROVIDER_TRUST_SANDBOXED,
  .min_trust_for_inference = YAI_PROVIDER_TRUST_SANDBOXED,
};

void yai_provider_policy_default(yai_provider_policy_t *policy_out)
{
  if (!policy_out) return;
  *policy_out = g_provider_policy;
}

int yai_provider_set_policy(const yai_provider_policy_t *policy)
{
  if (!policy) return YAI_MIND_ERR_INVALID_ARG;
  g_provider_policy = *policy;
  return YAI_MIND_OK;
}

int yai_provider_get_policy(yai_provider_policy_t *policy_out)
{
  if (!policy_out) return YAI_MIND_ERR_INVALID_ARG;
  *policy_out = g_provider_policy;
  return YAI_MIND_OK;
}

int yai_provider_policy_admits(const yai_provider_descriptor_t *descriptor,
                               yai_provider_capability_t capability,
                               const yai_provider_policy_t *policy)
{
  const yai_provider_policy_t *effective_policy = policy ? policy : &g_provider_policy;
  yai_provider_trust_level_t min_trust;

  if (!descriptor) return 0;
  if ((descriptor->capability_mask & (unsigned int)capability) == 0u) return 0;

  min_trust = effective_policy->min_trust_for_inference;
  if (capability == YAI_PROVIDER_CAPABILITY_EMBEDDING) {
    min_trust = effective_policy->min_trust_for_embedding;
  }

  if (!effective_policy->allow_mock_providers && descriptor->is_mock) return 0;
  if (descriptor->trust_level < min_trust) return 0;

  return 1;
}
