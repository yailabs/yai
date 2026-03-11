/* SPDX-License-Identifier: Apache-2.0 */

#include <yai/data/query.h>
#include <yai/daemon/source_plane_model.h>

#include <stdio.h>

#include "../store/internal.h"

int yai_data_query_count(const char *workspace_id,
                         const char *record_class,
                         size_t *out_count,
                         char *err,
                         size_t err_cap)
{
  char file[768];
  if (err && err_cap > 0) err[0] = '\0';
  if (out_count) *out_count = 0;
  if (!workspace_id || !workspace_id[0] || !record_class || !record_class[0] || !out_count) {
    if (err && err_cap > 0) snprintf(err, err_cap, "%s", "workspace_or_class_missing");
    return -1;
  }
  {
    int duck_rc = yai_data_duckdb_count(workspace_id, record_class, out_count, err, err_cap);
    if (duck_rc == 0) return 0;
  }
  if (yai_data_store_paths(workspace_id, record_class, NULL, 0, file, sizeof(file), err, err_cap) != 0) return -1;
  return yai_data_store_count_lines(file, out_count, err, err_cap);
}

int yai_data_query_summary_json(const char *workspace_id,
                                char *out_json,
                                size_t out_cap,
                                char *err,
                                size_t err_cap)
{
  size_t events = 0, decisions = 0, evidence = 0, governance = 0, authority = 0, authority_resolution = 0;
  size_t enforcement_outcome = 0, enforcement_linkage = 0;
  size_t source_node = 0, source_daemon_instance = 0, source_binding = 0;
  size_t source_asset = 0, source_acquisition_event = 0, source_evidence_candidate = 0, source_owner_link = 0;
  size_t source_enrollment_grant = 0;
  if (!out_json || out_cap == 0) return -1;
  out_json[0] = '\0';
  if (yai_data_query_count(workspace_id, "events", &events, err, err_cap) != 0) return -1;
  if (yai_data_query_count(workspace_id, "decisions", &decisions, err, err_cap) != 0) return -1;
  if (yai_data_query_count(workspace_id, "evidence", &evidence, err, err_cap) != 0) return -1;
  if (yai_data_query_count(workspace_id, "governance", &governance, err, err_cap) != 0) return -1;
  if (yai_data_query_count(workspace_id, "authority", &authority, err, err_cap) != 0) return -1;
  if (yai_data_query_count(workspace_id, "authority_resolution", &authority_resolution, err, err_cap) != 0) return -1;
  if (yai_data_query_count(workspace_id, "enforcement_outcome", &enforcement_outcome, err, err_cap) != 0) return -1;
  if (yai_data_query_count(workspace_id, "enforcement_linkage", &enforcement_linkage, err, err_cap) != 0) return -1;
  if (yai_data_query_count(workspace_id, YAI_SOURCE_RECORD_CLASS_NODE, &source_node, err, err_cap) != 0) return -1;
  if (yai_data_query_count(workspace_id, YAI_SOURCE_RECORD_CLASS_DAEMON_INSTANCE, &source_daemon_instance, err, err_cap) != 0) return -1;
  if (yai_data_query_count(workspace_id, YAI_SOURCE_RECORD_CLASS_BINDING, &source_binding, err, err_cap) != 0) return -1;
  if (yai_data_query_count(workspace_id, YAI_SOURCE_RECORD_CLASS_ASSET, &source_asset, err, err_cap) != 0) return -1;
  if (yai_data_query_count(workspace_id,
                           YAI_SOURCE_RECORD_CLASS_ACQUISITION_EVENT,
                           &source_acquisition_event,
                           err,
                           err_cap) != 0) return -1;
  if (yai_data_query_count(workspace_id,
                           YAI_SOURCE_RECORD_CLASS_EVIDENCE_CANDIDATE,
                           &source_evidence_candidate,
                           err,
                           err_cap) != 0) return -1;
  if (yai_data_query_count(workspace_id, YAI_SOURCE_RECORD_CLASS_OWNER_LINK, &source_owner_link, err, err_cap) != 0) return -1;
  if (yai_data_query_count(workspace_id, YAI_SOURCE_RECORD_CLASS_ENROLLMENT_GRANT, &source_enrollment_grant, err, err_cap) != 0) return -1;
  if (snprintf(out_json,
               out_cap,
               "{\"workspace_id\":\"%s\",\"counts\":{\"events\":%zu,\"decisions\":%zu,\"evidence\":%zu,\"governance\":%zu,\"authority\":%zu,\"authority_resolution\":%zu,\"enforcement_outcome\":%zu,\"enforcement_linkage\":%zu,\"source_node\":%zu,\"source_daemon_instance\":%zu,\"source_binding\":%zu,\"source_asset\":%zu,\"source_acquisition_event\":%zu,\"source_evidence_candidate\":%zu,\"source_owner_link\":%zu,\"source_enrollment_grant\":%zu}}",
               workspace_id,
               events,
               decisions,
               evidence,
               governance,
               authority,
               authority_resolution,
               enforcement_outcome,
               enforcement_linkage,
               source_node,
               source_daemon_instance,
               source_binding,
               source_asset,
               source_acquisition_event,
               source_evidence_candidate,
               source_owner_link,
               source_enrollment_grant) <= 0) {
    if (err && err_cap > 0) snprintf(err, err_cap, "%s", "summary_encode_failed");
    return -1;
  }
  return 0;
}

int yai_data_query_tail_json(const char *workspace_id,
                             const char *record_class,
                             size_t limit,
                             char *out_json,
                             size_t out_cap,
                             char *err,
                             size_t err_cap)
{
  return yai_data_store_tail_json(workspace_id, record_class, limit, out_json, out_cap, err, err_cap);
}
