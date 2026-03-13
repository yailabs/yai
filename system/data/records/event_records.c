/* SPDX-License-Identifier: Apache-2.0 */

#include <yai/data/records.h>
#include <yai/protocol/control/source_plane.h>
#include <stdio.h>

static int append_record(const char *workspace_id,
                         const char *record_class,
                         const char *record_json,
                         char *out_ref,
                         size_t out_ref_cap,
                         char *err,
                         size_t err_cap)
{
  return yai_data_store_append(workspace_id, record_class, record_json, out_ref, out_ref_cap, err, err_cap);
}

int yai_data_records_append_event(const char *workspace_id,
                                  const char *event_json,
                                  char *out_ref,
                                  size_t out_ref_cap,
                                  char *err,
                                  size_t err_cap)
{
  return append_record(workspace_id, "events", event_json, out_ref, out_ref_cap, err, err_cap);
}

int yai_data_records_append_decision(const char *workspace_id,
                                     const char *decision_json,
                                     char *out_ref,
                                     size_t out_ref_cap,
                                     char *err,
                                     size_t err_cap)
{
  return append_record(workspace_id, "decisions", decision_json, out_ref, out_ref_cap, err, err_cap);
}

int yai_data_records_append_evidence(const char *workspace_id,
                                     const char *evidence_json,
                                     char *out_ref,
                                     size_t out_ref_cap,
                                     char *err,
                                     size_t err_cap)
{
  return yai_data_evidence_append(workspace_id,
                                  evidence_json,
                                  out_ref,
                                  out_ref_cap,
                                  err,
                                  err_cap);
}

int yai_data_records_append_governance(const char *workspace_id,
                                       const char *governance_json,
                                       char *out_ref,
                                       size_t out_ref_cap,
                                       char *err,
                                       size_t err_cap)
{
  return append_record(workspace_id, "governance", governance_json, out_ref, out_ref_cap, err, err_cap);
}

int yai_data_records_append_authority(const char *workspace_id,
                                      const char *authority_json,
                                      char *out_ref,
                                      size_t out_ref_cap,
                                      char *err,
                                      size_t err_cap)
{
  return append_record(workspace_id, "authority", authority_json, out_ref, out_ref_cap, err, err_cap);
}

int yai_data_records_append_authority_resolution(const char *workspace_id,
                                                 const char *authority_resolution_json,
                                                 char *out_ref,
                                                 size_t out_ref_cap,
                                                 char *err,
                                                 size_t err_cap)
{
  return append_record(workspace_id, "authority_resolution", authority_resolution_json, out_ref, out_ref_cap, err, err_cap);
}

int yai_data_records_append_enforcement_outcome(const char *workspace_id,
                                                const char *enforcement_outcome_json,
                                                char *out_ref,
                                                size_t out_ref_cap,
                                                char *err,
                                                size_t err_cap)
{
  return append_record(workspace_id, "enforcement_outcome", enforcement_outcome_json, out_ref, out_ref_cap, err, err_cap);
}

int yai_data_records_append_enforcement_linkage(const char *workspace_id,
                                                const char *enforcement_linkage_json,
                                                char *out_ref,
                                                size_t out_ref_cap,
                                                char *err,
                                                size_t err_cap)
{
  return append_record(workspace_id, "enforcement_linkage", enforcement_linkage_json, out_ref, out_ref_cap, err, err_cap);
}

int yai_data_records_append_source_node(const char *workspace_id,
                                        const char *source_node_json,
                                        char *out_ref,
                                        size_t out_ref_cap,
                                        char *err,
                                        size_t err_cap)
{
  return append_record(workspace_id, YAI_SOURCE_RECORD_CLASS_NODE, source_node_json, out_ref, out_ref_cap, err, err_cap);
}

int yai_data_records_append_source_class(const char *workspace_id,
                                         const char *source_record_class,
                                         const char *source_record_json,
                                         char *out_ref,
                                         size_t out_ref_cap,
                                         char *err,
                                         size_t err_cap)
{
  if (!yai_source_record_class_is_known(source_record_class))
  {
    if (err && err_cap > 0)
    {
      (void)snprintf(err, err_cap, "%s", "unknown_source_record_class");
    }
    return -1;
  }
  return append_record(workspace_id,
                       source_record_class,
                       source_record_json,
                       out_ref,
                       out_ref_cap,
                       err,
                       err_cap);
}

int yai_data_records_append_source_daemon_instance(const char *workspace_id,
                                                   const char *source_daemon_instance_json,
                                                   char *out_ref,
                                                   size_t out_ref_cap,
                                                   char *err,
                                                   size_t err_cap)
{
  return append_record(workspace_id,
                       YAI_SOURCE_RECORD_CLASS_DAEMON_INSTANCE,
                       source_daemon_instance_json,
                       out_ref,
                       out_ref_cap,
                       err,
                       err_cap);
}

int yai_data_records_append_source_binding(const char *workspace_id,
                                           const char *source_binding_json,
                                           char *out_ref,
                                           size_t out_ref_cap,
                                           char *err,
                                           size_t err_cap)
{
  return append_record(workspace_id,
                       YAI_SOURCE_RECORD_CLASS_BINDING,
                       source_binding_json,
                       out_ref,
                       out_ref_cap,
                       err,
                       err_cap);
}

int yai_data_records_append_source_asset(const char *workspace_id,
                                         const char *source_asset_json,
                                         char *out_ref,
                                         size_t out_ref_cap,
                                         char *err,
                                         size_t err_cap)
{
  return append_record(workspace_id, YAI_SOURCE_RECORD_CLASS_ASSET, source_asset_json, out_ref, out_ref_cap, err, err_cap);
}

int yai_data_records_append_source_acquisition_event(const char *workspace_id,
                                                     const char *source_acquisition_event_json,
                                                     char *out_ref,
                                                     size_t out_ref_cap,
                                                     char *err,
                                                     size_t err_cap)
{
  return append_record(workspace_id,
                       YAI_SOURCE_RECORD_CLASS_ACQUISITION_EVENT,
                       source_acquisition_event_json,
                       out_ref,
                       out_ref_cap,
                       err,
                       err_cap);
}

int yai_data_records_append_source_evidence_candidate(const char *workspace_id,
                                                      const char *source_evidence_candidate_json,
                                                      char *out_ref,
                                                      size_t out_ref_cap,
                                                      char *err,
                                                      size_t err_cap)
{
  return append_record(workspace_id,
                       YAI_SOURCE_RECORD_CLASS_EVIDENCE_CANDIDATE,
                       source_evidence_candidate_json,
                       out_ref,
                       out_ref_cap,
                       err,
                       err_cap);
}

int yai_data_records_append_source_owner_link(const char *workspace_id,
                                              const char *source_owner_link_json,
                                              char *out_ref,
                                              size_t out_ref_cap,
                                              char *err,
                                              size_t err_cap)
{
  return append_record(workspace_id,
                       YAI_SOURCE_RECORD_CLASS_OWNER_LINK,
                       source_owner_link_json,
                       out_ref,
                       out_ref_cap,
                       err,
                       err_cap);
}
