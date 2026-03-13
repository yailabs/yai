/* SPDX-License-Identifier: Apache-2.0 */

#include <yai/graph/summary.h>

#include <stdio.h>

#include "../internal/counts.h"

int yai_graph_summary_activation_json(const char *workspace_id,
                                      char *out_json,
                                      size_t out_cap,
                                      char *err,
                                      size_t err_cap)
{
  size_t events = 0;
  size_t source_events = 0;
  size_t ingest_outcomes = 0;
  if (err && err_cap > 0) err[0] = '\0';
  if (!workspace_id || !workspace_id[0] || !out_json || out_cap == 0) {
    if (err && err_cap > 0) snprintf(err, err_cap, "%s", "graph_activation_domain_bad_args");
    return -1;
  }
  events = yai_graph_internal_query_count_or_zero(workspace_id, "events");
  source_events = yai_graph_internal_query_count_or_zero(workspace_id, "source_acquisition_event");
  ingest_outcomes = yai_graph_internal_query_count_or_zero(workspace_id, "source_ingest_outcome");
  if (snprintf(out_json,
               out_cap,
               "{\"events\":%zu,\"source_acquisition_event\":%zu,\"source_ingest_outcome\":%zu}",
               events,
               source_events,
               ingest_outcomes) <= 0) {
    if (err && err_cap > 0) snprintf(err, err_cap, "%s", "graph_activation_domain_encode_failed");
    return -1;
  }
  return 0;
}
