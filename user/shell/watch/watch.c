/* SPDX-License-Identifier: Apache-2.0 */

#define _POSIX_C_SOURCE 200809L

#include "yai/shell/watch.h"

#include "watch_internal.h"

#include "yai/shell/color.h"
#include "yai/shell/keys.h"
#include "yai/shell/term.h"

#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <unistd.h>
#include <sys/wait.h>

static volatile sig_atomic_t g_running = 1;

static void on_signal_stop(int signo)
{
  (void)signo;
  g_running = 0;
}

static void now_ts(char *out, size_t cap)
{
  time_t now = time(NULL);
  struct tm tmv;
  if (!out || cap == 0) return;
  out[0] = '\0';
  if (localtime_r(&now, &tmv)) {
    strftime(out, cap, "%H:%M:%S", &tmv);
  }
}

static long monotonic_ms(void)
{
  struct timespec ts;
  if (clock_gettime(CLOCK_MONOTONIC, &ts) != 0) return 0;
  return (long)(ts.tv_sec * 1000L + ts.tv_nsec / 1000000L);
}

static yai_watch_severity_t severity_from_rc(int rc)
{
  if (rc == 0) return YAI_WATCH_OK;
  if (rc == 10 || rc == 40) return YAI_WATCH_WARN;
  return YAI_WATCH_ERR;
}

static int parse_summary_from_raw(const char *raw, char *summary, size_t cap)
{
  const char *p;
  char lines[3][256];
  int count = 0;
  if (!summary || cap == 0) return 1;
  summary[0] = '\0';
  if (!raw || !raw[0]) {
    snprintf(summary, cap, "(no output)");
    return 0;
  }

  memset(lines, 0, sizeof(lines));
  p = raw;
  while (*p && count < 3) {
    char tmp[256];
    int n = 0;
    while (*p == '\n' || *p == '\r') p++;
    if (!*p) break;
    while (*p && *p != '\n' && *p != '\r' && n + 1 < (int)sizeof(tmp)) {
      tmp[n++] = *p++;
    }
    tmp[n] = '\0';
    while (*p == '\n' || *p == '\r') p++;
    if (!tmp[0]) continue;
    snprintf(lines[count], sizeof(lines[count]), "%s", tmp);
    count++;
  }

  if (count >= 3 && strncmp(lines[2], "Hint:", 5) != 0) {
    snprintf(summary, cap, "%s", lines[2]);
  } else if (count >= 2) {
    snprintf(summary, cap, "%s", lines[1]);
  } else if (count == 1) {
    snprintf(summary, cap, "%s", lines[0]);
  } else {
    snprintf(summary, cap, "(no output)");
  }
  return 0;
}

static int run_once_capture(const yai_watch_target_t *target, yai_watch_entry_t *entry)
{
  int pipefd[2] = {-1, -1};
  pid_t pid;
  int status = 0;
  long t0;
  long t1;

  if (!target || !entry || target->argc <= 0) return 1;

  memset(entry, 0, sizeof(*entry));
  now_ts(entry->ts, sizeof(entry->ts));
  snprintf(entry->target, sizeof(entry->target), "%.127s",
           target->last_exec_target[0] ? target->last_exec_target : target->resolved_target);

  if (pipe(pipefd) != 0) {
    entry->rc = 50;
    entry->sev = YAI_WATCH_ERR;
    snprintf(entry->summary, sizeof(entry->summary), "watch: cannot open capture pipe");
    entry->raw[0] = '\0';
    entry->latency_ms = -1;
    return 1;
  }

  t0 = monotonic_ms();
  pid = fork();
  if (pid < 0) {
    close(pipefd[0]);
    close(pipefd[1]);
    entry->rc = 50;
    entry->sev = YAI_WATCH_ERR;
    snprintf(entry->summary, sizeof(entry->summary), "watch: cannot fork");
    entry->raw[0] = '\0';
    entry->latency_ms = -1;
    return 1;
  }

  if (pid == 0) {
    char *exec_argv[YAI_WATCH_TARGET_ARGV_CAP + 2];
    int j = 0;

    close(pipefd[0]);
    (void)dup2(pipefd[1], STDOUT_FILENO);
    (void)dup2(pipefd[1], STDERR_FILENO);
    close(pipefd[1]);

    exec_argv[j++] = "./dist/bin/yai";
    for (int i = 0; i < target->argc && j < (int)(sizeof(exec_argv) / sizeof(exec_argv[0])) - 1; i++) {
      exec_argv[j++] = target->argv[i];
    }
    exec_argv[j] = NULL;

    execv(exec_argv[0], exec_argv);
    _exit(127);
  }

  close(pipefd[1]);

  {
    size_t rawlen = 0;
    ssize_t nread;
    while ((nread = read(pipefd[0], entry->raw + rawlen, sizeof(entry->raw) - rawlen - 1)) > 0) {
      rawlen += (size_t)nread;
      if (rawlen + 1 >= sizeof(entry->raw)) break;
    }
    entry->raw[rawlen] = '\0';
  }

  close(pipefd[0]);
  (void)waitpid(pid, &status, 0);
  t1 = monotonic_ms();
  entry->latency_ms = (t1 >= t0) ? (t1 - t0) : -1;

  if (WIFEXITED(status)) entry->rc = WEXITSTATUS(status);
  else entry->rc = 50;

  entry->sev = severity_from_rc(entry->rc);
  parse_summary_from_raw(entry->raw, entry->summary, sizeof(entry->summary));
  if (!entry->summary[0]) {
    snprintf(entry->summary, sizeof(entry->summary), "%s", (entry->rc == 0) ? "OK" : "Command failed");
  }
  return 0;
}

