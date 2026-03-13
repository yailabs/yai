/* SPDX-License-Identifier: Apache-2.0 */
#include <yai/policy/review.h>

#include <stdio.h>

int yai_policy_review_status(const char *scope_id, char *status, size_t status_cap)
{
  if (!status || status_cap == 0) return -1;
  if (!scope_id || !scope_id[0]) scope_id = "system";
  return (snprintf(status, status_cap, "review-ready:%s", scope_id) < (int)status_cap) ? 0 : -1;
}
