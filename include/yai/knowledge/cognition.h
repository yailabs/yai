/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <stddef.h>

#include <yai/knowledge/errors.h>
#include <yai/knowledge/types.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef enum yai_mind_agent_role {
  YAI_MIND_AGENT_ROLE_UNSPECIFIED = 0,
  YAI_MIND_AGENT_ROLE_SYSTEM,
  YAI_MIND_AGENT_ROLE_CODE,
  YAI_MIND_AGENT_ROLE_KNOWLEDGE,
  YAI_MIND_AGENT_ROLE_HISTORIAN,
  YAI_MIND_AGENT_ROLE_VALIDATOR
} yai_mind_agent_role_t;

typedef struct yai_mind_cognition_request {
  yai_mind_session_t session;
  yai_mind_task_t task;
  yai_mind_plan_step_t hint_step;
  yai_mind_agent_role_t preferred_role;
  char user_input[512];
  char provider_name[32];
} yai_mind_cognition_request_t;

typedef struct yai_mind_cognition_response {
  int status;
  yai_mind_agent_role_t selected_role;
  float score;
  int plan_steps;
  char summary[256];
  char output[768];
} yai_mind_cognition_response_t;

int yai_mind_cognition_init(void);
int yai_mind_cognition_shutdown(void);
/* Canonical knowledge-owned cognition lifecycle entrypoints. */
int yai_knowledge_cognition_start(void);
int yai_knowledge_cognition_stop(void);

int yai_mind_cognition_execute(const yai_mind_cognition_request_t *request,
                               yai_mind_cognition_response_t *response_out);

int yai_mind_cognition_execute_text(const char *input,
                                    const char *session_id,
                                    const char *provider_name,
                                    yai_mind_cognition_response_t *response_out);

const char *yai_mind_agent_role_name(yai_mind_agent_role_t role);
yai_mind_agent_role_t yai_mind_reasoning_select_role(const yai_mind_cognition_request_t *request);
float yai_mind_reasoning_score(const yai_mind_cognition_request_t *request,
                               yai_mind_agent_role_t role);

#ifdef __cplusplus
}
#endif
