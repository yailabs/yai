/* SPDX-License-Identifier: Apache-2.0 */

#include "../../knowledge/cognition/cognition_internal.h"

#include <stdio.h>
#include <string.h>

int yai_mind_rag_build_context(yai_mind_rag_session_state_t *session_state,
                               const yai_mind_cognition_request_t *request,
                               char **context_out)
{
  yai_mind_memory_query_t query = {0};
  yai_mind_memory_result_t result = {0};
  char *buf;
  int rc;

  if (!session_state || !request || !context_out) return YAI_MIND_ERR_INVALID_ARG;

  buf = (char *)yai_mind_arena_alloc(&session_state->arena, 1024, 16);
  if (!buf) return YAI_MIND_ERR_NO_MEMORY;

  snprintf(query.workspace_id, sizeof(query.workspace_id), "%s",
           request->session.workspace_id[0] ? request->session.workspace_id : "default");
  snprintf(query.query, sizeof(query.query), "%.255s", request->user_input);
  query.limit = 8;

  rc = yai_mind_memory_query_run(&query, &result);
  if (rc != YAI_MIND_OK) {
    snprintf(buf, 1024, "Context unavailable. query_error=%d", rc);
  } else {
    snprintf(buf, 1024,
             "Workspace=%s Session=%s Matches=%d Summary=%s UserInput=%s",
             query.workspace_id,
             request->session.session_id[0] ? request->session.session_id : "default",
             result.match_count,
             result.summary,
             request->user_input);
  }

  *context_out = buf;
  return YAI_MIND_OK;
}
