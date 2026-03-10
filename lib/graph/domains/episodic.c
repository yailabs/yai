/* SPDX-License-Identifier: Apache-2.0 */

#include <yai/knowledge/episodic.h>
#include "../state/graph_state_internal.h"

#include <stdio.h>
#include <string.h>

#define YAI_MIND_EPISODIC_CAP 256

static yai_mind_episodic_record_t g_records[YAI_MIND_EPISODIC_CAP];
static size_t g_count = 0;

int yai_mind_domain_episodic_append(const char *episode_id,
                                    yai_mind_node_id_t node_id,
                                    const char *summary)
{
  if (!episode_id || !episode_id[0] || !summary || !summary[0]) return YAI_MIND_ERR_INVALID_ARG;
  if (g_count >= YAI_MIND_EPISODIC_CAP) return YAI_MIND_ERR_STATE;

  if (!yai_mind_node_id_is_valid(node_id)) {
    int rc = yai_mind_graph_node_create("episodic", episode_id, summary, &node_id);
    if (rc != YAI_MIND_OK) return rc;
  }

  snprintf(g_records[g_count].episode_id, sizeof(g_records[g_count].episode_id), "%s", episode_id);
  snprintf(g_records[g_count].summary, sizeof(g_records[g_count].summary), "%s", summary);
  g_records[g_count].node_id = node_id;
  g_count++;
  return YAI_MIND_OK;
}

int yai_mind_domain_episodic_latest(yai_mind_episodic_record_t *record_out)
{
  if (!record_out) return YAI_MIND_ERR_INVALID_ARG;
  if (g_count == 0) return YAI_MIND_ERR_NOT_FOUND;
  *record_out = g_records[g_count - 1U];
  return YAI_MIND_OK;
}

void yai_mind_graph_episodic_reset(void)
{
  memset(g_records, 0, sizeof(g_records));
  g_count = 0;
}
