/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <stdio.h>

#include <yai/sdk/reply/reply.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct {
  int use_color;
  int is_tty;
  int show_trace;
  int quiet;
} yai_render_reply_opts_t;

void yai_render_reply_human(
    FILE *out,
    const yai_reply_t *r,
    const yai_render_reply_opts_t *opts);

void yai_render_reply_json(
    FILE *out,
    const yai_reply_t *r);

#ifdef __cplusplus
}
#endif
