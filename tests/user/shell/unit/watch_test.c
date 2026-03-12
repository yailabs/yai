/* SPDX-License-Identifier: Apache-2.0 */

#include "../../src/watch/watch_internal.h"

#include <assert.h>
#include <stdio.h>
#include <string.h>

static void test_target_resolution_defaults(void)
{
  yai_watch_target_t t;
  char err[128];
  char *argv1[] = {"runtime"};
  char *argv2[] = {"gov", "decision"};
  char *argv3[] = {"verify"};

  assert(yai_watch_target_resolve(1, argv1, &t, err, sizeof(err)) == 0);
  assert(strcmp(t.resolved_target, "runtime ping") == 0);
  assert(strcmp(t.display_target, "runtime ping") == 0);

  assert(yai_watch_target_resolve(2, argv2, &t, err, sizeof(err)) == 0);
  assert(strcmp(t.resolved_target, "gov decision status") == 0);
  assert(strcmp(t.topic, "decision") == 0);
  assert(strcmp(t.op, "status") == 0);

  assert(yai_watch_target_resolve(1, argv3, &t, err, sizeof(err)) == 0);
  assert(strcmp(t.resolved_target, "verify alignment") == 0);
}

static void test_target_resolution_filters_watch_options(void)
{
  yai_watch_target_t t;
  char err[128];
  char *argv[] = {"runtime", "ping", "--interval-ms", "500", "--count", "3", "--no-clear"};
  assert(yai_watch_target_resolve(7, argv, &t, err, sizeof(err)) == 0);
  assert(strcmp(t.requested_target, "runtime ping") == 0);
  assert(strcmp(t.resolved_target, "runtime ping") == 0);
}

static void test_model_counters_and_clear(void)
{
  yai_watch_model_t m;
  yai_watch_entry_t e;
  yai_watch_model_init(&m);

  memset(&e, 0, sizeof(e));
  e.sev = YAI_WATCH_OK;
  e.latency_ms = 10;
  yai_watch_model_push(&m, &e);

  e.sev = YAI_WATCH_WARN;
  e.latency_ms = 20;
  yai_watch_model_push(&m, &e);

  e.sev = YAI_WATCH_ERR;
  e.latency_ms = 30;
  yai_watch_model_push(&m, &e);

  assert(m.ok_count == 1);
  assert(m.warn_count == 1);
  assert(m.err_count == 1);
  assert(m.count == 3);
  assert(m.tick_count == 3);
  assert(m.last_latency_ms == 30);

  yai_watch_model_clear(&m);
  assert(m.count == 0);
  assert(m.ok_count == 0);
  assert(m.warn_count == 0);
  assert(m.err_count == 0);
  assert(m.tick_count == 3);
}

int main(void)
{
  test_target_resolution_defaults();
  test_target_resolution_filters_watch_options();
  test_model_counters_and_clear();
  puts("watch_test: ok");
  return 0;
}
