#pragma once

#include <stddef.h>

int yai_exec_grounding_context_build(const char *workspace_id,
                                     const char *task_prompt,
                                     const char *user_input,
                                     char *out_json,
                                     size_t out_cap,
                                     char *out_reason,
                                     size_t reason_cap);
