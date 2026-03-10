/* SPDX-License-Identifier: Apache-2.0 */

#include "../../knowledge/cognition/cognition_internal.h"

#include <stdio.h>
#include <string.h>

int yai_mind_agent_validator_handle(const yai_mind_agent_frame_t *frame,
                                    yai_mind_cognition_response_t *response_out)
{
  size_t len;
  if (!frame || !response_out) return YAI_MIND_ERR_INVALID_ARG;

  len = strlen(frame->request->user_input);
  response_out->status = 200;
  if (len < 8) {
    snprintf(response_out->summary, sizeof(response_out->summary), "Validator agent rejected weak input.");
    snprintf(response_out->output, sizeof(response_out->output), "validation=fail reason=input_too_short len=%zu", len);
    return YAI_MIND_OK;
  }

  snprintf(response_out->summary, sizeof(response_out->summary), "Validator agent accepted request.");
  snprintf(response_out->output, sizeof(response_out->output), "validation=pass len=%zu", len);
  return YAI_MIND_OK;
}
