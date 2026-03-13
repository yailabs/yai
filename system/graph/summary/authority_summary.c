/* SPDX-License-Identifier: Apache-2.0 */

#include <yai/graph/summary.h>

#include <stdio.h>

#include "../internal/counts.h"

int yai_graph_summary_authority_json(const char *workspace_id,
                                     char *out_json,
                                     size_t out_cap,
                                     char *err,
                                     size_t err_cap)
{
  size_t authority = 0;
  size_t authority_resolution = 0;
  size_t governance = 0;
  if (err && err_cap > 0) err[0] = '\0';
  if (!workspace_id || !workspace_id[0] || !out_json || out_cap == 0) {
    if (err && err_cap > 0) snprintf(err, err_cap, "%s", "graph_authority_domain_bad_args");
    return -1;
  }
  authority = yai_graph_internal_query_count_or_zero(workspace_id, "authority");
  authority_resolution = yai_graph_internal_query_count_or_zero(workspace_id, "authority_resolution");
  governance = yai_graph_internal_query_count_or_zero(workspace_id, "governance");
  if (snprintf(out_json,
               out_cap,
               "{\"authority\":%zu,\"authority_resolution\":%zu,\"governance\":%zu}",
               authority,
               authority_resolution,
               governance) <= 0) {
    if (err && err_cap > 0) snprintf(err, err_cap, "%s", "graph_authority_domain_encode_failed");
    return -1;
  }
  return 0;
}
