/* SPDX-License-Identifier: Apache-2.0 */
#include <yai/observability/traces.h>
#include <yai/observability/reporting.h>

#include <stdio.h>

int main(void)
{
  char traces[128];
  char reporting[128];
  if (yai_observability_traces_snapshot(traces, sizeof(traces)) != 0) return 1;
  if (yai_observability_reporting_snapshot(reporting, sizeof(reporting)) != 0) return 1;
  puts("yai-auditd - observability audit entrypoint");
  puts(traces);
  puts(reporting);
  return 0;
}
