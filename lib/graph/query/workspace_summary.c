/* SPDX-License-Identifier: Apache-2.0 */

#include <yai/graph/lineage.h>
#include <yai/graph/graph.h>
#include <yai/graph/materialization.h>
#include <yai/graph/query.h>
#include <yai/graph/summary.h>

#include <stdio.h>
#include <string.h>

int yai_graph_query_workspace_summary(const char *workspace_id,
                                      char *out_json,
                                      size_t out_cap,
                                      char *err,
                                      size_t err_cap)
{
  yai_mind_graph_stats_t stats = {0};
  size_t ws_nodes = 0;
  size_t ws_edges = 0;
  char node_ref[192];
  char edge_ref[192];
  char graph_store_ref[512];
  const char *backend = yai_graph_backend_name();
  if (err && err_cap > 0) err[0] = '\0';
  if (!workspace_id || !workspace_id[0] || !out_json || out_cap == 0) {
    if (err && err_cap > 0) snprintf(err, err_cap, "%s", "graph_query_bad_args");
    return -1;
  }
  if (yai_mind_graph_stats_get(&stats) != YAI_MIND_OK) {
    if (err && err_cap > 0) snprintf(err, err_cap, "%s", "graph_stats_failed");
    return -1;
  }
  (void)yai_graph_materialization_workspace_counts(workspace_id, &ws_nodes, &ws_edges);
  (void)yai_mind_storage_bridge_last_refs(workspace_id,
                                          node_ref,
                                          sizeof(node_ref),
                                          edge_ref,
                                          sizeof(edge_ref),
                                          NULL,
                                          0,
                                          NULL,
                                          0,
                                          graph_store_ref,
                                          sizeof(graph_store_ref),
                                          NULL,
                                          0);
  if (snprintf(out_json,
               out_cap,
               "{\"workspace_id\":\"%s\",\"backend\":\"%s\",\"graph_node_count\":%zu,\"graph_edge_count\":%zu,"
               "\"workspace_graph_node_count\":%zu,\"workspace_graph_edge_count\":%zu,"
               "\"last_graph_node_ref\":\"%s\",\"last_graph_edge_ref\":\"%s\",\"graph_store_ref\":\"%s\"}",
               workspace_id,
               backend ? backend : "unknown",
               stats.node_count,
               stats.edge_count,
               ws_nodes,
               ws_edges,
               node_ref,
               edge_ref,
               graph_store_ref) <= 0) {
    if (err && err_cap > 0) snprintf(err, err_cap, "%s", "graph_summary_encode_failed");
    return -1;
  }
  return 0;
}

int yai_graph_query_recent_summary(const char *workspace_id,
                                   size_t limit,
                                   char *out_json,
                                   size_t out_cap,
                                   char *err,
                                   size_t err_cap)
{
  (void)limit;
  return yai_graph_query_workspace_summary(workspace_id, out_json, out_cap, err, err_cap);
}

int yai_graph_query_lineage_summary(const char *workspace_id,
                                    const char *anchor_ref,
                                    char *out_json,
                                    size_t out_cap,
                                    char *err,
                                    size_t err_cap)
{
  if (err && err_cap > 0) err[0] = '\0';
  if (!workspace_id || !workspace_id[0] || !anchor_ref || !anchor_ref[0] || !out_json || out_cap == 0) {
    if (err && err_cap > 0) snprintf(err, err_cap, "%s", "graph_lineage_bad_args");
    return -1;
  }
  if (snprintf(out_json,
               out_cap,
               "{\"workspace_id\":\"%s\",\"anchor_ref\":\"%s\",\"lineage_status\":\"available\"}",
               workspace_id,
               anchor_ref) <= 0) {
    if (err && err_cap > 0) snprintf(err, err_cap, "%s", "graph_lineage_encode_failed");
    return -1;
  }
  return 0;
}
