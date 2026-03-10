/* SPDX-License-Identifier: Apache-2.0 */

#include "../../knowledge/cognition/cognition_internal.h"

#include <stdio.h>

int yai_mind_rag_build_prompt(yai_mind_rag_session_state_t *session_state,
                              const yai_mind_cognition_request_t *request,
                              const char *context,
                              char **prompt_out)
{
  char *buf;
  if (!session_state || !request || !context || !prompt_out) return YAI_MIND_ERR_INVALID_ARG;

  buf = (char *)yai_mind_arena_alloc(&session_state->arena, 1536, 16);
  if (!buf) return YAI_MIND_ERR_NO_MEMORY;

  snprintf(buf, 1536,
           "You are YAI Mind. Produce deterministic and concise output.\\n"
           "[context] %s\\n"
           "[task] %s\\n"
           "[input] %s\\n",
           context,
           request->task.prompt[0] ? request->task.prompt : "general_response",
           request->user_input);

  *prompt_out = buf;
  return YAI_MIND_OK;
}
