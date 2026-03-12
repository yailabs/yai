/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <stddef.h>
#include <yai/sdk/client.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct yai_display_label {
  char scope[32];
  char action[96];
  char detail[128];
} yai_display_label_t;

typedef struct yai_display_result {
  char status_label[48];
  char detail[256];
  char hint[256];
} yai_display_result_t;

void yai_display_from_command(const char *command_id, int argc, char **argv, yai_display_label_t *out);
void yai_display_reason_human(const char *reason, char *out, size_t out_cap);
void yai_display_from_reply(const yai_sdk_reply_t *reply, yai_display_result_t *out);

#ifdef __cplusplus
}
#endif
