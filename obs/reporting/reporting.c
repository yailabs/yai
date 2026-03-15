/* SPDX-License-Identifier: Apache-2.0 */
#include <yai/observability/reporting.h>

#include <stdio.h>

int yai_observability_reporting_snapshot(char *out, size_t out_cap)
{
  if (!out || out_cap == 0) return -1;
  return (snprintf(out, out_cap, "{\"reporting\":\"ready\"}") < (int)out_cap) ? 0 : -1;
}
