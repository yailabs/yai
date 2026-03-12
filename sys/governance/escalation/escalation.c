/* SPDX-License-Identifier: Apache-2.0 */
#include <yai/policy/governance/escalation.h>

#include <stdio.h>

int yai_governance_escalation_route(const char *scope_id, char *out, size_t out_cap)
{
  if (!out || out_cap == 0) return -1;
  if (!scope_id || !scope_id[0]) scope_id = "system";
  return (snprintf(out, out_cap, "escalation:none:%s", scope_id) < (int)out_cap) ? 0 : -1;
}
