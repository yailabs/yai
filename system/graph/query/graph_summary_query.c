/* SPDX-License-Identifier: Apache-2.0 */

#include <yai/graph/lineage.h>
#include <yai/graph/graph.h>
#include <yai/graph/materialization.h>
#include <yai/graph/query.h>
#include <yai/graph/summary.h>
#include <yai/protocol/control/source_plane.h>

#include <stdio.h>
#include <string.h>

#include "../internal/counts.h"

int yai_graph_query_summary(const char *graph_scope_id,
                            char *out_json,
                            size_t out_cap,
                            char *err,
                            size_t err_cap)
{
  yai_graph_stats_t stats = {0};
  size_t ws_nodes = 0;
  size_t ws_edges = 0;
  size_t src_nodes = 0;
  size_t src_edges = 0;
  char node_ref[192];
  char edge_ref[192];
  char graph_store_ref[512];
  const char *backend = yai_graph_backend_name();
  if (err && err_cap > 0) err[0] = '\0';
  if (!graph_scope_id || !graph_scope_id[0] || !out_json || out_cap == 0) {
    if (err && err_cap > 0) snprintf(err, err_cap, "%s", "graph_query_bad_args");
    return -1;
  }
  if (yai_graph_stats_get(&stats) != YAI_MIND_OK) {
    if (err && err_cap > 0) snprintf(err, err_cap, "%s", "graph_stats_failed");
    return -1;
  }
  (void)yai_graph_materialization_workspace_counts(graph_scope_id, &ws_nodes, &ws_edges);
  (void)yai_graph_materialization_workspace_source_counts(graph_scope_id, &src_nodes, &src_edges);
  (void)yai_storage_bridge_last_refs(graph_scope_id,
                                          node_ref,
                                          sizeof(node_ref),
                                          edge_ref,
                                          sizeof(edge_ref),
                                          NULL,
                                          0,
                                          NULL,
                                          0,
                                          graph_store_ref,
                                          sizeof(graph_store_ref),
                                          NULL,
                                          0);
  if (snprintf(out_json,
               out_cap,
               "{\"workspace_id\":\"%s\",\"backend\":\"%s\",\"graph_node_count\":%zu,\"graph_edge_count\":%zu,"
               "\"workspace_graph_node_count\":%zu,\"workspace_graph_edge_count\":%zu,"
               "\"source_graph_node_count\":%zu,\"source_graph_edge_count\":%zu,"
               "\"last_graph_node_ref\":\"%s\",\"last_graph_edge_ref\":\"%s\",\"graph_store_ref\":\"%s\"}",
               graph_scope_id,
               backend ? backend : "unknown",
               stats.node_count,
               stats.edge_count,
               ws_nodes,
               ws_edges,
               src_nodes,
               src_edges,
               node_ref,
               edge_ref,
               graph_store_ref) <= 0) {
    if (err && err_cap > 0) snprintf(err, err_cap, "%s", "graph_summary_encode_failed");
    return -1;
  }
  return 0;
}

int yai_graph_query_recent_summary(const char *graph_scope_id,
                                   size_t limit,
                                   char *out_json,
                                   size_t out_cap,
                                   char *err,
                                   size_t err_cap)
{
  (void)limit;
  return yai_graph_query_summary(graph_scope_id, out_json, out_cap, err, err_cap);
}

