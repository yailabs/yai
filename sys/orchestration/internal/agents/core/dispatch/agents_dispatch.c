/* SPDX-License-Identifier: Apache-2.0 */

#include "../../../cognition/core/cognition/cognition_internal.h"

int yai_agent_code_handle(const yai_agent_frame_t *frame,
                               yai_cognition_response_t *response_out);
int yai_agent_historian_handle(const yai_agent_frame_t *frame,
                                    yai_cognition_response_t *response_out);
int yai_agent_knowledge_handle(const yai_agent_frame_t *frame,
                                    yai_cognition_response_t *response_out);
int yai_agent_system_handle(const yai_agent_frame_t *frame,
                                 yai_cognition_response_t *response_out);
int yai_agent_validator_handle(const yai_agent_frame_t *frame,
                                    yai_cognition_response_t *response_out);

int yai_agent_execute(yai_agent_role_t role,
                           const yai_agent_frame_t *frame,
                           yai_cognition_response_t *response_out)
{
  if (!frame || !response_out) return YAI_MIND_ERR_INVALID_ARG;
  switch (role) {
    case YAI_MIND_AGENT_ROLE_CODE:
      return yai_agent_code_handle(frame, response_out);
    case YAI_MIND_AGENT_ROLE_HISTORIAN:
      return yai_agent_historian_handle(frame, response_out);
    case YAI_MIND_AGENT_ROLE_KNOWLEDGE:
      return yai_agent_knowledge_handle(frame, response_out);
    case YAI_MIND_AGENT_ROLE_VALIDATOR:
      return yai_agent_validator_handle(frame, response_out);
    case YAI_MIND_AGENT_ROLE_SYSTEM:
    default:
      return yai_agent_system_handle(frame, response_out);
  }
}

void yai_agents_reset(void)
{
}
