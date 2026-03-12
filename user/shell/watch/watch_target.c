/* SPDX-License-Identifier: Apache-2.0 */

#include "watch_internal.h"

#include <stdio.h>
#include <string.h>

static int is_watch_opt(const char *s)
{
  return s && (
      strcmp(s, "--interval-ms") == 0 ||
      strcmp(s, "--interval") == 0 ||
      strcmp(s, "--count") == 0 ||
      strcmp(s, "--no-clear") == 0 ||
      strcmp(s, "--once") == 0);
}

static int watch_opt_has_value(const char *s)
{
  return s && (
      strcmp(s, "--interval-ms") == 0 ||
      strcmp(s, "--interval") == 0 ||
      strcmp(s, "--count") == 0);
}

static int copy_token(char *dst, size_t cap, const char *src)
{
  if (!dst || cap == 0 || !src || !src[0]) return 1;
  snprintf(dst, cap, "%s", src);
  return 0;
}

static void join_tokens(char *dst, size_t cap, int argc, char *const *argv)
{
  size_t off = 0;
  if (!dst || cap == 0) return;
  dst[0] = '\0';
  for (int i = 0; i < argc; i++) {
    if (!argv || !argv[i] || !argv[i][0]) continue;
    off += (size_t)snprintf(dst + off, (off < cap) ? cap - off : 0,
                            "%s%s", (off > 0) ? " " : "", argv[i]);
    if (off + 1 >= cap) break;
  }
}

static int add_token(yai_watch_target_t *target, const char *token)
{
  int idx;
  if (!target || !token || !token[0]) return 1;
  idx = target->argc;
  if (idx >= YAI_WATCH_TARGET_ARGV_CAP) return 1;
  if (copy_token(target->argv_storage[idx], sizeof(target->argv_storage[idx]), token) != 0) return 1;
  target->argv[idx] = target->argv_storage[idx];
  target->argc++;
  return 0;
}

static int is_gov_topic(const char *topic)
{
  return topic && (
      strcmp(topic, "decision") == 0 ||
      strcmp(topic, "evidence") == 0 ||
      strcmp(topic, "event") == 0 ||
      strcmp(topic, "effect") == 0 ||
      strcmp(topic, "disclosure") == 0);
}

static int apply_defaults(yai_watch_target_t *target)
{
  const char *entrypoint;
  if (!target || target->argc <= 0) return 1;
  entrypoint = target->argv[0];

  if (strcmp(entrypoint, "runtime") == 0 || strcmp(entrypoint, "service") == 0) {
    if (target->argc == 1) return add_token(target, "ping");
    return 0;
  }

  if (strcmp(entrypoint, "gov") == 0) {
    if (target->argc == 1) {
      if (add_token(target, "decision") != 0) return 1;
      return add_token(target, "status");
    }
    if (target->argc == 2 && is_gov_topic(target->argv[1])) {
      return add_token(target, "status");
    }
    return 0;
  }

  if (strcmp(entrypoint, "verify") == 0 && target->argc == 1) {
    return add_token(target, "alignment");
  }

  if (strcmp(entrypoint, "inspect") == 0 && target->argc == 1) {
    return add_token(target, "runtime");
  }

  if (strcmp(entrypoint, "source") == 0 && target->argc == 1) {
    return add_token(target, "status");
  }

  return 0;
}

int yai_watch_target_resolve(int argc,
                             char **argv,
                             yai_watch_target_t *target,
                             char *err,
                             size_t err_cap)
{
  int requested_count = 0;
  char *requested[YAI_WATCH_TARGET_ARGV_CAP] = {0};

  if (err && err_cap) err[0] = '\0';
  if (!target) {
    if (err && err_cap) snprintf(err, err_cap, "watch target resolver received null target");
    return 1;
  }

  memset(target, 0, sizeof(*target));

  for (int i = 0; i < argc; i++) {
    if (!argv || !argv[i] || !argv[i][0]) continue;
    if (is_watch_opt(argv[i])) {
      if (watch_opt_has_value(argv[i]) && i + 1 < argc) i++;
      continue;
    }
    if (requested_count >= YAI_WATCH_TARGET_ARGV_CAP) {
      if (err && err_cap) snprintf(err, err_cap, "watch target has too many tokens");
      return 1;
    }
    requested[requested_count++] = argv[i];
  }

  if (requested_count <= 0) {
    if (err && err_cap) snprintf(err, err_cap, "missing watch target");
    return 1;
  }

  join_tokens(target->requested_target, sizeof(target->requested_target), requested_count, requested);

  for (int i = 0; i < requested_count; i++) {
    if (add_token(target, requested[i]) != 0) {
      if (err && err_cap) snprintf(err, err_cap, "watch target token is too long");
      return 1;
    }
  }

  if (apply_defaults(target) != 0) {
    if (err && err_cap) snprintf(err, err_cap, "unable to normalize watch target");
    return 1;
  }

  if (target->argc <= 0 || !target->argv[0] || !target->argv[0][0]) {
    if (err && err_cap) snprintf(err, err_cap, "watch target is empty");
    return 1;
  }

  snprintf(target->entrypoint, sizeof(target->entrypoint), "%s", target->argv[0]);
  if (target->argc > 1 && target->argv[1]) snprintf(target->topic, sizeof(target->topic), "%s", target->argv[1]);
  if (target->argc > 2 && target->argv[2]) snprintf(target->op, sizeof(target->op), "%s", target->argv[2]);

  join_tokens(target->resolved_target, sizeof(target->resolved_target), target->argc, target->argv);
  if (target->argc >= 3) {
    snprintf(target->display_target, sizeof(target->display_target), "%s %s %s", target->argv[0], target->argv[1], target->argv[2]);
  } else if (target->argc == 2) {
    snprintf(target->display_target, sizeof(target->display_target), "%s %s", target->argv[0], target->argv[1]);
  } else {
    snprintf(target->display_target, sizeof(target->display_target), "%s", target->argv[0]);
  }
  snprintf(target->last_exec_target, sizeof(target->last_exec_target), "%s", target->resolved_target);
  return 0;
}

void yai_watch_target_mark_exec(yai_watch_target_t *target)
{
  if (!target) return;
  snprintf(target->last_exec_target, sizeof(target->last_exec_target), "%s", target->resolved_target);
}
