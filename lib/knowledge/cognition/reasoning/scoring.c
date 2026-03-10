/* SPDX-License-Identifier: Apache-2.0 */

#include <yai/knowledge/cognition.h>

#include <string.h>

float yai_mind_reasoning_score(const yai_mind_cognition_request_t *request,
                               yai_mind_agent_role_t role)
{
  float base = 0.45f;
  if (!request) return 0.0f;

  if (request->preferred_role != YAI_MIND_AGENT_ROLE_UNSPECIFIED) base += 0.35f;
  if (request->task.prompt[0]) base += 0.08f;
  if (request->user_input[0] && strlen(request->user_input) > 24) base += 0.07f;

  switch (role) {
    case YAI_MIND_AGENT_ROLE_VALIDATOR: base += 0.06f; break;
    case YAI_MIND_AGENT_ROLE_CODE: base += 0.05f; break;
    case YAI_MIND_AGENT_ROLE_KNOWLEDGE: base += 0.04f; break;
    case YAI_MIND_AGENT_ROLE_HISTORIAN: base += 0.03f; break;
    case YAI_MIND_AGENT_ROLE_SYSTEM: base += 0.01f; break;
    default: break;
  }

  if (base > 0.99f) base = 0.99f;
  return base;
}
