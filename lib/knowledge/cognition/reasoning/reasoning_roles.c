/* SPDX-License-Identifier: Apache-2.0 */

#include <yai/knowledge/cognition.h>

#include <ctype.h>
#include <string.h>

static int contains_word(const char *haystack, const char *needle)
{
  size_t n;
  const char *p;
  if (!haystack || !needle || !needle[0]) return 0;
  n = strlen(needle);
  for (p = haystack; *p; p++) {
    size_t i = 0;
    while (needle[i] && p[i] && tolower((unsigned char)p[i]) == tolower((unsigned char)needle[i])) i++;
    if (i == n) return 1;
  }
  return 0;
}

yai_mind_agent_role_t yai_mind_reasoning_select_role(const yai_mind_cognition_request_t *request)
{
  const char *text;
  if (!request) return YAI_MIND_AGENT_ROLE_SYSTEM;
  if (request->preferred_role != YAI_MIND_AGENT_ROLE_UNSPECIFIED) return request->preferred_role;
  text = request->user_input;
  if (contains_word(text, "validate") || contains_word(text, "policy") || contains_word(text, "compliance")) {
    return YAI_MIND_AGENT_ROLE_VALIDATOR;
  }
  if (contains_word(text, "history") || contains_word(text, "audit") || contains_word(text, "timeline")) {
    return YAI_MIND_AGENT_ROLE_HISTORIAN;
  }
  if (contains_word(text, "code") || contains_word(text, "bug") || contains_word(text, "refactor")) {
    return YAI_MIND_AGENT_ROLE_CODE;
  }
  if (contains_word(text, "know") || contains_word(text, "what") || contains_word(text, "why")) {
    return YAI_MIND_AGENT_ROLE_KNOWLEDGE;
  }
  return YAI_MIND_AGENT_ROLE_SYSTEM;
}
