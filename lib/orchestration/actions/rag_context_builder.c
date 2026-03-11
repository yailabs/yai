/* SPDX-License-Identifier: Apache-2.0 */

#include "../../knowledge/cognition/cognition_internal.h"
#include <yai/orchestration/grounding.h>

#include <stdio.h>
#include <string.h>

int yai_rag_build_context(yai_rag_session_state_t *session_state,
                               const yai_cognition_request_t *request,
                               char **context_out)
{
  yai_memory_query_t query = {0};
  yai_memory_result_t result = {0};
  char governed_json[6144];
  char grounding_reason[128];
  char *buf;
  int rc;

  if (!session_state || !request || !context_out) return YAI_MIND_ERR_INVALID_ARG;

  buf = (char *)yai_arena_alloc(&session_state->arena, 4096, 16);
  if (!buf) return YAI_MIND_ERR_NO_MEMORY;

  snprintf(query.workspace_id, sizeof(query.workspace_id), "%s",
           request->session.workspace_id[0] ? request->session.workspace_id : "default");
  snprintf(query.query, sizeof(query.query), "%.255s", request->user_input);
  query.limit = 8;

  rc = yai_memory_query_run(&query, &result);
  governed_json[0] = '\0';
  grounding_reason[0] = '\0';
  if (yai_exec_grounding_context_build(query.workspace_id,
                                       request->task.prompt,
                                       request->user_input,
                                       governed_json,
                                       sizeof(governed_json),
                                       grounding_reason,
                                       sizeof(grounding_reason)) != 0) {
    snprintf(governed_json,
             sizeof(governed_json),
             "{\"workspace_id\":\"%s\",\"grounding_contract\":\"governed_case_state_v1\",\"status\":\"degraded\",\"reason\":\"%s\"}",
             query.workspace_id,
             grounding_reason[0] ? grounding_reason : "grounding_unavailable");
  }

  if (rc != YAI_MIND_OK) {
    snprintf(buf, 4096, "Context unavailable. query_error=%d Grounding=%s", rc, governed_json);
  } else {
    snprintf(buf, 4096,
             "Workspace=%s Session=%s Matches=%d Summary=%s UserInput=%s GovernedGrounding=%s",
             query.workspace_id,
             request->session.session_id[0] ? request->session.session_id : "default",
             result.match_count,
             result.summary,
             request->user_input,
             governed_json);
  }

  *context_out = buf;
  return YAI_MIND_OK;
}
