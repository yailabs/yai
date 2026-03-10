/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <yai/knowledge/memory.h>

#ifdef __cplusplus
extern "C" {
#endif

int yai_mind_domain_activation_record(yai_mind_node_id_t node_id,
                                      float score,
                                      const char *source);
int yai_mind_domain_activation_last(yai_mind_activation_record_t *record_out,
                                    yai_mind_activation_trace_t *trace_out);

#ifdef __cplusplus
}
#endif
