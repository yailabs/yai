#pragma once

#include <stddef.h>

int yai_source_id_node(char *out, size_t out_cap, const char *source_label);
int yai_source_id_daemon_instance(char *out, size_t out_cap, const char *source_node_id);
int yai_source_id_binding(char *out, size_t out_cap, const char *source_node_id, const char *workspace_id);
int yai_source_id_asset(char *out, size_t out_cap, const char *source_binding_id, const char *locator);
int yai_source_id_acquisition_event(char *out,
                                    size_t out_cap,
                                    const char *source_binding_id,
                                    const char *event_type);
int yai_source_id_evidence_candidate(char *out,
                                     size_t out_cap,
                                     const char *source_acquisition_event_id,
                                     const char *candidate_type);
int yai_source_id_action_point(char *out,
                               size_t out_cap,
                               const char *source_binding_id,
                               const char *action_ref);
int yai_source_id_owner_link(char *out, size_t out_cap, const char *source_node_id, const char *owner_ref);
int yai_source_id_enrollment_grant(char *out,
                                   size_t out_cap,
                                   const char *source_node_id,
                                   const char *daemon_instance_id);
int yai_source_id_workspace_peer_membership(char *out,
                                            size_t out_cap,
                                            const char *workspace_id,
                                            const char *source_node_id,
                                            const char *source_binding_id);
