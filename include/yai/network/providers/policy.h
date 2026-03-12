/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <yai/network/providers/registry.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct yai_provider_policy {
  int allow_mock_providers;
  yai_provider_trust_level_t min_trust_for_embedding;
  yai_provider_trust_level_t min_trust_for_inference;
} yai_provider_policy_t;

void yai_provider_policy_default(yai_provider_policy_t *policy_out);
int yai_provider_set_policy(const yai_provider_policy_t *policy);
int yai_provider_get_policy(yai_provider_policy_t *policy_out);
int yai_provider_policy_admits(const yai_provider_descriptor_t *descriptor,
                               yai_provider_capability_t capability,
                               const yai_provider_policy_t *policy);

#ifdef __cplusplus
}
#endif
