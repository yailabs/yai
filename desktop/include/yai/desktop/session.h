/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#ifdef __cplusplus
extern "C" {
#endif

typedef struct yai_desktop_session {
  const char *session_id;
  const char *operator_id;
  const char *scope;
} yai_desktop_session_t;

int yai_desktop_session_start(yai_desktop_session_t *session);
int yai_desktop_session_stop(yai_desktop_session_t *session);

#ifdef __cplusplus
}
#endif
