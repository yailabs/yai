/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <yai/network/providers/registry.h>

#ifdef __cplusplus
extern "C" {
#endif

int yai_mock_provider_create(yai_provider_t **provider_out);

#ifdef __cplusplus
}
#endif
