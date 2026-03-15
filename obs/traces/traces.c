/* SPDX-License-Identifier: Apache-2.0 */
#include <yai/observability/traces.h>

#include <stdio.h>

int yai_observability_traces_snapshot(char *out, size_t out_cap)
{
  if (!out || out_cap == 0) return -1;
  return (snprintf(out, out_cap, "{\"traces\":{\"open_spans\":0}}") < (int)out_cap) ? 0 : -1;
}
