/* SPDX-License-Identifier: Apache-2.0 */

#include "yai/shell/term.h"

#include <stdio.h>
#include <unistd.h>
#include <sys/ioctl.h>
#include <termios.h>
#include <stdlib.h>

int yai_term_width(void)
{
  struct winsize ws;
  if (ioctl(STDOUT_FILENO, TIOCGWINSZ, &ws) == 0 && ws.ws_col > 0) return (int)ws.ws_col;
  return 80;
}

int yai_term_height(void)
{
  struct winsize ws;
  if (ioctl(STDOUT_FILENO, TIOCGWINSZ, &ws) == 0 && ws.ws_row > 0) return (int)ws.ws_row;
  return 24;
}

int yai_term_is_tty(void)
{
  return isatty(STDOUT_FILENO) ? 1 : 0;
}

int yai_term_is_stream_tty(FILE *stream)
{
  if (!stream) return 0;
  if (stream == stderr) return isatty(STDERR_FILENO) ? 1 : 0;
  return isatty(STDOUT_FILENO) ? 1 : 0;
}

void yai_term_clear(void)
{
  fputs("\033[H\033[J", stdout);
}

void yai_term_alt_screen_enter(void)
{
  fputs("\033[?1049h", stdout);
}

void yai_term_alt_screen_leave(void)
{
  fputs("\033[?1049l", stdout);
}

void yai_term_cursor_hide(void)
{
  fputs("\033[?25l", stdout);
}

void yai_term_cursor_show(void)
{
  fputs("\033[?25h", stdout);
}

int yai_term_enable_raw_mode(yai_term_raw_state_t *state)
{
  struct termios *saved;
  struct termios raw;
  if (!state) return 1;
  if (state->active) return 0;
  if (!isatty(STDIN_FILENO)) return 2;
  saved = (struct termios *)malloc(sizeof(*saved));
  if (!saved) return 3;
  if (tcgetattr(STDIN_FILENO, saved) != 0) {
    free(saved);
    return 4;
  }
  raw = *saved;
  raw.c_lflag &= (tcflag_t)~(ECHO | ICANON | IEXTEN | ISIG);
  raw.c_iflag &= (tcflag_t)~(IXON | ICRNL | BRKINT | INPCK | ISTRIP);
  raw.c_oflag &= (tcflag_t)~(OPOST);
  raw.c_cflag |= (tcflag_t)CS8;
  raw.c_cc[VMIN] = 0;
  raw.c_cc[VTIME] = 1;
  if (tcsetattr(STDIN_FILENO, TCSAFLUSH, &raw) != 0) {
    free(saved);
    return 5;
  }
  state->active = 1;
  state->fd = STDIN_FILENO;
  state->saved_termios = saved;
  return 0;
}

void yai_term_disable_raw_mode(yai_term_raw_state_t *state)
{
  struct termios *saved;
  if (!state || !state->active) return;
  saved = (struct termios *)state->saved_termios;
  if (saved) {
    (void)tcsetattr(state->fd, TCSAFLUSH, saved);
    free(saved);
  }
  state->active = 0;
  state->fd = -1;
  state->saved_termios = NULL;
}
