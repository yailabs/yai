/* SPDX-License-Identifier: Apache-2.0 */

#include "yai/shell/style_map.h"

const char *yai_style_color(yai_style_role_t role)
{
  switch (role) {
    case YAI_STYLE_OK: return YAI_COLOR_GREEN;
    case YAI_STYLE_WARN: return YAI_COLOR_YELLOW;
    case YAI_STYLE_ERR: return YAI_COLOR_RED;
    case YAI_STYLE_INFO: return YAI_COLOR_CYAN;
    case YAI_STYLE_MUTED: return YAI_COLOR_DIM;
    case YAI_STYLE_EMPH: return YAI_COLOR_BOLD;
    default: return "";
  }
}
