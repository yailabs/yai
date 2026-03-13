/* SPDX-License-Identifier: Apache-2.0 */

#include <yai/graph/internal/backend.h>

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct yai_graph_store {
  yai_graph_node_t *nodes;
  size_t node_count;
  size_t node_cap;
  yai_graph_edge_t *edges;
  size_t edge_count;
  size_t edge_cap;
  yai_node_id_t next_node_id;
  yai_edge_id_t next_edge_id;
} yai_graph_store_t;

static yai_graph_store_t g_store = {0};

static int ensure_node_cap(size_t needed)
{
  yai_graph_node_t *next;
  size_t new_cap;
  if (needed <= g_store.node_cap) return YAI_MIND_OK;
  new_cap = (g_store.node_cap == 0) ? 64 : g_store.node_cap * 2;
  while (new_cap < needed) new_cap *= 2;
  next = (yai_graph_node_t *)realloc(g_store.nodes, new_cap * sizeof(*next));
  if (!next) return YAI_MIND_ERR_NO_MEMORY;
  g_store.nodes = next;
  g_store.node_cap = new_cap;
  return YAI_MIND_OK;
}

static int ensure_edge_cap(size_t needed)
{
  yai_graph_edge_t *next;
  size_t new_cap;
  if (needed <= g_store.edge_cap) return YAI_MIND_OK;
  new_cap = (g_store.edge_cap == 0) ? 64 : g_store.edge_cap * 2;
  while (new_cap < needed) new_cap *= 2;
  next = (yai_graph_edge_t *)realloc(g_store.edges, new_cap * sizeof(*next));
  if (!next) return YAI_MIND_ERR_NO_MEMORY;
  g_store.edges = next;
  g_store.edge_cap = new_cap;
  return YAI_MIND_OK;
}

static int node_exists(yai_node_id_t node_id)
{
  if (!yai_node_id_is_valid(node_id)) return 0;
  return (size_t)node_id <= g_store.node_count;
}

static int backend_node_create(const char *domain,
                               const char *key,
                               const char *value,
                               yai_node_id_t *node_id_out)
{
  yai_graph_node_t *slot;
  int rc;
  if (!domain || !domain[0] || !key || !key[0] || !node_id_out) return YAI_MIND_ERR_INVALID_ARG;
  rc = ensure_node_cap(g_store.node_count + 1);
  if (rc != YAI_MIND_OK) return rc;

  slot = &g_store.nodes[g_store.node_count++];
  memset(slot, 0, sizeof(*slot));
  slot->node_id = g_store.next_node_id++;
  if (slot->node_id == YAI_MIND_NODE_ID_INVALID) slot->node_id = g_store.next_node_id++;
  snprintf(slot->domain, sizeof(slot->domain), "%s", domain);
  snprintf(slot->key, sizeof(slot->key), "%s", key);
  snprintf(slot->value, sizeof(slot->value), "%s", value ? value : "");
  *node_id_out = slot->node_id;
  return YAI_MIND_OK;
}

static int backend_edge_create(yai_node_id_t from_node,
                               yai_node_id_t to_node,
                               const char *relation,
                               float weight,
                               yai_edge_id_t *edge_id_out)
{
  yai_graph_edge_t *slot;
  int rc;
  if (!relation || !relation[0] || !edge_id_out) return YAI_MIND_ERR_INVALID_ARG;
  if (!node_exists(from_node) || !node_exists(to_node)) return YAI_MIND_ERR_NOT_FOUND;

  rc = ensure_edge_cap(g_store.edge_count + 1);
  if (rc != YAI_MIND_OK) return rc;

  slot = &g_store.edges[g_store.edge_count++];
  memset(slot, 0, sizeof(*slot));
  slot->edge_id = g_store.next_edge_id++;
  if (slot->edge_id == YAI_MIND_EDGE_ID_INVALID) slot->edge_id = g_store.next_edge_id++;
  slot->from_node = from_node;
  slot->to_node = to_node;
  slot->weight = weight;
  snprintf(slot->relation, sizeof(slot->relation), "%s", relation);
  *edge_id_out = slot->edge_id;
  return YAI_MIND_OK;
}

static int backend_node_get(yai_node_id_t node_id, yai_graph_node_t *node_out)
{
  if (!node_out || !node_exists(node_id)) return YAI_MIND_ERR_NOT_FOUND;
  *node_out = g_store.nodes[node_id - 1U];
  return YAI_MIND_OK;
}

static int backend_edge_get(yai_edge_id_t edge_id, yai_graph_edge_t *edge_out)
{
  if (!edge_out || !yai_edge_id_is_valid(edge_id) || (size_t)edge_id > g_store.edge_count) {
    return YAI_MIND_ERR_NOT_FOUND;
  }
  *edge_out = g_store.edges[edge_id - 1U];
  return YAI_MIND_OK;
}

static int backend_query(const yai_memory_query_t *query, yai_memory_result_t *result_out)
{
  size_t matches = 0;
  if (!query || !result_out || !query->query[0]) return YAI_MIND_ERR_INVALID_ARG;

  for (size_t i = 0; i < g_store.node_count; i++) {
    const yai_graph_node_t *n = &g_store.nodes[i];
    if (strstr(n->key, query->query) || strstr(n->value, query->query) || strstr(n->domain, query->query)) {
      matches++;
    }
  }

  result_out->match_count = (int)matches;
  snprintf(result_out->summary, sizeof(result_out->summary), "query=%.200s matches=%zu", query->query, matches);
  return YAI_MIND_OK;
}

static int backend_stats(yai_graph_stats_t *stats_out)
{
  if (!stats_out) return YAI_MIND_ERR_INVALID_ARG;
  stats_out->node_count = g_store.node_count;
  stats_out->edge_count = g_store.edge_count;
  return YAI_MIND_OK;
}

static void backend_shutdown(void)
{
  free(g_store.nodes);
  free(g_store.edges);
  memset(&g_store, 0, sizeof(g_store));
}

static const yai_graph_backend_ops_t g_ops = {
  .node_create = backend_node_create,
  .edge_create = backend_edge_create,
  .node_get = backend_node_get,
  .edge_get = backend_edge_get,
  .query = backend_query,
  .stats = backend_stats,
  .shutdown = backend_shutdown,
};

static const yai_graph_backend_ops_t *g_selected = NULL;
static const char *g_backend_name = "none";

int yai_graph_backend_select_inmemory(void)
{
  if (g_selected && g_selected->shutdown) g_selected->shutdown();
  g_store.next_node_id = 1;
  g_store.next_edge_id = 1;
  g_selected = &g_ops;
  g_backend_name = "inmemory";
  return YAI_MIND_OK;
}

const yai_graph_backend_ops_t *yai_graph_backend_ops(void)
{
  return g_selected;
}

const char *yai_graph_backend_name_internal(void)
{
  return g_backend_name;
}

void yai_graph_backend_shutdown_internal(void)
{
  if (g_selected && g_selected->shutdown) g_selected->shutdown();
  g_selected = NULL;
  g_backend_name = "none";
}
