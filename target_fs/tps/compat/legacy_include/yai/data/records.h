/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <yai/data/store.h>
#include <yai/data/evidence.h>

int yai_data_records_append_event(const char *data_scope_id,
                                  const char *event_json,
                                  char *out_ref,
                                  size_t out_ref_cap,
                                  char *err,
                                  size_t err_cap);
int yai_data_records_append_decision(const char *data_scope_id,
                                     const char *decision_json,
                                     char *out_ref,
                                     size_t out_ref_cap,
                                     char *err,
                                     size_t err_cap);
/* Transitional compatibility wrapper for yai_data_evidence_append(). */
int yai_data_records_append_evidence(const char *data_scope_id,
                                     const char *evidence_json,
                                     char *out_ref,
                                     size_t out_ref_cap,
                                     char *err,
                                     size_t err_cap);
int yai_data_records_append_governance(const char *data_scope_id,
                                       const char *governance_json,
                                       char *out_ref,
                                       size_t out_ref_cap,
                                       char *err,
                                       size_t err_cap);
int yai_data_records_append_authority(const char *data_scope_id,
                                      const char *authority_json,
                                      char *out_ref,
                                      size_t out_ref_cap,
                                      char *err,
                                      size_t err_cap);
int yai_data_records_append_authority_resolution(const char *data_scope_id,
                                                 const char *authority_resolution_json,
                                                 char *out_ref,
                                                 size_t out_ref_cap,
                                                 char *err,
                                                 size_t err_cap);
int yai_data_records_append_enforcement_outcome(const char *data_scope_id,
                                                const char *enforcement_outcome_json,
                                                char *out_ref,
                                                size_t out_ref_cap,
                                                char *err,
                                                size_t err_cap);
int yai_data_records_append_enforcement_linkage(const char *data_scope_id,
                                                const char *enforcement_linkage_json,
                                                char *out_ref,
                                                size_t out_ref_cap,
                                                char *err,
                                                size_t err_cap);
int yai_data_records_append_source_class(const char *data_scope_id,
                                         const char *source_record_class,
                                         const char *source_record_json,
                                         char *out_ref,
                                         size_t out_ref_cap,
                                         char *err,
                                         size_t err_cap);

int yai_data_records_append_source_node(const char *data_scope_id,
                                        const char *source_node_json,
                                        char *out_ref,
                                        size_t out_ref_cap,
                                        char *err,
                                        size_t err_cap);
int yai_data_records_append_source_daemon_instance(const char *data_scope_id,
                                                   const char *source_daemon_instance_json,
                                                   char *out_ref,
                                                   size_t out_ref_cap,
                                                   char *err,
                                                   size_t err_cap);
int yai_data_records_append_source_binding(const char *data_scope_id,
                                           const char *source_binding_json,
                                           char *out_ref,
                                           size_t out_ref_cap,
                                           char *err,
                                           size_t err_cap);
int yai_data_records_append_source_asset(const char *data_scope_id,
                                         const char *source_asset_json,
                                         char *out_ref,
                                         size_t out_ref_cap,
                                         char *err,
                                         size_t err_cap);
int yai_data_records_append_source_acquisition_event(const char *data_scope_id,
                                                     const char *source_acquisition_event_json,
                                                     char *out_ref,
                                                     size_t out_ref_cap,
                                                     char *err,
                                                     size_t err_cap);
int yai_data_records_append_source_evidence_candidate(const char *data_scope_id,
                                                      const char *source_evidence_candidate_json,
                                                      char *out_ref,
                                                      size_t out_ref_cap,
                                                      char *err,
                                                      size_t err_cap);
int yai_data_records_append_source_owner_link(const char *data_scope_id,
                                              const char *source_owner_link_json,
                                              char *out_ref,
                                              size_t out_ref_cap,
                                              char *err,
                                              size_t err_cap);
