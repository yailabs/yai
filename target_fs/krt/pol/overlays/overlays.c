/* SPDX-License-Identifier: Apache-2.0 */
#include <yai/pol/overlays.h>

#include <stdio.h>

int yai_policy_overlays_apply(const char *scope_id, char *summary, size_t summary_cap)
{
  if (!summary || summary_cap == 0) return -1;
  if (!scope_id || !scope_id[0]) scope_id = "user";
  return (snprintf(summary, summary_cap, "overlay-merge:%s", scope_id) < (int)summary_cap) ? 0 : -1;
}
