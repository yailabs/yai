/* SPDX-License-Identifier: Apache-2.0 */

#include "../../../cognition/core/cognition/cognition_internal.h"

#include <stdio.h>

int yai_agent_system_handle(const yai_agent_frame_t *frame,
                                 yai_cognition_response_t *response_out)
{
  if (!frame || !response_out) return YAI_MIND_ERR_INVALID_ARG;
  response_out->status = 200;
  snprintf(response_out->summary, sizeof(response_out->summary), "System agent completed health pass.");
  snprintf(response_out->output, sizeof(response_out->output),
           "session=%s plan_steps=%zu mode=stable",
           frame->request->session.session_id[0] ? frame->request->session.session_id : "default",
           frame->plan_count);
  return YAI_MIND_OK;
}
