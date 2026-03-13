/* SPDX-License-Identifier: Apache-2.0 */
#include <yai/policy/grants.h>

#include <stdio.h>

int yai_policy_grants_view_json(const char *scope_id, char *out, size_t out_cap)
{
  if (!out || out_cap == 0) return -1;
  if (!scope_id || !scope_id[0]) scope_id = "system";
  return (snprintf(out, out_cap, "{\"scope\":\"%s\",\"grants_state\":\"active\"}", scope_id) < (int)out_cap)
             ? 0
             : -1;
}
