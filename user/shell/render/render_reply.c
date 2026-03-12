/* SPDX-License-Identifier: Apache-2.0 */

#include "yai/shell/render/render_reply.h"

#include <stdlib.h>
#include <string.h>

#include "yai/shell/color.h"
#include "yai/shell/style_map.h"

#include <yai/sdk/reply/reply_json.h>

static const char *outcome_label(yai_outcome_t outcome)
{
  switch (outcome) {
    case YAI_OUTCOME_OK: return "OK";
    case YAI_OUTCOME_WARN: return "WARN";
    case YAI_OUTCOME_DENY: return "DENY";
    case YAI_OUTCOME_FAIL: return "FAIL";
    default: return "FAIL";
  }
}

static const char *outcome_color(yai_outcome_t outcome)
{
  switch (outcome) {
    case YAI_OUTCOME_OK: return yai_style_color(YAI_STYLE_OK);
    case YAI_OUTCOME_WARN: return yai_style_color(YAI_STYLE_WARN);
    case YAI_OUTCOME_DENY: return yai_style_color(YAI_STYLE_ERR);
    case YAI_OUTCOME_FAIL: return yai_style_color(YAI_STYLE_ERR);
    default: return yai_style_color(YAI_STYLE_ERR);
  }
}

void yai_render_reply_human(
    FILE *out,
    const yai_reply_t *r,
    const yai_render_reply_opts_t *opts)
{
  int use_color = 0;
  if (!out || !r) return;
  if (opts && opts->use_color && opts->is_tty) use_color = 1;

  yai_color_print(out, use_color, outcome_color(r->outcome), outcome_label(r->outcome));
  fprintf(out, " %s  %s\n",
          r->code ? r->code : "INTERNAL_ERROR",
          r->summary ? r->summary : "operation failed");

  if (!(opts && opts->quiet) && opts && opts->show_trace && r->trace.trace_id && r->trace.trace_id[0]) {
    fprintf(out, "trace: %s", r->trace.trace_id);
    if (r->trace.workspace_id && r->trace.workspace_id[0]) {
      fprintf(out, "  ws: %s", r->trace.workspace_id);
    }
    fputc('\n', out);
  }

  if (!(opts && opts->quiet) && r->hint_count > 0) {
    fputs("\nHINTS\n", out);
    for (size_t i = 0; i < r->hint_count; i++) {
      if (r->hints && r->hints[i] && r->hints[i][0]) {
        fprintf(out, "  %s\n", r->hints[i]);
      }
    }
  }

  if (!(opts && opts->quiet) && r->data && cJSON_GetArraySize(r->data) > 0) {
    char *printed = cJSON_PrintUnformatted(r->data);
    if (printed) {
      fputs("\nDETAILS\n", out);
      fprintf(out, "  data=%s\n", printed);
      free(printed);
    }
  }
}

void yai_render_reply_json(
    FILE *out,
    const yai_reply_t *r)
{
  char *json;
  if (!out || !r) return;
  json = yai_reply_to_json(r);
  if (!json) return;
  fprintf(out, "%s\n", json);
  free(json);
}
