#pragma once

#include <yai/cognition/cognition.h>

#ifdef __cplusplus
extern "C" {
#endif

int yai_orchestration_plan(const yai_cognition_request_t *request,
                                yai_plan_step_t *steps_out,
                                size_t steps_cap,
                                size_t *steps_count_out);

#ifdef __cplusplus
}
#endif
