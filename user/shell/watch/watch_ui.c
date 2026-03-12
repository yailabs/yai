/* SPDX-License-Identifier: Apache-2.0 */

#include "watch_internal.h"

#include "yai/shell/color.h"
#include "yai/shell/style_map.h"
#include "yai/shell/screen.h"

#include <stdio.h>
#include <string.h>

static const char *sev_label(yai_watch_severity_t sev)
{
  switch (sev) {
    case YAI_WATCH_OK: return "OK";
    case YAI_WATCH_WARN: return "WARN";
    default: return "ERR";
  }
}

static const char *watch_state_label(const yai_watch_model_t *m)
{
  if (!m) return "RUNNING";
  if (m->paused) return "PAUSED";
  if (m->count == 0) return "RUNNING";
  if (m->last_sev == YAI_WATCH_ERR) return "ERROR";
  if (m->last_sev == YAI_WATCH_WARN) return "DEGRADED";
  return "RUNNING";
}

static void truncate_to_width(char *dst, size_t cap, const char *src, int width)
{
  int src_len;
  if (!dst || cap == 0) return;
  dst[0] = '\0';
  if (!src) src = "";
  if (width <= 0) return;
  src_len = (int)strlen(src);
  if (src_len <= width) {
    snprintf(dst, cap, "%s", src);
    return;
  }
  if (width <= 3) {
    snprintf(dst, cap, "%.*s", width, src);
    return;
  }
  snprintf(dst, cap, "%.*s...", width - 3, src);
}

static void print_bounded_line(const char *line, int width)
{
  char out[2048];
  if (width <= 0) {
    fputs("\r\n", stdout);
    return;
  }
  truncate_to_width(out, sizeof(out), line ? line : "", width);
  fputs("\r", stdout);
  fputs(out, stdout);
  fputc('\n', stdout);
}

static int entry_matches_filter(const yai_watch_model_t *m, const yai_watch_entry_t *e)
{
  if (!m || !e) return 0;
  if (!m->filter[0]) return 1;
  return (strstr(e->summary, m->filter) != NULL ||
          strstr(e->raw, m->filter) != NULL ||
          strstr(e->target, m->filter) != NULL);
}

static int collect_visible_indices(const yai_watch_model_t *m, size_t *idx, size_t cap)
{
  int n = 0;
  if (!m || !idx || cap == 0) return 0;
  for (size_t i = 0; i < m->count; i++) {
    const yai_watch_entry_t *e = yai_watch_model_at(m, i);
    if (!e) continue;
    if (!entry_matches_filter(m, e)) continue;
    if ((size_t)n < cap) idx[n] = i;
    n++;
  }
  return n;
}

static void render_header(const yai_watch_model_t *m,
                          const yai_watch_target_t *target,
                          int width,
                          int interval_ms,
                          int compact,
                          int use_color)
{
  char line[1024];
  const char *state = watch_state_label(m);
  const char *display_target = (target && target->display_target[0]) ? target->display_target : "-";
  const char *filter = (m && m->filter[0]) ? m->filter : "-";
  (void)use_color;

  if (compact) {
    snprintf(line, sizeof(line), "YAI WATCH | %s | %s", display_target, state);
    print_bounded_line(line, width);
    snprintf(line, sizeof(line), "interval=%dms filter=%s", interval_ms, filter);
    print_bounded_line(line, width);
  } else {
    snprintf(line, sizeof(line), "YAI WATCH | target=%s | interval=%dms | state=%s | filter=%s",
             display_target, interval_ms, state, filter);
    print_bounded_line(line, width);
  }

  snprintf(line, sizeof(line), "Keys: q/ESC quit  space pause  r refresh  arrows scroll  / filter  c clear  j view  ? help");
  print_bounded_line(line, width);
}

