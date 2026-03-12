/* SPDX-License-Identifier: Apache-2.0 */

#include "yai/shell/pager.h"
#include "yai/shell/term.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

static int count_lines(const char *s)
{
  int lines = 0;
  if (!s || !s[0]) return 0;
  for (size_t i = 0; s[i]; i++) {
    if (s[i] == '\n') lines++;
  }
  return lines + 1;
}

static int env_pager_enabled(void)
{
  const char *v = getenv("YAI_PAGER");
  if (v && v[0]) {
    if (strcmp(v, "0") == 0 || strcmp(v, "false") == 0 || strcmp(v, "off") == 0) return 0;
    return 1;
  }
  v = getenv("PAGER");
  return (v && v[0]) ? 1 : 0;
}

static int env_no_pager_enabled(void)
{
  const char *v = getenv("YAI_NO_PAGER");
  if (!v || !v[0]) return 0;
  return (strcmp(v, "0") == 0 || strcmp(v, "false") == 0) ? 0 : 1;
}

void yai_shell_page_if_needed(const char *text, int force_pager, int no_pager)
{
  int lines;
  int height;
  int enabled;
  FILE *f;
  char tmp[256];
  char cmd[1024];
  const char *pager_cmd = getenv("YAI_PAGER");
  if (!text) return;
  if (no_pager || env_no_pager_enabled()) {
    fputs(text, stdout);
    return;
  }
  enabled = force_pager || env_pager_enabled();
  if (!enabled || !yai_term_is_tty()) {
    fputs(text, stdout);
    return;
  }
  lines = count_lines(text);
  height = yai_term_height();
  if (lines <= height) {
    fputs(text, stdout);
    return;
  }
  snprintf(tmp, sizeof(tmp), "/tmp/yai-pager-%ld.txt", (long)getpid());
  f = fopen(tmp, "w");
  if (!f) {
    fputs(text, stdout);
    return;
  }
  fputs(text, f);
  fclose(f);
  if (!pager_cmd || !pager_cmd[0] || strcmp(pager_cmd, "1") == 0 || strcmp(pager_cmd, "true") == 0) {
    pager_cmd = getenv("PAGER");
  }
  if (!pager_cmd || !pager_cmd[0]) pager_cmd = "less";
  if (strcmp(pager_cmd, "off") == 0) {
    fputs(text, stdout);
    unlink(tmp);
    return;
  }
  if (snprintf(cmd, sizeof(cmd), "%s -FRSX %s >/dev/null 2>&1 || more %s", pager_cmd, tmp, tmp) >= (int)sizeof(cmd)) {
    fputs(text, stdout);
    unlink(tmp);
    return;
  }
  (void)system(cmd);
  unlink(tmp);
}
