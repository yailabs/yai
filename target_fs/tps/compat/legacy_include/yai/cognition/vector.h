/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <yai/cognition/memory.h>

#ifdef __cplusplus
extern "C" {
#endif

int yai_domain_vector_upsert(yai_node_id_t node_id,
                                  const float *values,
                                  size_t dim);
int yai_domain_vector_nearest(const float *values,
                                   size_t dim,
                                   yai_node_id_t *node_id_out,
                                   float *distance_out);

#ifdef __cplusplus
}
#endif
