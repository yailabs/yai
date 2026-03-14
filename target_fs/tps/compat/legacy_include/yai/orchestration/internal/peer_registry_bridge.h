#pragma once

#include <stddef.h>
#include <stdint.h>

typedef struct yai_owner_peer_registry_entry {
  char workspace_id[64];
  char source_node_id[96];
  char source_binding_id[96];
  char daemon_instance_id[96];
  char peer_role[64];
  char peer_scope[256];
  char peer_state[32];
  int64_t backlog_queued;
  int64_t backlog_retry_due;
  int64_t backlog_failed;
  char coverage_ref[256];
  char overlap_state[32];
  int64_t replay_detected_count;
  int64_t overlap_detected_count;
  int64_t conflict_detected_count;
  int64_t ordering_late_count;
  int64_t review_required_count;
  char last_integrity_classification[48];
  char last_handling_action[48];
  int64_t last_seen_epoch;
  int64_t last_activity_epoch;
  int64_t updated_at_epoch;
} yai_owner_peer_registry_entry_t;

int yai_owner_peer_registry_upsert(const yai_owner_peer_registry_entry_t *entry,
                                   char *err,
                                   size_t err_cap);

int yai_owner_peer_registry_note_emit(const char *workspace_id,
                                      const char *source_node_id,
                                      const char *source_binding_id,
                                      int success,
                                      char *err,
                                      size_t err_cap);

int yai_owner_peer_registry_note_integrity(const char *workspace_id,
                                           const char *source_node_id,
                                           const char *source_binding_id,
                                           const char *classification,
                                           const char *handling_action,
                                           int ordering_late,
                                           int replay_detected,
                                           int overlap_detected,
                                           int conflict_detected,
                                           int review_required,
                                           char *err,
                                           size_t err_cap);

int yai_owner_peer_registry_workspace_summary_json(const char *workspace_id,
                                                   char *out_json,
                                                   size_t out_cap,
                                                   char *err,
                                                   size_t err_cap);

int yai_owner_peer_registry_workspace_peer_rows_json(const char *workspace_id,
                                                     char *out_json,
                                                     size_t out_cap,
                                                     char *err,
                                                     size_t err_cap);

int yai_owner_peer_registry_workspace_coverage_summary_json(const char *workspace_id,
                                                            char *out_json,
                                                            size_t out_cap,
                                                            char *err,
                                                            size_t err_cap);
