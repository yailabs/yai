/* SPDX-License-Identifier: Apache-2.0 */
#include <yai/services/sockets.h>

#include <stdio.h>

int yai_services_sockets_snapshot(char *out, size_t out_cap)
{
  if (!out || out_cap == 0) return -1;
  return (snprintf(out, out_cap, "{\"sockets\":{\"control\":\"/tmp/yai.sock\"}}") < (int)out_cap) ? 0 : -1;
}