int yai_graph_query_unified_summary(const char *graph_scope_id,
                                    char *out_json,
                                    size_t out_cap,
                                    char *err,
                                    size_t err_cap)
{
  yai_graph_stats_t stats = {0};
  size_t ws_nodes = 0;
  size_t ws_edges = 0;
  size_t src_nodes = 0;
  size_t src_edges = 0;
  const char *backend = yai_graph_backend_name();
  size_t source_node = 0;
  size_t source_daemon_instance = 0;
  size_t source_binding = 0;
  size_t source_action_point = 0;
  size_t source_policy_snapshot = 0;
  size_t source_enrollment_grant = 0;
  size_t source_capability_envelope = 0;
  size_t source_acquisition_event = 0;
  size_t source_ingest_outcome = 0;
  size_t source_evidence_candidate = 0;
  size_t mesh_coordination_membership = 0;
  size_t mesh_peer_awareness = 0;
  size_t mesh_peer_legitimacy = 0;
  size_t mesh_authority_scope = 0;
  size_t mesh_transport_endpoint = 0;
  size_t mesh_transport_path_state = 0;
  size_t mesh_owner_remote_ingress = 0;
  size_t mesh_owner_remote_ingress_decision = 0;
  size_t mesh_overlay_presence = 0;
  size_t mesh_overlay_target_association = 0;
  size_t mesh_overlay_path_mutation = 0;
  char activation_json[192];
  char authority_json[192];
  char episodic_json[192];
  char semantic_json[256];
  char graph_store_ref[512];

  if (err && err_cap > 0) err[0] = '\0';
  if (!graph_scope_id || !graph_scope_id[0] || !out_json || out_cap == 0)
  {
    if (err && err_cap > 0) snprintf(err, err_cap, "%s", "graph_unified_summary_bad_args");
    return -1;
  }

  if (yai_graph_stats_get(&stats) != YAI_MIND_OK)
  {
    if (err && err_cap > 0) snprintf(err, err_cap, "%s", "graph_stats_failed");
    return -1;
  }

  (void)yai_graph_materialization_workspace_counts(graph_scope_id, &ws_nodes, &ws_edges);
  (void)yai_graph_materialization_workspace_source_counts(graph_scope_id, &src_nodes, &src_edges);
  (void)yai_storage_bridge_last_refs(graph_scope_id,
                                          NULL,
                                          0,
                                          NULL,
                                          0,
                                          NULL,
                                          0,
                                          NULL,
                                          0,
                                          graph_store_ref,
                                          sizeof(graph_store_ref),
                                          NULL,
                                          0);

  source_node = yai_graph_internal_query_count_or_zero(graph_scope_id, YAI_SOURCE_RECORD_CLASS_NODE);
  source_daemon_instance = yai_graph_internal_query_count_or_zero(graph_scope_id, YAI_SOURCE_RECORD_CLASS_DAEMON_INSTANCE);
  source_binding = yai_graph_internal_query_count_or_zero(graph_scope_id, YAI_SOURCE_RECORD_CLASS_BINDING);
  source_action_point = yai_graph_internal_query_count_or_zero(graph_scope_id, YAI_SOURCE_RECORD_CLASS_ACTION_POINT);
  source_policy_snapshot = yai_graph_internal_query_count_or_zero(graph_scope_id, YAI_SOURCE_RECORD_CLASS_POLICY_SNAPSHOT);
  source_enrollment_grant = yai_graph_internal_query_count_or_zero(graph_scope_id, YAI_SOURCE_RECORD_CLASS_ENROLLMENT_GRANT);
  source_capability_envelope = yai_graph_internal_query_count_or_zero(graph_scope_id, YAI_SOURCE_RECORD_CLASS_CAPABILITY_ENVELOPE);
  source_acquisition_event = yai_graph_internal_query_count_or_zero(graph_scope_id, YAI_SOURCE_RECORD_CLASS_ACQUISITION_EVENT);
  source_ingest_outcome = yai_graph_internal_query_count_or_zero(graph_scope_id, YAI_SOURCE_RECORD_CLASS_INGEST_OUTCOME);
  source_evidence_candidate = yai_graph_internal_query_count_or_zero(graph_scope_id, YAI_SOURCE_RECORD_CLASS_EVIDENCE_CANDIDATE);
  mesh_coordination_membership = yai_graph_internal_query_count_or_zero(graph_scope_id, YAI_SOURCE_RECORD_CLASS_MESH_COORDINATION_MEMBERSHIP);
  mesh_peer_awareness = yai_graph_internal_query_count_or_zero(graph_scope_id, YAI_SOURCE_RECORD_CLASS_MESH_PEER_AWARENESS);
  mesh_peer_legitimacy = yai_graph_internal_query_count_or_zero(graph_scope_id, YAI_SOURCE_RECORD_CLASS_MESH_PEER_LEGITIMACY);
  mesh_authority_scope = yai_graph_internal_query_count_or_zero(graph_scope_id, YAI_SOURCE_RECORD_CLASS_MESH_AUTHORITY_SCOPE);
  mesh_transport_endpoint = yai_graph_internal_query_count_or_zero(graph_scope_id, YAI_SOURCE_RECORD_CLASS_MESH_TRANSPORT_ENDPOINT);
  mesh_transport_path_state = yai_graph_internal_query_count_or_zero(graph_scope_id, YAI_SOURCE_RECORD_CLASS_MESH_TRANSPORT_PATH_STATE);
  mesh_owner_remote_ingress = yai_graph_internal_query_count_or_zero(graph_scope_id, YAI_SOURCE_RECORD_CLASS_MESH_OWNER_REMOTE_INGRESS);
  mesh_owner_remote_ingress_decision = yai_graph_internal_query_count_or_zero(graph_scope_id, YAI_SOURCE_RECORD_CLASS_MESH_OWNER_REMOTE_INGRESS_DECISION);
  mesh_overlay_presence = yai_graph_internal_query_count_or_zero(graph_scope_id, YAI_SOURCE_RECORD_CLASS_MESH_OVERLAY_PRESENCE);
  mesh_overlay_target_association = yai_graph_internal_query_count_or_zero(graph_scope_id, YAI_SOURCE_RECORD_CLASS_MESH_OVERLAY_TARGET_ASSOCIATION);
  mesh_overlay_path_mutation = yai_graph_internal_query_count_or_zero(graph_scope_id, YAI_SOURCE_RECORD_CLASS_MESH_OVERLAY_PATH_MUTATION);
  if (yai_graph_summary_activation_json(graph_scope_id, activation_json, sizeof(activation_json), NULL, 0) != 0) {
    snprintf(activation_json, sizeof(activation_json), "%s", "{}");
  }
  if (yai_graph_summary_authority_json(graph_scope_id, authority_json, sizeof(authority_json), NULL, 0) != 0) {
    snprintf(authority_json, sizeof(authority_json), "%s", "{}");
  }
  if (yai_graph_summary_episodic_json(graph_scope_id, episodic_json, sizeof(episodic_json), NULL, 0) != 0) {
    snprintf(episodic_json, sizeof(episodic_json), "%s", "{}");
  }
  if (yai_graph_summary_semantic_json(graph_scope_id, semantic_json, sizeof(semantic_json), NULL, 0) != 0) {
    snprintf(semantic_json, sizeof(semantic_json), "%s", "{}");
  }

  if (snprintf(out_json,
               out_cap,
               "{\"workspace_id\":\"%s\",\"backend\":\"%s\","
               "\"graph_counts\":{\"global_node_count\":%zu,\"global_edge_count\":%zu,\"workspace_node_count\":%zu,\"workspace_edge_count\":%zu,\"source_node_count\":%zu,\"source_edge_count\":%zu},"
               "\"entity_families\":{\"source_edge\":{\"source_node\":%zu,\"source_daemon_instance\":%zu,\"source_binding\":%zu,\"source_action_point\":%zu,\"source_policy_snapshot\":%zu,\"source_enrollment_grant\":%zu,\"source_capability_envelope\":%zu,\"source_acquisition_event\":%zu,\"source_ingest_outcome\":%zu,\"source_evidence_candidate\":%zu},"
               "\"mesh_coordination\":{\"mesh_coordination_membership\":%zu,\"mesh_peer_awareness\":%zu,\"mesh_peer_legitimacy\":%zu,\"mesh_authority_scope\":%zu},"
               "\"transport_ingress_overlay\":{\"mesh_transport_endpoint\":%zu,\"mesh_transport_path_state\":%zu,\"mesh_owner_remote_ingress\":%zu,\"mesh_owner_remote_ingress_decision\":%zu,\"mesh_overlay_presence\":%zu,\"mesh_overlay_target_association\":%zu,\"mesh_overlay_path_mutation\":%zu}},"
               "\"domains\":{\"activation\":%s,\"authority\":%s,\"episodic\":%s,\"semantic\":%s},"
               "\"adjudication_model\":{\"observed\":\"runtime_distributed_state\",\"accepted\":\"owner_ingress_or_owner_processing_accepted\",\"canonicalized\":\"owner_final_adjudication\"},"
               "\"graph_store_ref\":\"%s\"}",
               graph_scope_id,
               backend ? backend : "unknown",
               stats.node_count,
               stats.edge_count,
               ws_nodes,
               ws_edges,
               src_nodes,
               src_edges,
               source_node,
               source_daemon_instance,
               source_binding,
               source_action_point,
               source_policy_snapshot,
               source_enrollment_grant,
               source_capability_envelope,
               source_acquisition_event,
               source_ingest_outcome,
               source_evidence_candidate,
               mesh_coordination_membership,
               mesh_peer_awareness,
               mesh_peer_legitimacy,
               mesh_authority_scope,
               mesh_transport_endpoint,
               mesh_transport_path_state,
               mesh_owner_remote_ingress,
               mesh_owner_remote_ingress_decision,
               mesh_overlay_presence,
               mesh_overlay_target_association,
               mesh_overlay_path_mutation,
               activation_json,
               authority_json,
               episodic_json,
               semantic_json,
               graph_store_ref) <= 0)
  {
    if (err && err_cap > 0) snprintf(err, err_cap, "%s", "graph_unified_summary_encode_failed");
    return -1;
  }

  return 0;
}
