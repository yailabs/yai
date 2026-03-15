/* SPDX-License-Identifier: Apache-2.0 */

#include <yai/data/evidence.h>
#include <yai/data/query.h>
#include <yai/data/store.h>

#include <stdio.h>

int yai_data_evidence_append(const char *workspace_id,
                             const char *evidence_json,
                             char *out_ref,
                             size_t out_ref_cap,
                             char *err,
                             size_t err_cap)
{
  return yai_data_store_append(workspace_id,
                               "evidence",
                               evidence_json,
                               out_ref,
                               out_ref_cap,
                               err,
                               err_cap);
}

int yai_data_evidence_summary_json(const char *workspace_id,
                                   char *out_json,
                                   size_t out_cap,
                                   char *err,
                                   size_t err_cap)
{
  size_t evidence_count = 0;
  size_t decision_count = 0;
  size_t authority_count = 0;
  if (err && err_cap > 0) err[0] = '\0';
  if (!workspace_id || !workspace_id[0] || !out_json || out_cap == 0) {
    if (err && err_cap > 0) snprintf(err, err_cap, "%s", "evidence_summary_bad_args");
    return -1;
  }
  if (yai_data_query_count(workspace_id, "evidence", &evidence_count, err, err_cap) != 0) return -1;
  if (yai_data_query_count(workspace_id, "decisions", &decision_count, err, err_cap) != 0) return -1;
  if (yai_data_query_count(workspace_id, "authority_resolution", &authority_count, err, err_cap) != 0) return -1;
  if (snprintf(out_json,
               out_cap,
               "{\"workspace_id\":\"%s\",\"evidence_count\":%zu,\"decision_count\":%zu,\"authority_resolution_count\":%zu}",
               workspace_id,
               evidence_count,
               decision_count,
               authority_count) <= 0) {
    if (err && err_cap > 0) snprintf(err, err_cap, "%s", "evidence_summary_encode_failed");
    return -1;
  }
  return 0;
}
