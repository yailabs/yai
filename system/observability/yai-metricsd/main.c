/* SPDX-License-Identifier: Apache-2.0 */
#include <yai/observability/metrics.h>

#include <stdio.h>

int main(void)
{
  char out[128];
  if (yai_observability_metrics_snapshot(out, sizeof(out)) != 0) return 1;
  puts("yai-metricsd - observability metrics entrypoint");
  puts(out);
  return 0;
}
