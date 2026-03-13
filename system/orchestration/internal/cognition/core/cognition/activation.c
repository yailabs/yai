/* SPDX-License-Identifier: Apache-2.0 */

#include <yai/cognition/activation.h>

#include <stdio.h>
#include <string.h>

static yai_activation_record_t g_last_record = {0};
static yai_activation_trace_t g_last_trace = {0};

int yai_domain_activation_record(yai_node_id_t node_id,
                                      float score,
                                      const char *source)
{
  if (!yai_node_id_is_valid(node_id) || !source || !source[0]) return YAI_MIND_ERR_INVALID_ARG;
  g_last_record.node_id = node_id;
  g_last_record.score = score;
  snprintf(g_last_record.source, sizeof(g_last_record.source), "%s", source);

  g_last_trace.tick++;
  snprintf(g_last_trace.detail, sizeof(g_last_trace.detail), "activation node=%lu score=%.3f", (unsigned long)node_id, score);
  return YAI_MIND_OK;
}

int yai_domain_activation_last(yai_activation_record_t *record_out,
                                    yai_activation_trace_t *trace_out)
{
  if (!record_out || !trace_out) return YAI_MIND_ERR_INVALID_ARG;
  if (!yai_node_id_is_valid(g_last_record.node_id)) return YAI_MIND_ERR_NOT_FOUND;
  *record_out = g_last_record;
  *trace_out = g_last_trace;
  return YAI_MIND_OK;
}

void yai_graph_activation_reset(void)
{
  memset(&g_last_record, 0, sizeof(g_last_record));
  memset(&g_last_trace, 0, sizeof(g_last_trace));
}
