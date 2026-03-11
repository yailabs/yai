/* SPDX-License-Identifier: Apache-2.0 */

#include <yai/exec/grounding.h>

#include <yai/data/query.h>
#include <yai/graph/query.h>

#include <stdio.h>
#include <string.h>

int yai_exec_grounding_context_build(const char *workspace_id,
                                     const char *task_prompt,
                                     const char *user_input,
                                     char *out_json,
                                     size_t out_cap,
                                     char *out_reason,
                                     size_t reason_cap)
{
  char op_json[4096];
  char graph_json[4096];
  char qerr[128];
  char gerr[128];
  const char *ws;

  if (out_reason && reason_cap > 0) out_reason[0] = '\0';
  if (!out_json || out_cap == 0)
  {
    return -1;
  }
  out_json[0] = '\0';
  ws = (workspace_id && workspace_id[0]) ? workspace_id : "default";
  op_json[0] = '\0';
  graph_json[0] = '\0';
  qerr[0] = '\0';
  gerr[0] = '\0';

  if (yai_data_query_operational_summary_json(ws, op_json, sizeof(op_json), qerr, sizeof(qerr)) != 0)
  {
    if (snprintf(op_json, sizeof(op_json),
                 "{\"workspace_id\":\"%s\",\"status\":\"unavailable\",\"reason\":\"%s\"}",
                 ws,
                 qerr[0] ? qerr : "operational_summary_unavailable") <= 0)
    {
      if (out_reason && reason_cap > 0) snprintf(out_reason, reason_cap, "%s", "grounding_operational_summary_encode_failed");
      return -1;
    }
  }

  if (yai_graph_query_workspace_unified_summary(ws, graph_json, sizeof(graph_json), gerr, sizeof(gerr)) != 0)
  {
    if (snprintf(graph_json, sizeof(graph_json),
                 "{\"workspace_id\":\"%s\",\"status\":\"unavailable\",\"reason\":\"%s\"}",
                 ws,
                 gerr[0] ? gerr : "unified_graph_summary_unavailable") <= 0)
    {
      if (out_reason && reason_cap > 0) snprintf(out_reason, reason_cap, "%s", "grounding_graph_summary_encode_failed");
      return -1;
    }
  }

  if (snprintf(out_json,
               out_cap,
               "{\"workspace_id\":\"%s\","
               "\"grounding_contract\":\"governed_case_state_v1\","
               "\"adjudication_boundary\":{\"observed\":\"runtime_distributed\",\"accepted\":\"owner_accepted\",\"canonicalized\":\"owner_final_truth\"},"
               "\"task_scope\":{\"task_prompt\":\"%s\",\"user_input\":\"%s\"},"
               "\"operational_summary\":%s,"
               "\"unified_graph_summary\":%s}",
               ws,
               (task_prompt && task_prompt[0]) ? task_prompt : "general_response",
               (user_input && user_input[0]) ? user_input : "",
               op_json,
               graph_json) <= 0)
  {
    if (out_reason && reason_cap > 0) snprintf(out_reason, reason_cap, "%s", "grounding_context_encode_failed");
    return -1;
  }

  return 0;
}
