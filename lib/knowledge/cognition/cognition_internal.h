/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <yai/knowledge/cognition.h>
#include <yai/knowledge/memory.h>

typedef struct yai_mind_rag_session_state {
  int slot;
  char session_id[64];
  yai_mind_arena_t arena;
  unsigned long cycle;
  int initialized;
} yai_mind_rag_session_state_t;

typedef struct yai_mind_agent_frame {
  const yai_mind_cognition_request_t *request;
  const yai_mind_plan_step_t *plan;
  size_t plan_count;
  const char *context;
  const char *prompt;
} yai_mind_agent_frame_t;

int yai_mind_orchestration_plan(const yai_mind_cognition_request_t *request,
                                yai_mind_plan_step_t *steps_out,
                                size_t steps_cap,
                                size_t *steps_count_out);

int yai_mind_rag_session_acquire(const yai_mind_session_t *session,
                                 yai_mind_rag_session_state_t **session_out);
int yai_mind_rag_session_release(yai_mind_rag_session_state_t *session_state);

int yai_mind_rag_build_context(yai_mind_rag_session_state_t *session_state,
                               const yai_mind_cognition_request_t *request,
                               char **context_out);
int yai_mind_rag_build_prompt(yai_mind_rag_session_state_t *session_state,
                              const yai_mind_cognition_request_t *request,
                              const char *context,
                              char **prompt_out);

int yai_mind_agent_execute(yai_mind_agent_role_t role,
                           const yai_mind_agent_frame_t *frame,
                           yai_mind_cognition_response_t *response_out);

int yai_mind_rag_pipeline_run(const yai_mind_cognition_request_t *request,
                              yai_mind_cognition_response_t *response_out);

void yai_mind_agents_reset(void);
void yai_mind_rag_sessions_reset(void);
