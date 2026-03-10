/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <yai/knowledge/cognition.h>

#ifdef __cplusplus
extern "C" {
#endif

int yai_mind_orchestration_plan(const yai_mind_cognition_request_t *request,
                                yai_mind_plan_step_t *steps_out,
                                size_t steps_cap,
                                size_t *steps_count_out);
void yai_mind_rag_sessions_reset(void);

#ifdef __cplusplus
}
#endif
