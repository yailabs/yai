/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <stdio.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef enum yai_color_mode {
  YAI_COLOR_AUTO = 0,
  YAI_COLOR_ALWAYS = 1,
  YAI_COLOR_NEVER = 2
} yai_color_mode_t;

#define YAI_COLOR_RESET "\x1b[0m"
#define YAI_COLOR_GREEN "\x1b[32m"
#define YAI_COLOR_YELLOW "\x1b[33m"
#define YAI_COLOR_RED "\x1b[31m"
#define YAI_COLOR_CYAN "\x1b[36m"
#define YAI_COLOR_BOLD "\x1b[1m"
#define YAI_COLOR_DIM "\x1b[2m"

int yai_color_enabled(FILE *stream, int no_color_flag, yai_color_mode_t mode);
void yai_color_print(FILE *stream, int enabled, const char *color, const char *text);

#ifdef __cplusplus
}
#endif
