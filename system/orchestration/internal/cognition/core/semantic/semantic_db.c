/* SPDX-License-Identifier: Apache-2.0 */

#include <yai/cognition/memory.h>
#include <yai/graph/internal/state_reset.h>

#include <stdio.h>
#include <string.h>

#define YAI_MIND_SEMANTIC_CAP 256

static yai_semantic_record_t g_semantic[YAI_MIND_SEMANTIC_CAP];
static size_t g_semantic_count = 0;

int yai_domain_semantic_put(const char *term,
                                 const char *definition,
                                 yai_node_id_t node_id)
{
  size_t i;
  if (!term || !term[0] || !definition || !definition[0]) return YAI_MIND_ERR_INVALID_ARG;

  for (i = 0; i < g_semantic_count; i++) {
    if (strcmp(g_semantic[i].term, term) == 0) {
      snprintf(g_semantic[i].definition, sizeof(g_semantic[i].definition), "%s", definition);
      if (yai_node_id_is_valid(node_id)) g_semantic[i].node_id = node_id;
      return YAI_MIND_OK;
    }
  }

  if (g_semantic_count >= YAI_MIND_SEMANTIC_CAP) return YAI_MIND_ERR_STATE;

  if (!yai_node_id_is_valid(node_id)) {
    int rc = yai_graph_node_create("semantic", term, definition, &node_id);
    if (rc != YAI_MIND_OK) return rc;
  }

  snprintf(g_semantic[g_semantic_count].term, sizeof(g_semantic[g_semantic_count].term), "%s", term);
  snprintf(g_semantic[g_semantic_count].definition, sizeof(g_semantic[g_semantic_count].definition), "%s", definition);
  g_semantic[g_semantic_count].node_id = node_id;
  g_semantic_count++;
  return YAI_MIND_OK;
}

int yai_domain_semantic_get(const char *term,
                                 yai_semantic_record_t *record_out)
{
  if (!term || !term[0] || !record_out) return YAI_MIND_ERR_INVALID_ARG;
  for (size_t i = 0; i < g_semantic_count; i++) {
    if (strcmp(g_semantic[i].term, term) == 0) {
      *record_out = g_semantic[i];
      return YAI_MIND_OK;
    }
  }
  return YAI_MIND_ERR_NOT_FOUND;
}

void yai_graph_semantic_reset(void)
{
  memset(g_semantic, 0, sizeof(g_semantic));
  g_semantic_count = 0;
}
