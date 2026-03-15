/* SPDX-License-Identifier: Apache-2.0 */
#include <yai/observability/metrics.h>

#include <stdio.h>

int yai_observability_metrics_snapshot(char *out, size_t out_cap)
{
  if (!out || out_cap == 0) return -1;
  return (snprintf(out, out_cap, "{\"metrics\":{\"service_up\":1}}") < (int)out_cap) ? 0 : -1;
}
