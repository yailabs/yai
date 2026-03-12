/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <yai/cognition/cognition.h>
#include <yai/cognition/memory.h>

typedef struct yai_rag_session_state {
  int slot;
  char session_id[64];
  yai_arena_t arena;
  unsigned long cycle;
  int initialized;
} yai_rag_session_state_t;

typedef struct yai_agent_frame {
  const yai_cognition_request_t *request;
  const yai_plan_step_t *plan;
  size_t plan_count;
  const char *context;
  const char *prompt;
} yai_agent_frame_t;

int yai_orchestration_plan(const yai_cognition_request_t *request,
                                yai_plan_step_t *steps_out,
                                size_t steps_cap,
                                size_t *steps_count_out);

int yai_rag_session_acquire(const yai_knowledge_session_t *session,
                                 yai_rag_session_state_t **session_out);
int yai_rag_session_release(yai_rag_session_state_t *session_state);

int yai_rag_build_context(yai_rag_session_state_t *session_state,
                               const yai_cognition_request_t *request,
                               char **context_out);
int yai_rag_build_prompt(yai_rag_session_state_t *session_state,
                              const yai_cognition_request_t *request,
                              const char *context,
                              char **prompt_out);

int yai_agent_execute(yai_agent_role_t role,
                           const yai_agent_frame_t *frame,
                           yai_cognition_response_t *response_out);

int yai_rag_pipeline_run(const yai_cognition_request_t *request,
                              yai_cognition_response_t *response_out);

void yai_agents_reset(void);
void yai_rag_sessions_reset(void);
