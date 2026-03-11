/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <yai/graph/graph.h>

typedef struct yai_graph_materialization_result {
  yai_mind_node_id_t workspace_node;
  yai_mind_node_id_t governance_node;
  yai_mind_node_id_t decision_node;
  yai_mind_node_id_t evidence_node;
  yai_mind_node_id_t authority_node;
  yai_mind_node_id_t artifact_node;
  yai_mind_node_id_t episode_node;
  yai_mind_edge_id_t edge_decision_in_workspace;
  yai_mind_edge_id_t edge_decision_under_governance;
  yai_mind_edge_id_t edge_decision_under_authority;
  yai_mind_edge_id_t edge_decision_on_artifact;
  yai_mind_edge_id_t edge_evidence_for_decision;
  yai_mind_edge_id_t edge_artifact_governed_by;
  yai_mind_edge_id_t edge_workspace_uses_governance;
  yai_mind_edge_id_t edge_episode_yielded_decision;
  char last_graph_node_ref[192];
  char last_graph_edge_ref[192];
} yai_graph_materialization_result_t;

int yai_graph_materialize_runtime_record_bundle(const char *workspace_id,
                                                const char *family_id,
                                                const char *specialization_id,
                                                const char *effect,
                                                const char *authority_profile,
                                                const char *resource_hint,
                                                const char *governance_ref,
                                                const char *authority_ref,
                                                const char *artifact_ref,
                                                const char *event_ref,
                                                const char *decision_ref,
                                                const char *evidence_ref,
                                                yai_graph_materialization_result_t *out,
                                                char *err,
                                                size_t err_cap);

int yai_graph_materialization_workspace_counts(const char *workspace_id,
                                               size_t *node_count_out,
                                               size_t *edge_count_out);

int yai_graph_materialization_workspace_source_counts(const char *workspace_id,
                                                      size_t *source_node_count_out,
                                                      size_t *source_edge_count_out);

int yai_graph_materialize_source_record(const char *workspace_id,
                                        const char *record_class,
                                        const char *record_json,
                                        char *out_node_ref,
                                        size_t out_node_ref_cap,
                                        char *out_edge_ref,
                                        size_t out_edge_ref_cap,
                                        char *err,
                                        size_t err_cap);

/* YD-3 source-plane canonical projection classes (implemented in YD-6). */
#define YAI_GRAPH_SOURCE_NODE_CLASS "source_node"
#define YAI_GRAPH_SOURCE_DAEMON_INSTANCE_CLASS "source_daemon_instance"
#define YAI_GRAPH_SOURCE_BINDING_CLASS "source_binding"
#define YAI_GRAPH_SOURCE_ASSET_CLASS "source_asset"
#define YAI_GRAPH_SOURCE_ACQUISITION_EVENT_CLASS "source_acquisition_event"
#define YAI_GRAPH_SOURCE_EVIDENCE_CANDIDATE_CLASS "source_evidence_candidate"
#define YAI_GRAPH_SOURCE_OWNER_LINK_CLASS "source_owner_link"
#define YAI_GRAPH_SOURCE_POLICY_SNAPSHOT_CLASS "source_policy_snapshot"
#define YAI_GRAPH_SOURCE_CAPABILITY_ENVELOPE_CLASS "source_capability_envelope"
#define YAI_GRAPH_WORKSPACE_PEER_MEMBERSHIP_CLASS "workspace_peer_membership"
#define YAI_GRAPH_SOURCE_SCOPE_CLASS "source_scope"
#define YAI_GRAPH_SOURCE_INGEST_OUTCOME_CLASS "source_ingest_outcome"
