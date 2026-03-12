/* SPDX-License-Identifier: Apache-2.0 */

#include <yai/sdk/reply/reply_builder.h>

#include <stdlib.h>
#include <string.h>
#include <time.h>

#ifndef YAI_SDK_VERSION_STR
#define YAI_SDK_VERSION_STR "unknown"
#endif

static char *dup_or_null(const char *s)
{
  size_t n;
  char *p;
  if (!s) return NULL;
  n = strlen(s);
  p = (char *)malloc(n + 1);
  if (!p) return NULL;
  memcpy(p, s, n + 1);
  return p;
}

static void init_common(yai_reply_t *r, yai_outcome_t outcome, const char *code, const char *summary)
{
  if (!r) return;
  memset(r, 0, sizeof(*r));
  r->outcome = outcome;
  r->code = dup_or_null(code ? code : "INTERNAL_ERROR");
  r->summary = dup_or_null(summary ? summary : "operation failed");
  r->meta.version = dup_or_null(YAI_SDK_VERSION_STR);
}

static void set_ts_now(yai_reply_t *r)
{
  struct timespec ts;
  struct tm tmv;
  char buf[64];
  if (!r) return;
  if (clock_gettime(CLOCK_REALTIME, &ts) != 0) return;
  if (!gmtime_r(&ts.tv_sec, &tmv)) return;
  if (strftime(buf, sizeof(buf), "%Y-%m-%dT%H:%M:%SZ", &tmv) == 0) return;
  if (r->meta.ts) free((void *)r->meta.ts);
  r->meta.ts = dup_or_null(buf);
}

void yai_reply_init_ok(yai_reply_t *r, const char *summary)
{
  init_common(r, YAI_OUTCOME_OK, "OK", summary ? summary : "ok");
  set_ts_now(r);
}

void yai_reply_init_warn(yai_reply_t *r, const char *code, const char *summary)
{
  init_common(r, YAI_OUTCOME_WARN, code ? code : "WARN", summary ? summary : "warning");
  set_ts_now(r);
}

void yai_reply_init_deny(yai_reply_t *r, const char *code, const char *summary)
{
  init_common(r, YAI_OUTCOME_DENY, code ? code : "DENY", summary ? summary : "denied");
  set_ts_now(r);
}

void yai_reply_init_fail(yai_reply_t *r, const char *code, const char *summary)
{
  init_common(r, YAI_OUTCOME_FAIL, code ? code : "INTERNAL_ERROR", summary ? summary : "failed");
  set_ts_now(r);
}

void yai_reply_add_hint(yai_reply_t *r, const char *hint)
{
  const char **arr;
  if (!r || !hint || !hint[0]) return;
  arr = (const char **)realloc((void *)r->hints, sizeof(char *) * (r->hint_count + 1));
  if (!arr) return;
  r->hints = arr;
  r->hints[r->hint_count] = dup_or_null(hint);
  if (r->hints[r->hint_count]) r->hint_count++;
}

void yai_reply_set_trace(
    yai_reply_t *r,
    const char *trace_id,
    const char *claim_id,
    const char *workspace_id)
{
  if (!r) return;
  if (r->trace.trace_id) free((void *)r->trace.trace_id);
  if (r->trace.claim_id) free((void *)r->trace.claim_id);
  if (r->trace.workspace_id) free((void *)r->trace.workspace_id);
  r->trace.trace_id = dup_or_null(trace_id);
  r->trace.claim_id = dup_or_null(claim_id);
  r->trace.workspace_id = dup_or_null(workspace_id);
}

void yai_reply_set_meta(
    yai_reply_t *r,
    const char *command_id,
    const char *target_plane)
{
  if (!r) return;
  if (r->meta.command_id) free((void *)r->meta.command_id);
  if (r->meta.target_plane) free((void *)r->meta.target_plane);
  r->meta.command_id = dup_or_null(command_id);
  r->meta.target_plane = dup_or_null(target_plane);
}

void yai_reply_free(yai_reply_t *r)
{
  size_t i;
  if (!r) return;

  free((void *)r->code);
  free((void *)r->summary);

  for (i = 0; i < r->hint_count; i++) free((void *)r->hints[i]);
  free((void *)r->hints);

  if (r->data) cJSON_Delete(r->data);

  free((void *)r->trace.trace_id);
  free((void *)r->trace.claim_id);
  free((void *)r->trace.workspace_id);
  free((void *)r->trace.law_ref);
  free((void *)r->trace.evidence_ref);

  free((void *)r->meta.version);
  free((void *)r->meta.ts);
  free((void *)r->meta.target_plane);
  free((void *)r->meta.command_id);

  memset(r, 0, sizeof(*r));
}
