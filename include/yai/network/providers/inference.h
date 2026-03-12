/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <yai/network/providers/selection.h>

#ifdef __cplusplus
extern "C" {
#endif

int yai_client_completion(const char *provider_name,
                          const char *payload,
                          yai_provider_response_t *response_out);

#ifdef __cplusplus
}
#endif
