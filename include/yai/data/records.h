/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <yai/data/store.h>

int yai_data_records_append_event(const char *workspace_id,
                                  const char *event_json,
                                  char *out_ref,
                                  size_t out_ref_cap,
                                  char *err,
                                  size_t err_cap);
int yai_data_records_append_decision(const char *workspace_id,
                                     const char *decision_json,
                                     char *out_ref,
                                     size_t out_ref_cap,
                                     char *err,
                                     size_t err_cap);
int yai_data_records_append_evidence(const char *workspace_id,
                                     const char *evidence_json,
                                     char *out_ref,
                                     size_t out_ref_cap,
                                     char *err,
                                     size_t err_cap);
int yai_data_records_append_governance(const char *workspace_id,
                                       const char *governance_json,
                                       char *out_ref,
                                       size_t out_ref_cap,
                                       char *err,
                                       size_t err_cap);
int yai_data_records_append_authority(const char *workspace_id,
                                      const char *authority_json,
                                      char *out_ref,
                                      size_t out_ref_cap,
                                      char *err,
                                      size_t err_cap);
int yai_data_records_append_authority_resolution(const char *workspace_id,
                                                 const char *authority_resolution_json,
                                                 char *out_ref,
                                                 size_t out_ref_cap,
                                                 char *err,
                                                 size_t err_cap);
int yai_data_records_append_enforcement_outcome(const char *workspace_id,
                                                const char *enforcement_outcome_json,
                                                char *out_ref,
                                                size_t out_ref_cap,
                                                char *err,
                                                size_t err_cap);
int yai_data_records_append_enforcement_linkage(const char *workspace_id,
                                                const char *enforcement_linkage_json,
                                                char *out_ref,
                                                size_t out_ref_cap,
                                                char *err,
                                                size_t err_cap);
