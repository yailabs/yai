/* SPDX-License-Identifier: Apache-2.0 */

#include <yai/cognition/memory.h>
#include <yai/graph/internal/backend.h>

static int ensure_backend(void)
{
  if (!yai_graph_backend_ops()) return yai_graph_backend_select_inmemory();
  return YAI_MIND_OK;
}

int yai_graph_node_create(const char *domain,
                               const char *key,
                               const char *value,
                               yai_node_id_t *node_id_out)
{
  int rc = ensure_backend();
  if (rc != YAI_MIND_OK) return rc;
  return yai_graph_backend_ops()->node_create(domain, key, value, node_id_out);
}

int yai_graph_edge_create(yai_node_id_t from_node,
                               yai_node_id_t to_node,
                               const char *relation,
                               float weight,
                               yai_edge_id_t *edge_id_out)
{
  int rc = ensure_backend();
  if (rc != YAI_MIND_OK) return rc;
  return yai_graph_backend_ops()->edge_create(from_node, to_node, relation, weight, edge_id_out);
}

int yai_graph_node_get(yai_node_id_t node_id, yai_graph_node_t *node_out)
{
  int rc = ensure_backend();
  if (rc != YAI_MIND_OK) return rc;
  return yai_graph_backend_ops()->node_get(node_id, node_out);
}

int yai_graph_edge_get(yai_edge_id_t edge_id, yai_graph_edge_t *edge_out)
{
  int rc = ensure_backend();
  if (rc != YAI_MIND_OK) return rc;
  return yai_graph_backend_ops()->edge_get(edge_id, edge_out);
}

int yai_graph_stats_get(yai_graph_stats_t *stats_out)
{
  int rc = ensure_backend();
  if (rc != YAI_MIND_OK) return rc;
  return yai_graph_backend_ops()->stats(stats_out);
}

int yai_memory_query_run(const yai_memory_query_t *query,
                              yai_memory_result_t *result_out)
{
  int rc = ensure_backend();
  if (rc != YAI_MIND_OK) return rc;
  return yai_graph_backend_ops()->query(query, result_out);
}

int yai_graph_backend_use_inmemory(void)
{
  return yai_graph_backend_select_inmemory();
}

int yai_graph_backend_use_rpc(const char *endpoint)
{
  return yai_graph_backend_select_rpc(endpoint);
}

const char *yai_graph_backend_name(void)
{
  return yai_graph_backend_name_internal();
}
