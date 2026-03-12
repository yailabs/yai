/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#ifdef __cplusplus
extern "C" {
#endif

typedef enum yai_key_code {
  YAI_KEY_NONE = 0,
  YAI_KEY_CHAR,
  YAI_KEY_ESC,
  YAI_KEY_ENTER,
  YAI_KEY_UP,
  YAI_KEY_DOWN
} yai_key_code_t;

typedef struct yai_key_event {
  yai_key_code_t code;
  char ch;
} yai_key_event_t;

int yai_keys_read(yai_key_event_t *ev, int timeout_ms);

#ifdef __cplusplus
}
#endif
