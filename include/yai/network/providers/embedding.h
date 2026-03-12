/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <yai/network/providers/selection.h>

#ifdef __cplusplus
extern "C" {
#endif

int yai_client_embedding(const char *provider_name,
                         const char *text,
                         float *vector_out,
                         size_t vector_dim);

#ifdef __cplusplus
}
#endif
