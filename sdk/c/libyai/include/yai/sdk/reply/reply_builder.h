/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <yai/sdk/reply/reply.h>

#ifdef __cplusplus
extern "C" {
#endif

void yai_reply_init_ok(yai_reply_t *r, const char *summary);
void yai_reply_init_warn(yai_reply_t *r, const char *code, const char *summary);
void yai_reply_init_deny(yai_reply_t *r, const char *code, const char *summary);
void yai_reply_init_fail(yai_reply_t *r, const char *code, const char *summary);

void yai_reply_add_hint(yai_reply_t *r, const char *hint);

void yai_reply_set_trace(
    yai_reply_t *r,
    const char *trace_id,
    const char *claim_id,
    const char *workspace_id);

void yai_reply_set_meta(
    yai_reply_t *r,
    const char *command_id,
    const char *target_plane);

void yai_reply_free(yai_reply_t *r);

#ifdef __cplusplus
}
#endif
