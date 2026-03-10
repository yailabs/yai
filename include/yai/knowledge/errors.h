/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#ifdef __cplusplus
extern "C" {
#endif

typedef enum yai_mind_error {
  YAI_MIND_OK = 0,
  YAI_MIND_ERR_INVALID_ARG = -1,
  YAI_MIND_ERR_NO_MEMORY = -2,
  YAI_MIND_ERR_NOT_FOUND = -3,
  YAI_MIND_ERR_NOT_IMPLEMENTED = -4,
  YAI_MIND_ERR_PROVIDER = -5,
  YAI_MIND_ERR_TRANSPORT = -6,
  YAI_MIND_ERR_STATE = -7
} yai_mind_error_t;

#ifdef __cplusplus
}
#endif
