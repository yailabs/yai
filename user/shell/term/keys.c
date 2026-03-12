/* SPDX-License-Identifier: Apache-2.0 */

#include "yai/shell/keys.h"

#include <unistd.h>
#include <sys/select.h>

int yai_keys_read(yai_key_event_t *ev, int timeout_ms)
{
  fd_set rfds;
  struct timeval tv;
  unsigned char c = 0;
  int rc;
  if (!ev) return 0;
  ev->code = YAI_KEY_NONE;
  ev->ch = '\0';
  FD_ZERO(&rfds);
  FD_SET(STDIN_FILENO, &rfds);
  if (timeout_ms < 0) timeout_ms = 0;
  tv.tv_sec = timeout_ms / 1000;
  tv.tv_usec = (timeout_ms % 1000) * 1000;
  rc = select(STDIN_FILENO + 1, &rfds, NULL, NULL, &tv);
  if (rc <= 0 || !FD_ISSET(STDIN_FILENO, &rfds)) return 0;
  if (read(STDIN_FILENO, &c, 1) != 1) return 0;

  if (c == 27) {
    unsigned char seq[2];
    if (read(STDIN_FILENO, &seq[0], 1) == 1 && read(STDIN_FILENO, &seq[1], 1) == 1) {
      if (seq[0] == '[' && seq[1] == 'A') {
        ev->code = YAI_KEY_UP;
        return 1;
      }
      if (seq[0] == '[' && seq[1] == 'B') {
        ev->code = YAI_KEY_DOWN;
        return 1;
      }
    }
    ev->code = YAI_KEY_ESC;
    return 1;
  }
  if (c == '\n' || c == '\r') {
    ev->code = YAI_KEY_ENTER;
    return 1;
  }
  ev->code = YAI_KEY_CHAR;
  ev->ch = (char)c;
  return 1;
}
