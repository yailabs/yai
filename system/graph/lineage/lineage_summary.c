/* SPDX-License-Identifier: Apache-2.0 */

#include <yai/graph/lineage.h>

#include <stdio.h>

int yai_graph_lineage_summary(const char *graph_scope_id,
                              const char *anchor_ref,
                              char *out_json,
                              size_t out_cap,
                              char *err,
                              size_t err_cap)
{
  if (err && err_cap > 0) err[0] = '\0';
  if (!graph_scope_id || !graph_scope_id[0] || !anchor_ref || !anchor_ref[0] || !out_json || out_cap == 0) {
    if (err && err_cap > 0) snprintf(err, err_cap, "%s", "graph_lineage_bad_args");
    return -1;
  }
  if (snprintf(out_json,
               out_cap,
               "{\"graph_scope_id\":\"%s\",\"anchor_ref\":\"%s\",\"lineage_status\":\"available\"}",
               graph_scope_id,
               anchor_ref) <= 0) {
    if (err && err_cap > 0) snprintf(err, err_cap, "%s", "graph_lineage_encode_failed");
    return -1;
  }
  return 0;
}
