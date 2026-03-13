/* SPDX-License-Identifier: Apache-2.0 */
#define _POSIX_C_SOURCE 200809L

#include "yai/shell/lifecycle.h"

#include <yai/sdk/client.h>
#include <yai/sdk/errors.h>
#include <yai/sdk/paths.h>

#include <errno.h>
#include <fcntl.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <unistd.h>
#include <sys/stat.h>
#include <sys/wait.h>

static int is_executable_file(const char *path)
{
  struct stat st;
  if (!path || !path[0]) return 0;
  if (stat(path, &st) != 0) return 0;
  if (!S_ISREG(st.st_mode)) return 0;
  return access(path, X_OK) == 0;
}

static int resolve_runtime_bin(char *out, size_t out_cap)
{
  const char *env_bin = getenv("YAI_RUNTIME_BIN");
  const char *env_ws = getenv("YAI_WS");
  char cwd[1024] = {0};
  char candidate[1024] = {0};

  if (!out || out_cap == 0) return YAI_SDK_BAD_ARGS;

  if (env_bin && env_bin[0] && is_executable_file(env_bin)) {
    snprintf(out, out_cap, "%s", env_bin);
    return YAI_SDK_OK;
  }

  if (yai_path_runtime_bin(candidate, sizeof(candidate)) == 0 &&
      candidate[0] && is_executable_file(candidate)) {
    snprintf(out, out_cap, "%s", candidate);
    return YAI_SDK_OK;
  }

  if (env_ws && env_ws[0]) {
    if (snprintf(candidate, sizeof(candidate), "%s/yai/build/bin/yai", env_ws) > 0 &&
        is_executable_file(candidate)) {
      snprintf(out, out_cap, "%s", candidate);
      return YAI_SDK_OK;
    }
  }

  if (getcwd(cwd, sizeof(cwd))) {
    const char *probes[] = {
      "build/bin/yai",
      "../yai/build/bin/yai",
      "../../yai/build/bin/yai",
      NULL
    };
    for (size_t i = 0; probes[i]; i++) {
      if (snprintf(candidate, sizeof(candidate), "%s/%s", cwd, probes[i]) > 0 &&
          is_executable_file(candidate)) {
        snprintf(out, out_cap, "%s", candidate);
        return YAI_SDK_OK;
      }
    }
  }

  return YAI_SDK_IO;
}

static void reply_set(
    yai_sdk_reply_t *r,
    const char *status,
    const char *code,
    const char *reason,
    const char *command_id,
    const char *summary,
    const char *hint_1,
    const char *hint_2)
{
  if (!r) return;
  memset(r, 0, sizeof(*r));
  snprintf(r->status, sizeof(r->status), "%s", status ? status : "error");
  snprintf(r->code, sizeof(r->code), "%s", code ? code : "INTERNAL_ERROR");
  snprintf(r->reason, sizeof(r->reason), "%s", reason ? reason : "internal_error");
  snprintf(r->command_id, sizeof(r->command_id), "%s", command_id ? command_id : "yai.unknown.unknown");
  snprintf(r->summary, sizeof(r->summary), "%s", summary ? summary : "");
  if (hint_1 && hint_1[0]) {
    snprintf(r->hints[0], sizeof(r->hints[0]), "%s", hint_1);
    r->hint_count = 1;
  }
  if (hint_2 && hint_2[0]) {
    snprintf(r->hints[1], sizeof(r->hints[1]), "%s", hint_2);
    r->hint_count = 2;
  }
  snprintf(r->target_plane, sizeof(r->target_plane), "%s", "runtime");
}

static void sleep_ms(long ms)
{
  struct timespec ts;
  if (ms <= 0) return;
  ts.tv_sec = ms / 1000;
  ts.tv_nsec = (ms % 1000) * 1000000L;
  while (nanosleep(&ts, &ts) != 0 && errno == EINTR) {}
}

static int runtime_is_up(void)
{
  yai_sdk_client_t *client = NULL;
  yai_sdk_client_opts_t opts = {.container_id = "default", .arming = 1, .role = "operator", .auto_handshake = 0};
  int rc = yai_sdk_client_open(&client, &opts);
  if (rc != YAI_SDK_OK) return 0;
  yai_sdk_client_close(client);
  return 1;
}

static int runtime_pidfile_path(char *out, size_t cap)
{
  char runtime_home[1024];
  if (!out || cap == 0) return YAI_SDK_BAD_ARGS;
  if (yai_path_runtime_home(runtime_home, sizeof(runtime_home)) != 0 || !runtime_home[0]) {
    return YAI_SDK_IO;
  }
  if (snprintf(out, cap, "%s/runtime.pid", runtime_home) <= 0) return YAI_SDK_IO;
  return YAI_SDK_OK;
}

static int write_pidfile(pid_t pid)
{
  char pidfile[1024];
  FILE *f = NULL;
  if (runtime_pidfile_path(pidfile, sizeof(pidfile)) != YAI_SDK_OK) return YAI_SDK_IO;
  f = fopen(pidfile, "w");
  if (!f) return YAI_SDK_IO;
  fprintf(f, "%ld\n", (long)pid);
  fclose(f);
  return YAI_SDK_OK;
}

