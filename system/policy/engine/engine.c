/* SPDX-License-Identifier: Apache-2.0 */
#include <yai/policy/engine.h>

#include <stdio.h>

int yai_policy_engine_evaluate(const char *scope_id, char *decision, size_t decision_cap)
{
  if (!decision || decision_cap == 0) return -1;
  if (!scope_id || !scope_id[0]) scope_id = "system";
  if (snprintf(decision, decision_cap, "allow:%s", scope_id) >= (int)decision_cap) return -1;
  return 0;
}