static void non_tty_watch(const yai_watch_target_t *target, int interval_ms, int count)
{
  int iter = 0;
  if (count <= 0) count = 5;
  while (g_running && iter < count) {
    yai_watch_entry_t e;
    run_once_capture(target, &e);
    printf("%s %-4s %-24s %s (%ldms)\n",
           e.ts,
           (e.sev == YAI_WATCH_OK) ? "OK" : (e.sev == YAI_WATCH_WARN ? "WARN" : "ERR"),
           e.target,
           e.summary,
           e.latency_ms);
    fflush(stdout);
    iter++;
    if (iter >= count) break;
    {
      struct timespec ts;
      ts.tv_sec = interval_ms / 1000;
      ts.tv_nsec = (long)(interval_ms % 1000) * 1000000L;
      while (nanosleep(&ts, &ts) != 0 && g_running) {
      }
    }
  }
}

static void handle_key(yai_watch_model_t *m, const yai_key_event_t *ev, long *next_tick)
{
  size_t vis;
  if (!m || !ev) return;
  vis = yai_watch_model_visible_count(m);

  if (m->filter_input_mode) {
    size_t len = strlen(m->filter);
    if (ev->code == YAI_KEY_ESC) {
      m->filter_input_mode = 0;
      return;
    }
    if (ev->code == YAI_KEY_ENTER) {
      m->filter_input_mode = 0;
      return;
    }
    if (ev->code == YAI_KEY_CHAR && (unsigned char)ev->ch == 127) {
      if (len > 0) m->filter[len - 1] = '\0';
      return;
    }
    if (ev->code == YAI_KEY_CHAR && ev->ch >= 32 && ev->ch < 127 && len + 1 < sizeof(m->filter)) {
      m->filter[len] = ev->ch;
      m->filter[len + 1] = '\0';
    }
    return;
  }

  if (ev->code == YAI_KEY_ESC || (ev->code == YAI_KEY_CHAR && (ev->ch == 'q' || ev->ch == 3))) {
    g_running = 0;
    return;
  }

  if (ev->code == YAI_KEY_CHAR && ev->ch == ' ') {
    m->paused = !m->paused;
    return;
  }

  if (ev->code == YAI_KEY_CHAR && ev->ch == 'r') {
    if (next_tick) *next_tick = monotonic_ms();
    return;
  }

  if (ev->code == YAI_KEY_CHAR && ev->ch == 'c') {
    yai_watch_model_clear(m);
    return;
  }

  if (ev->code == YAI_KEY_CHAR && ev->ch == '?') {
    m->show_help = !m->show_help;
    return;
  }

  if (ev->code == YAI_KEY_CHAR && ev->ch == 'j') {
    m->view = (yai_watch_view_t)(((int)m->view + 1) % 3);
    return;
  }

  if (ev->code == YAI_KEY_CHAR && ev->ch == '/') {
    m->filter_input_mode = 1;
    m->filter[0] = '\0';
    return;
  }

  if (ev->code == YAI_KEY_UP || (ev->code == YAI_KEY_CHAR && ev->ch == 'k')) {
    if (m->scroll + 1 < vis) m->scroll++;
    return;
  }

  if (ev->code == YAI_KEY_DOWN || (ev->code == YAI_KEY_CHAR && ev->ch == 'n')) {
    if (m->scroll > 0) m->scroll--;
    return;
  }

  if (ev->code == YAI_KEY_CHAR && ev->ch == 'g') {
    m->scroll = (vis > 0) ? (vis - 1) : 0;
    return;
  }

  if (ev->code == YAI_KEY_CHAR && ev->ch == 'G') {
    m->scroll = 0;
  }
}

int yai_watch_mode_run(int argc, char **argv, int interval_ms, int count, int no_clear, int no_color)
{
  int iter = 0;
  int is_tty;
  int use_color;
  long next_tick;
  yai_term_raw_state_t raw = {0};
  yai_watch_model_t model;
  yai_watch_target_t target;
  char resolve_err[160];

  if (interval_ms <= 0) interval_ms = 1000;

  if (yai_watch_target_resolve(argc, argv, &target, resolve_err, sizeof(resolve_err)) != 0) {
    fprintf(stderr, "watch\nBAD ARGS\n%s\nHint: Run: yai help watch\n",
            resolve_err[0] ? resolve_err : "invalid watch target");
    return 20;
  }

  signal(SIGINT, on_signal_stop);
  signal(SIGTERM, on_signal_stop);

  is_tty = yai_term_is_tty();
  if (!is_tty) {
    non_tty_watch(&target, interval_ms, count);
    return 0;
  }

  use_color = yai_color_enabled(stdout, no_color, YAI_COLOR_AUTO);
  yai_watch_model_init(&model);

  yai_term_alt_screen_enter();
  yai_term_cursor_hide();
  (void)yai_term_enable_raw_mode(&raw);

  next_tick = monotonic_ms();
  while (g_running) {
    long now = monotonic_ms();
    if (!model.paused && now >= next_tick) {
      yai_watch_entry_t e;
      yai_watch_target_mark_exec(&target);
      run_once_capture(&target, &e);
      yai_watch_model_push(&model, &e);
      iter++;
      next_tick = now + interval_ms;
      if (count > 0 && iter >= count) break;
    }

    yai_watch_ui_render(&model,
                        &target,
                        yai_term_width(),
                        yai_term_height(),
                        interval_ms,
                        use_color,
                        no_clear);

    {
      yai_key_event_t ev;
      if (yai_keys_read(&ev, 100)) {
        handle_key(&model, &ev, &next_tick);
      }
    }
  }

  yai_term_disable_raw_mode(&raw);
  yai_term_cursor_show();
  yai_term_alt_screen_leave();
  yai_term_clear();
  return 0;
}
