/* SPDX-License-Identifier: Apache-2.0 */

#include "../../knowledge/cognition/cognition_internal.h"

int yai_mind_agent_code_handle(const yai_mind_agent_frame_t *frame,
                               yai_mind_cognition_response_t *response_out);
int yai_mind_agent_historian_handle(const yai_mind_agent_frame_t *frame,
                                    yai_mind_cognition_response_t *response_out);
int yai_mind_agent_knowledge_handle(const yai_mind_agent_frame_t *frame,
                                    yai_mind_cognition_response_t *response_out);
int yai_mind_agent_system_handle(const yai_mind_agent_frame_t *frame,
                                 yai_mind_cognition_response_t *response_out);
int yai_mind_agent_validator_handle(const yai_mind_agent_frame_t *frame,
                                    yai_mind_cognition_response_t *response_out);

int yai_mind_agent_execute(yai_mind_agent_role_t role,
                           const yai_mind_agent_frame_t *frame,
                           yai_mind_cognition_response_t *response_out)
{
  if (!frame || !response_out) return YAI_MIND_ERR_INVALID_ARG;
  switch (role) {
    case YAI_MIND_AGENT_ROLE_CODE:
      return yai_mind_agent_code_handle(frame, response_out);
    case YAI_MIND_AGENT_ROLE_HISTORIAN:
      return yai_mind_agent_historian_handle(frame, response_out);
    case YAI_MIND_AGENT_ROLE_KNOWLEDGE:
      return yai_mind_agent_knowledge_handle(frame, response_out);
    case YAI_MIND_AGENT_ROLE_VALIDATOR:
      return yai_mind_agent_validator_handle(frame, response_out);
    case YAI_MIND_AGENT_ROLE_SYSTEM:
    default:
      return yai_mind_agent_system_handle(frame, response_out);
  }
}

void yai_mind_agents_reset(void)
{
}