static void render_help_overlay(int width)
{
  print_bounded_line("", width);
  print_bounded_line("WATCH CONTROLS", width);
  print_bounded_line("  q / ESC   quit", width);
  print_bounded_line("  space     pause/resume", width);
  print_bounded_line("  r         refresh now", width);
  print_bounded_line("  up/down   scroll history", width);
  print_bounded_line("  /         filter mode (ENTER apply, ESC cancel)", width);
  print_bounded_line("  c         clear history", width);
  print_bounded_line("  j         toggle view", width);
  print_bounded_line("  ?         toggle this help", width);
}

static void render_event_line(const yai_watch_entry_t *e, int width, int compact, int use_color)
{
  char label[512];
  char clipped[512];
  char line[1024];
  const char *lat = "-";
  char latbuf[32];
  int summary_width;

  if (!e) return;
  (void)use_color;

  if (e->latency_ms >= 0) {
    snprintf(latbuf, sizeof(latbuf), "%ldms", e->latency_ms);
    lat = latbuf;
  }

  if (compact) {
    snprintf(label, sizeof(label), "%s", e->summary[0] ? e->summary : "(no output)");
    summary_width = width - 8 - 1 - 4 - 1;
    truncate_to_width(clipped, sizeof(clipped), label, summary_width > 0 ? summary_width : 0);
    snprintf(line, sizeof(line), "%-8s %-4s %s", e->ts, sev_label(e->sev), clipped);
  } else {
    snprintf(label, sizeof(label), "%s | %s", e->target[0] ? e->target : "-", e->summary[0] ? e->summary : "(no output)");
    summary_width = width - 8 - 1 - 4 - 1 - 6 - 1;
    truncate_to_width(clipped, sizeof(clipped), label, summary_width > 0 ? summary_width : 0);
    snprintf(line, sizeof(line), "%-8s %-4s %-*s %5s", e->ts, sev_label(e->sev),
             summary_width > 0 ? summary_width : 0, clipped, lat);
  }

  print_bounded_line(line, width);
}

void yai_watch_ui_render(const yai_watch_model_t *m,
                         const yai_watch_target_t *target,
                         int width,
                         int height,
                         int interval_ms,
                         int use_color,
                         int no_clear)
{
  size_t visible_idx[YAI_WATCH_HISTORY_CAP];
  int visible_count;
  int compact;
  int reserved;
  int event_rows;
  int start;
  int end;
  char footer[512];

  if (width <= 0) width = 80;
  if (height <= 0) height = 24;

  compact = (width < 96) ? 1 : 0;

  yai_screen_begin_frame(!no_clear);

  render_header(m, target, width, interval_ms, compact, use_color);

  if (m && m->show_help) {
    render_help_overlay(width);
  }

  if (m && m->filter_input_mode) {
    char fline[256];
    snprintf(fline, sizeof(fline), "Filter> %s_", m->filter);
    print_bounded_line(fline, width);
  }

  if (!m) {
    yai_screen_end_frame();
    return;
  }

  visible_count = collect_visible_indices(m, visible_idx, YAI_WATCH_HISTORY_CAP);

  reserved = 3; /* header + keys */
  if (m->show_help) reserved += 9;
  if (m->filter_input_mode) reserved += 1;
  reserved += 1; /* footer */
  event_rows = height - reserved;
  if (event_rows < 3) event_rows = 3;

  start = 0;
  end = visible_count;
  if (visible_count > event_rows) {
    int end_pos = visible_count - (int)m->scroll;
    if (end_pos < event_rows) end_pos = event_rows;
    start = end_pos - event_rows;
    end = end_pos;
    if (end > visible_count) end = visible_count;
    if (start < 0) start = 0;
  }

  for (int i = start; i < end; i++) {
    const yai_watch_entry_t *e = yai_watch_model_at(m, visible_idx[i]);
    render_event_line(e, width, compact, use_color);
  }

  snprintf(footer, sizeof(footer),
           "ok=%d warn=%d err=%d  ticks=%lu  last=%ldms  mode=%s",
           m->ok_count,
           m->warn_count,
           m->err_count,
           m->tick_count,
           m->last_latency_ms,
           (m->paused ? "PAUSED" : "RUNNING"));
  print_bounded_line(footer, width);

  yai_screen_end_frame();
}
