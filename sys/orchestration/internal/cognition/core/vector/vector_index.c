/* SPDX-License-Identifier: Apache-2.0 */

#include <yai/cognition/memory.h>
#include <yai/graph/internal/state_reset.h>

#include <math.h>
#include <string.h>

#define YAI_MIND_VECTOR_CAP 256

static yai_vector_item_t g_items[YAI_MIND_VECTOR_CAP];
static size_t g_count = 0;

static float l2_distance(const float *a, const float *b, size_t dim)
{
  float acc = 0.0f;
  for (size_t i = 0; i < dim; i++) {
    float d = a[i] - b[i];
    acc += d * d;
  }
  return sqrtf(acc);
}

int yai_domain_vector_upsert(yai_node_id_t node_id,
                                  const float *values,
                                  size_t dim)
{
  size_t i;
  if (!yai_node_id_is_valid(node_id) || !values || dim == 0 || dim > YAI_MIND_VECTOR_MAX_DIM) {
    return YAI_MIND_ERR_INVALID_ARG;
  }

  for (i = 0; i < g_count; i++) {
    if (g_items[i].node_id == node_id) {
      g_items[i].dim = dim;
      memcpy(g_items[i].values, values, sizeof(float) * dim);
      return YAI_MIND_OK;
    }
  }

  if (g_count >= YAI_MIND_VECTOR_CAP) return YAI_MIND_ERR_STATE;

  g_items[g_count].node_id = node_id;
  g_items[g_count].dim = dim;
  memcpy(g_items[g_count].values, values, sizeof(float) * dim);
  g_count++;
  return YAI_MIND_OK;
}

int yai_domain_vector_nearest(const float *values,
                                   size_t dim,
                                   yai_node_id_t *node_id_out,
                                   float *distance_out)
{
  float best = 0.0f;
  size_t best_idx = 0;
  int found = 0;

  if (!values || !node_id_out || !distance_out || dim == 0 || dim > YAI_MIND_VECTOR_MAX_DIM) {
    return YAI_MIND_ERR_INVALID_ARG;
  }

  for (size_t i = 0; i < g_count; i++) {
    float d;
    if (g_items[i].dim != dim) continue;
    d = l2_distance(values, g_items[i].values, dim);
    if (!found || d < best) {
      best = d;
      best_idx = i;
      found = 1;
    }
  }

  if (!found) return YAI_MIND_ERR_NOT_FOUND;

  *node_id_out = g_items[best_idx].node_id;
  *distance_out = best;
  return YAI_MIND_OK;
}

void yai_graph_vector_reset(void)
{
  memset(g_items, 0, sizeof(g_items));
  g_count = 0;
}
