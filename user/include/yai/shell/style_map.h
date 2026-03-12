/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include "yai/shell/color.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef enum yai_style_role {
  YAI_STYLE_OK = 0,
  YAI_STYLE_WARN,
  YAI_STYLE_ERR,
  YAI_STYLE_INFO,
  YAI_STYLE_MUTED,
  YAI_STYLE_EMPH
} yai_style_role_t;

const char *yai_style_color(yai_style_role_t role);

#ifdef __cplusplus
}
#endif
