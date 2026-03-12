/* SPDX-License-Identifier: Apache-2.0 */

#include "yai/shell/screen.h"
#include "yai/shell/term.h"

#include <stdio.h>

void yai_screen_begin_frame(int clear)
{
  if (clear) yai_term_clear();
  else fputs("\033[H", stdout);
}

void yai_screen_end_frame(void)
{
  fflush(stdout);
}
