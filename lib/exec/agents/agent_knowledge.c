/* SPDX-License-Identifier: Apache-2.0 */

#include "../../knowledge/cognition/cognition_internal.h"

#include <stdio.h>

int yai_mind_agent_knowledge_handle(const yai_mind_agent_frame_t *frame,
                                    yai_mind_cognition_response_t *response_out)
{
  yai_mind_memory_query_t query = {0};
  yai_mind_memory_result_t result = {0};
  int rc;

  if (!frame || !response_out) return YAI_MIND_ERR_INVALID_ARG;

  snprintf(query.workspace_id, sizeof(query.workspace_id), "%s",
           frame->request->session.workspace_id[0] ? frame->request->session.workspace_id : "default");
  snprintf(query.query, sizeof(query.query), "%.255s", frame->request->user_input);
  query.limit = 6;

  rc = yai_mind_memory_query_run(&query, &result);
  response_out->status = (rc == YAI_MIND_OK) ? 200 : 500;
  if (rc != YAI_MIND_OK) {
    snprintf(response_out->summary, sizeof(response_out->summary), "Knowledge agent query failed.");
    snprintf(response_out->output, sizeof(response_out->output), "memory_error=%d", rc);
    return rc;
  }

  snprintf(response_out->summary, sizeof(response_out->summary), "Knowledge agent retrieved memory context.");
  snprintf(response_out->output, sizeof(response_out->output), "matches=%d %s",
           result.match_count, result.summary);
  return YAI_MIND_OK;
}
