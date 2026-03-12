/* SPDX-License-Identifier: Apache-2.0 */

#include "../../../cognition/core/cognition/cognition_internal.h"
#include <yai/network/providers/catalog.h>

#include <stdio.h>

int yai_agent_code_handle(const yai_agent_frame_t *frame,
                               yai_cognition_response_t *response_out)
{
  yai_provider_response_t provider_response = {0};
  int rc;
  if (!frame || !response_out || !frame->prompt) return YAI_MIND_ERR_INVALID_ARG;

  rc = yai_client_completion(frame->request->provider_name, frame->prompt, &provider_response);
  response_out->status = (rc == YAI_MIND_OK) ? 200 : 500;
  if (rc == YAI_MIND_OK) {
    snprintf(response_out->summary, sizeof(response_out->summary), "Code agent completed provider cycle.");
    snprintf(response_out->output, sizeof(response_out->output), "%s", provider_response.output);
    return YAI_MIND_OK;
  }

  snprintf(response_out->summary, sizeof(response_out->summary), "Code agent provider call failed.");
  snprintf(response_out->output, sizeof(response_out->output), "provider_error=%d", rc);
  return rc;
}
