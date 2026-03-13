/* SPDX-License-Identifier: Apache-2.0 */
#include "yai/desktop/session.h"

int yai_desktop_session_start(yai_desktop_session_t *session)
{
  return (session && session->session_id && session->session_id[0]) ? 0 : -1;
}

int yai_desktop_session_stop(yai_desktop_session_t *session)
{
  (void)session;
  return 0;
}
