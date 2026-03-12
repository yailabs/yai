/* SPDX-License-Identifier: Apache-2.0 */

#include <yai/cognition/memory.h>

#include <stdio.h>
#include <string.h>

#define YAI_MIND_AUTHORITY_CAP 128

static yai_authority_record_t g_records[YAI_MIND_AUTHORITY_CAP];
static size_t g_count = 0;

int yai_domain_authority_grant(yai_node_id_t node_id,
                                    const char *policy,
                                    int level)
{
  if (!yai_node_id_is_valid(node_id) || !policy || !policy[0]) return YAI_MIND_ERR_INVALID_ARG;
  for (size_t i = 0; i < g_count; i++) {
    if (g_records[i].node_id == node_id) {
      snprintf(g_records[i].policy, sizeof(g_records[i].policy), "%s", policy);
      g_records[i].level = level;
      return YAI_MIND_OK;
    }
  }
  if (g_count >= YAI_MIND_AUTHORITY_CAP) return YAI_MIND_ERR_STATE;
  g_records[g_count].node_id = node_id;
  g_records[g_count].level = level;
  snprintf(g_records[g_count].policy, sizeof(g_records[g_count].policy), "%s", policy);
  g_count++;
  return YAI_MIND_OK;
}

int yai_domain_authority_get(yai_node_id_t node_id,
                                  yai_authority_record_t *record_out)
{
  if (!yai_node_id_is_valid(node_id) || !record_out) return YAI_MIND_ERR_INVALID_ARG;
  for (size_t i = 0; i < g_count; i++) {
    if (g_records[i].node_id == node_id) {
      *record_out = g_records[i];
      return YAI_MIND_OK;
    }
  }
  return YAI_MIND_ERR_NOT_FOUND;
}

void yai_graph_authority_reset(void)
{
  memset(g_records, 0, sizeof(g_records));
  g_count = 0;
}
