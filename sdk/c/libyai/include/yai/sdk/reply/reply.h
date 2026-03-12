/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <stddef.h>

#include <cJSON.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef enum {
  YAI_OUTCOME_OK = 0,
  YAI_OUTCOME_WARN = 1,
  YAI_OUTCOME_DENY = 2,
  YAI_OUTCOME_FAIL = 3,
} yai_outcome_t;

typedef struct {
  const char *trace_id;
  const char *claim_id;
  const char *workspace_id;
  const char *law_ref;
  const char *evidence_ref;
} yai_reply_trace_t;

typedef struct {
  const char *version;
  const char *ts;
  long duration_ms;
  const char *target_plane;
  const char *command_id;
} yai_reply_meta_t;

typedef struct {
  yai_outcome_t outcome;
  const char *code;
  const char *summary;

  const char **hints;
  size_t hint_count;

  cJSON *data;

  yai_reply_trace_t trace;
  yai_reply_meta_t meta;
} yai_reply_t;

#ifdef __cplusplus
}
#endif
