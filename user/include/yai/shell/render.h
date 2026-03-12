// SPDX-License-Identifier: Apache-2.0
#pragma once

#include <yai/sdk/client.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct yai_render_opts {
  int use_color;
  int is_tty;
  int show_trace;
  int quiet;
  const char *command_id;
  int argc;
  char **argv;
} yai_render_opts_t;

int yai_render_exec_short(const yai_sdk_reply_t *out, int rc, const yai_render_opts_t *opts);
int yai_render_exec_verbose(const yai_sdk_reply_t *out, int rc, const yai_render_opts_t *opts);
int yai_render_exec_contract_verbose(const yai_sdk_reply_t *out, int rc, const char *control_call_json);
int yai_render_exec_json(const yai_sdk_reply_t *out);
int yai_render_exec_exit_code(const yai_sdk_reply_t *out, int sdk_rc);

#ifdef __cplusplus
}
#endif
