/* SPDX-License-Identifier: Apache-2.0 */

#include <yai/graph/summary.h>

#include <stdio.h>

#include "../internal/counts.h"

int yai_graph_summary_semantic_json(const char *workspace_id,
                                    char *out_json,
                                    size_t out_cap,
                                    char *err,
                                    size_t err_cap)
{
  size_t source_node = 0;
  size_t source_binding = 0;
  size_t source_asset = 0;
  size_t source_policy_snapshot = 0;
  if (err && err_cap > 0) err[0] = '\0';
  if (!workspace_id || !workspace_id[0] || !out_json || out_cap == 0) {
    if (err && err_cap > 0) snprintf(err, err_cap, "%s", "graph_semantic_domain_bad_args");
    return -1;
  }
  source_node = yai_graph_internal_query_count_or_zero(workspace_id, "source_node");
  source_binding = yai_graph_internal_query_count_or_zero(workspace_id, "source_binding");
  source_asset = yai_graph_internal_query_count_or_zero(workspace_id, "source_asset");
  source_policy_snapshot = yai_graph_internal_query_count_or_zero(workspace_id, "source_policy_snapshot");
  if (snprintf(out_json,
               out_cap,
               "{\"source_node\":%zu,\"source_binding\":%zu,\"source_asset\":%zu,\"source_policy_snapshot\":%zu}",
               source_node,
               source_binding,
               source_asset,
               source_policy_snapshot) <= 0) {
    if (err && err_cap > 0) snprintf(err, err_cap, "%s", "graph_semantic_domain_encode_failed");
    return -1;
  }
  return 0;
}
