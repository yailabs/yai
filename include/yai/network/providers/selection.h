/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <yai/network/providers/policy.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct yai_provider_selection_request {
  const char *provider_name;
  yai_provider_capability_t capability;
  yai_provider_trust_level_t min_trust;
  int allow_mock_override;
} yai_provider_selection_request_t;

int yai_provider_select(yai_provider_registry_t *registry,
                        const yai_provider_selection_request_t *request,
                        const yai_provider_policy_t *policy,
                        yai_provider_t **provider_out);

#ifdef __cplusplus
}
#endif
