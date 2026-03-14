/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <yai/cognition/memory.h>

#ifdef __cplusplus
extern "C" {
#endif

int yai_domain_semantic_put(const char *term,
                                 const char *definition,
                                 yai_node_id_t node_id);
int yai_domain_semantic_get(const char *term,
                                 yai_semantic_record_t *record_out);

#ifdef __cplusplus
}
#endif
