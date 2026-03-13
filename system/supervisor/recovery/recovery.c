/* SPDX-License-Identifier: Apache-2.0 */
#include <yai/supervisor/recovery.h>

#include <stdio.h>

int yai_supervisor_recovery_plan(const char *scope_id, char *out, size_t out_cap)
{
  if (!out || out_cap == 0) return -1;
  if (!scope_id || !scope_id[0]) scope_id = "system";
  return (snprintf(out, out_cap, "recovery:none:%s", scope_id) < (int)out_cap) ? 0 : -1;
}
