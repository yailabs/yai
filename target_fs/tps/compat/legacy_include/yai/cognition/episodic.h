/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <yai/cognition/memory.h>

#ifdef __cplusplus
extern "C" {
#endif

int yai_domain_episodic_append(const char *episode_id,
                                    yai_node_id_t node_id,
                                    const char *summary);
int yai_domain_episodic_latest(yai_episodic_record_t *record_out);

#ifdef __cplusplus
}
#endif
