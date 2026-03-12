/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <stdio.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct yai_term_raw_state {
  int active;
  int fd;
  void *saved_termios;
} yai_term_raw_state_t;

int yai_term_width(void);
int yai_term_height(void);
int yai_term_is_tty(void);
int yai_term_is_stream_tty(FILE *stream);
void yai_term_clear(void);
void yai_term_alt_screen_enter(void);
void yai_term_alt_screen_leave(void);
void yai_term_cursor_hide(void);
void yai_term_cursor_show(void);
int yai_term_enable_raw_mode(yai_term_raw_state_t *state);
void yai_term_disable_raw_mode(yai_term_raw_state_t *state);

#ifdef __cplusplus
}
#endif
