/* SPDX-License-Identifier: Apache-2.0 */
#include <yai/supervisor/registry.h>

#include <stdio.h>

int yai_supervisor_registry_snapshot(char *out, size_t out_cap)
{
  if (!out || out_cap == 0) return -1;
  return (snprintf(out, out_cap, "{\"supervisor\":\"ready\"}") < (int)out_cap) ? 0 : -1;
}
