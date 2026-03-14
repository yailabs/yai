/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <yai/cognition/memory.h>

#ifdef __cplusplus
extern "C" {
#endif

int yai_domain_activation_record(yai_node_id_t node_id,
                                      float score,
                                      const char *source);
int yai_domain_activation_last(yai_activation_record_t *record_out,
                                    yai_activation_trace_t *trace_out);

#ifdef __cplusplus
}
#endif
