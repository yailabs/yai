/* SPDX-License-Identifier: Apache-2.0 */

#include <yai/data/records.h>

int yai_data_records_append_event(const char *workspace_id,
                                  const char *event_json,
                                  char *out_ref,
                                  size_t out_ref_cap,
                                  char *err,
                                  size_t err_cap)
{
  return yai_data_store_append(workspace_id, "events", event_json, out_ref, out_ref_cap, err, err_cap);
}

int yai_data_records_append_decision(const char *workspace_id,
                                     const char *decision_json,
                                     char *out_ref,
                                     size_t out_ref_cap,
                                     char *err,
                                     size_t err_cap)
{
  return yai_data_store_append(workspace_id, "decisions", decision_json, out_ref, out_ref_cap, err, err_cap);
}

int yai_data_records_append_evidence(const char *workspace_id,
                                     const char *evidence_json,
                                     char *out_ref,
                                     size_t out_ref_cap,
                                     char *err,
                                     size_t err_cap)
{
  return yai_data_store_append(workspace_id, "evidence", evidence_json, out_ref, out_ref_cap, err, err_cap);
}

int yai_data_records_append_governance(const char *workspace_id,
                                       const char *governance_json,
                                       char *out_ref,
                                       size_t out_ref_cap,
                                       char *err,
                                       size_t err_cap)
{
  return yai_data_store_append(workspace_id, "governance", governance_json, out_ref, out_ref_cap, err, err_cap);
}

int yai_data_records_append_authority(const char *workspace_id,
                                      const char *authority_json,
                                      char *out_ref,
                                      size_t out_ref_cap,
                                      char *err,
                                      size_t err_cap)
{
  return yai_data_store_append(workspace_id, "authority", authority_json, out_ref, out_ref_cap, err, err_cap);
}

int yai_data_records_append_authority_resolution(const char *workspace_id,
                                                 const char *authority_resolution_json,
                                                 char *out_ref,
                                                 size_t out_ref_cap,
                                                 char *err,
                                                 size_t err_cap)
{
  return yai_data_store_append(workspace_id, "authority_resolution", authority_resolution_json, out_ref, out_ref_cap, err, err_cap);
}

int yai_data_records_append_enforcement_outcome(const char *workspace_id,
                                                const char *enforcement_outcome_json,
                                                char *out_ref,
                                                size_t out_ref_cap,
                                                char *err,
                                                size_t err_cap)
{
  return yai_data_store_append(workspace_id, "enforcement_outcome", enforcement_outcome_json, out_ref, out_ref_cap, err, err_cap);
}

int yai_data_records_append_enforcement_linkage(const char *workspace_id,
                                                const char *enforcement_linkage_json,
                                                char *out_ref,
                                                size_t out_ref_cap,
                                                char *err,
                                                size_t err_cap)
{
  return yai_data_store_append(workspace_id, "enforcement_linkage", enforcement_linkage_json, out_ref, out_ref_cap, err, err_cap);
}
