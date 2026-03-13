/* SPDX-License-Identifier: Apache-2.0 */

#include <yai/cognition/cognition.h>

#include "cognition_internal.h"

#include <stdio.h>
#include <string.h>

static int g_cognition_initialized = 0;

int yai_cognition_init(void)
{
  if (g_cognition_initialized) return YAI_MIND_OK;
  yai_agents_reset();
  yai_rag_sessions_reset();
  g_cognition_initialized = 1;
  return YAI_MIND_OK;
}

int yai_cognition_shutdown(void)
{
  if (!g_cognition_initialized) return YAI_MIND_OK;
  yai_rag_sessions_reset();
  yai_agents_reset();
  g_cognition_initialized = 0;
  return YAI_MIND_OK;
}

int yai_knowledge_cognition_start(void)
{
  return yai_cognition_init();
}

int yai_knowledge_cognition_stop(void)
{
  return yai_cognition_shutdown();
}

int yai_cognition_execute(const yai_cognition_request_t *request,
                               yai_cognition_response_t *response_out)
{
  if (!g_cognition_initialized) return YAI_MIND_ERR_STATE;
  if (!request || !response_out || !request->user_input[0]) return YAI_MIND_ERR_INVALID_ARG;
  memset(response_out, 0, sizeof(*response_out));
  return yai_rag_pipeline_run(request, response_out);
}

int yai_cognition_execute_text(const char *input,
                                    const char *session_id,
                                    const char *provider_name,
                                    yai_cognition_response_t *response_out)
{
  yai_cognition_request_t request = {0};

  if (!input || !input[0] || !response_out) return YAI_MIND_ERR_INVALID_ARG;

  snprintf(request.session.session_id, sizeof(request.session.session_id), "%s",
           (session_id && session_id[0]) ? session_id : "default");
  snprintf(request.session.workspace_id, sizeof(request.session.workspace_id), "%s", "default");
  snprintf(request.task.task_id, sizeof(request.task.task_id), "task-%.58s", request.session.session_id);
  snprintf(request.task.prompt, sizeof(request.task.prompt), "%s", input);
  snprintf(request.user_input, sizeof(request.user_input), "%s", input);
  if (provider_name && provider_name[0]) {
    snprintf(request.provider_name, sizeof(request.provider_name), "%s", provider_name);
  }
  request.preferred_role = YAI_MIND_AGENT_ROLE_UNSPECIFIED;

  return yai_cognition_execute(&request, response_out);
}

const char *yai_agent_role_name(yai_agent_role_t role)
{
  switch (role) {
    case YAI_MIND_AGENT_ROLE_SYSTEM: return "system";
    case YAI_MIND_AGENT_ROLE_CODE: return "code";
    case YAI_MIND_AGENT_ROLE_KNOWLEDGE: return "knowledge";
    case YAI_MIND_AGENT_ROLE_HISTORIAN: return "historian";
    case YAI_MIND_AGENT_ROLE_VALIDATOR: return "validator";
    default: return "unspecified";
  }
}
