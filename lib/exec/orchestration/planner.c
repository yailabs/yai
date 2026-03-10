/* SPDX-License-Identifier: Apache-2.0 */

#include "../../knowledge/cognition/cognition_internal.h"

#include <stdio.h>
#include <string.h>

int yai_mind_orchestration_plan(const yai_mind_cognition_request_t *request,
                                yai_mind_plan_step_t *steps_out,
                                size_t steps_cap,
                                size_t *steps_count_out)
{
  size_t n = 0;
  if (!request || !steps_out || steps_cap == 0 || !steps_count_out) return YAI_MIND_ERR_INVALID_ARG;

  snprintf(steps_out[n].step_id, sizeof(steps_out[n].step_id), "plan-01");
  snprintf(steps_out[n].action, sizeof(steps_out[n].action), "classify-role");
  steps_out[n].priority = 100;
  n++;

  if (n < steps_cap) {
    snprintf(steps_out[n].step_id, sizeof(steps_out[n].step_id), "plan-02");
    snprintf(steps_out[n].action, sizeof(steps_out[n].action), "build-context");
    steps_out[n].priority = 80;
    n++;
  }

  if (request->user_input[0] && n < steps_cap) {
    snprintf(steps_out[n].step_id, sizeof(steps_out[n].step_id), "plan-03");
    snprintf(steps_out[n].action, sizeof(steps_out[n].action), "provider-dispatch");
    steps_out[n].priority = 70;
    n++;
  }

  *steps_count_out = n;
  return YAI_MIND_OK;
}