static int spawn_runtime_detached(void)
{
  char runtime_bin[1024];
  pid_t pid;
  if (resolve_runtime_bin(runtime_bin, sizeof(runtime_bin)) != YAI_SDK_OK || !runtime_bin[0]) {
    return YAI_SDK_IO;
  }
  pid = fork();
  if (pid < 0) return YAI_SDK_IO;
  if (pid == 0)
  {
    int devnull;
    (void)setsid();
    devnull = open("/dev/null", O_RDWR);
    if (devnull >= 0) {
      (void)dup2(devnull, STDIN_FILENO);
      (void)dup2(devnull, STDOUT_FILENO);
      (void)dup2(devnull, STDERR_FILENO);
      if (devnull > STDERR_FILENO) (void)close(devnull);
    }
    execl(runtime_bin, runtime_bin, "up", (char *)NULL);
    _exit(127);
  }
  (void)write_pidfile(pid);
  return YAI_SDK_OK;
}

static int run_up(yai_sdk_reply_t *out)
{
  if (runtime_is_up()) {
    reply_set(out, "ok", "OK", "lifecycle_up_already_running", "yai.lifecycle.up",
              "YAI service is already running.", NULL, NULL);
    return YAI_SDK_OK;
  }
  if (spawn_runtime_detached() != YAI_SDK_OK) {
    reply_set(out, "error", "SERVER_UNAVAILABLE", "lifecycle_up_spawn_failed", "yai.lifecycle.up",
              "Unable to start YAI service.",
              "Set YAI_RUNTIME_BIN or YAI_INSTALL_ROOT and retry.",
              "Run: yai doctor runtime");
    return YAI_SDK_SERVER_OFF;
  }
  for (int i = 0; i < 120; i++) {
    if (runtime_is_up()) {
      reply_set(out, "ok", "OK", "lifecycle_up_started", "yai.lifecycle.up",
                "YAI service started.", NULL, NULL);
      return YAI_SDK_OK;
    }
    sleep_ms(100);
  }
  reply_set(out, "error", "RUNTIME_NOT_READY", "lifecycle_up_timeout", "yai.lifecycle.up",
            "YAI service is up but not ready for control calls.",
            "Try: yai doctor runtime",
            "Run: yai doctor runtime");
  return YAI_SDK_RUNTIME_NOT_READY;
}

static int run_down(yai_sdk_reply_t *out)
{
  char runtime_bin[1024];
  pid_t pid;
  int status = 0;

  if (resolve_runtime_bin(runtime_bin, sizeof(runtime_bin)) != YAI_SDK_OK || !runtime_bin[0]) {
    reply_set(out, "error", "SERVER_UNAVAILABLE", "lifecycle_down_runtime_missing", "yai.lifecycle.down",
              "Unable to resolve YAI service host binary.",
              "Set YAI_RUNTIME_BIN or YAI_INSTALL_ROOT and retry.",
              NULL);
    return YAI_SDK_SERVER_OFF;
  }

  pid = fork();
  if (pid < 0) {
    reply_set(out, "error", "INTERNAL_ERROR", "lifecycle_down_fork_failed", "yai.lifecycle.down",
              "Unable to stop YAI service.", NULL, NULL);
    return YAI_SDK_IO;
  }

  if (pid == 0) {
    execl(runtime_bin, runtime_bin, "down", (char *)NULL);
    _exit(127);
  }

  if (waitpid(pid, &status, 0) < 0 || !WIFEXITED(status) || WEXITSTATUS(status) != 0) {
    reply_set(out, "error", "INTERNAL_ERROR", "lifecycle_down_failed", "yai.lifecycle.down",
              "YAI service fallback stop failed.",
              "Run: yai doctor runtime",
              NULL);
    return YAI_SDK_IO;
  }

  reply_set(out, "ok", "OK", "lifecycle_down_completed", "yai.lifecycle.down",
            "YAI service stop signal delivered.", NULL, NULL);
  return YAI_SDK_OK;
}

int yai_shell_lifecycle_run(const char *command_id, yai_sdk_reply_t *out)
{
  if (!command_id || !out) return YAI_SDK_BAD_ARGS;
  if (strcmp(command_id, "yai.lifecycle.up") == 0) return run_up(out);
  if (strcmp(command_id, "yai.lifecycle.down") == 0) return run_down(out);
  if (strcmp(command_id, "yai.lifecycle.restart") == 0) {
    int rc = run_down(out);
    if (rc != 0) return rc;
    rc = run_up(out);
    if (rc == 0) {
      reply_set(out, "ok", "OK", "lifecycle_restart_completed", "yai.lifecycle.restart",
                "YAI service restarted.", NULL, NULL);
    }
    return rc;
  }
  return YAI_SDK_BAD_ARGS;
}
