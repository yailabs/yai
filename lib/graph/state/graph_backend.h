/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <yai/knowledge/memory.h>

typedef struct yai_mind_graph_backend_ops {
  int (*node_create)(const char *domain,
                     const char *key,
                     const char *value,
                     yai_mind_node_id_t *node_id_out);
  int (*edge_create)(yai_mind_node_id_t from_node,
                     yai_mind_node_id_t to_node,
                     const char *relation,
                     float weight,
                     yai_mind_edge_id_t *edge_id_out);
  int (*node_get)(yai_mind_node_id_t node_id, yai_mind_graph_node_t *node_out);
  int (*edge_get)(yai_mind_edge_id_t edge_id, yai_mind_graph_edge_t *edge_out);
  int (*query)(const yai_mind_memory_query_t *query, yai_mind_memory_result_t *result_out);
  int (*stats)(yai_mind_graph_stats_t *stats_out);
  void (*shutdown)(void);
} yai_mind_graph_backend_ops_t;

int yai_mind_graph_backend_select_inmemory(void);
int yai_mind_graph_backend_select_rpc(const char *endpoint);
void yai_mind_graph_backend_shutdown_internal(void);
const yai_mind_graph_backend_ops_t *yai_mind_graph_backend_ops(void);
const char *yai_mind_graph_backend_name_internal(void);
