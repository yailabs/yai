/* SPDX-License-Identifier: Apache-2.0 */

#include "../../knowledge/cognition/cognition_internal.h"

#include <stdio.h>
#include <string.h>

int yai_mind_rag_pipeline_run(const yai_mind_cognition_request_t *request,
                              yai_mind_cognition_response_t *response_out)
{
  yai_mind_agent_role_t role;
  yai_mind_rag_session_state_t *session_state = NULL;
  yai_mind_plan_step_t plan[4];
  size_t plan_count = 0;
  yai_mind_agent_frame_t frame = {0};
  char *context = NULL;
  char *prompt = NULL;
  int rc;

  if (!request || !response_out) return YAI_MIND_ERR_INVALID_ARG;

  rc = yai_mind_rag_session_acquire(&request->session, &session_state);
  if (rc != YAI_MIND_OK) return rc;

  rc = yai_mind_orchestration_plan(request, plan, 4, &plan_count);
  if (rc != YAI_MIND_OK) return rc;

  rc = yai_mind_rag_build_context(session_state, request, &context);
  if (rc != YAI_MIND_OK) return rc;

  rc = yai_mind_rag_build_prompt(session_state, request, context, &prompt);
  if (rc != YAI_MIND_OK) return rc;

  role = yai_mind_reasoning_select_role(request);

  frame.request = request;
  frame.plan = plan;
  frame.plan_count = plan_count;
  frame.context = context;
  frame.prompt = prompt;

  rc = yai_mind_agent_execute(role, &frame, response_out);
  if (rc != YAI_MIND_OK) return rc;

  response_out->selected_role = role;
  response_out->score = yai_mind_reasoning_score(request, role);
  response_out->plan_steps = (int)plan_count;
  if (!response_out->summary[0]) {
    snprintf(response_out->summary, sizeof(response_out->summary),
             "Cognition pipeline completed with role=%s", yai_mind_agent_role_name(role));
  }

  (void)yai_mind_rag_session_release(session_state);
  return YAI_MIND_OK;
}
