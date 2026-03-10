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
