/* SPDX-License-Identifier: Apache-2.0 */

#include "yai/shell/color.h"

#include <stdlib.h>
#include <stdio.h>
#include <unistd.h>
#include <string.h>

int yai_color_enabled(FILE *stream, int no_color_flag, yai_color_mode_t mode)
{
  const char *term;
  int fd = STDOUT_FILENO;
  if (!stream) return 0;
  if (mode == YAI_COLOR_ALWAYS) return 1;
  if (mode == YAI_COLOR_NEVER) return 0;
  if (no_color_flag) return 0;
  if (getenv("NO_COLOR") != NULL) return 0;
  if (getenv("YAI_NO_COLOR") != NULL) return 0;
  term = getenv("TERM");
  if (term && strcmp(term, "dumb") == 0) return 0;
  if (stream == stderr) fd = STDERR_FILENO;
  return isatty(fd) ? 1 : 0;
}

void yai_color_print(FILE *stream, int enabled, const char *color, const char *text)
{
  if (!stream || !text) return;
  if (enabled && color && color[0]) fprintf(stream, "%s%s%s", color, text, YAI_COLOR_RESET);
  else fputs(text, stream);
}
