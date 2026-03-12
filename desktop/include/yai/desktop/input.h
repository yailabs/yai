/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#ifdef __cplusplus
extern "C" {
#endif

typedef enum yai_desktop_input_type {
  YAI_DESKTOP_INPUT_NONE = 0,
  YAI_DESKTOP_INPUT_KEY = 1,
  YAI_DESKTOP_INPUT_POINTER = 2,
} yai_desktop_input_type_t;

typedef struct yai_desktop_input_event {
  yai_desktop_input_type_t type;
  int code;
  int value;
  int x;
  int y;
} yai_desktop_input_event_t;

#ifdef __cplusplus
}
#endif
