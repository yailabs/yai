/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct yai_knowledge_session {
  char session_id[64];
  char workspace_id[64];
  char transport[32];
} yai_knowledge_session_t;

typedef struct yai_task {
  char task_id[64];
  char role[32];
  char prompt[256];
} yai_task_t;

typedef struct yai_plan_step {
  char step_id[64];
  char action[64];
  int priority;
} yai_plan_step_t;

typedef struct yai_provider_request {
  char request_id[64];
  char model[64];
  char payload[512];
} yai_provider_request_t;

typedef struct yai_provider_response {
  int status;
  char output[512];
} yai_provider_response_t;

typedef uint64_t yai_node_id_t;
typedef uint64_t yai_edge_id_t;

#define YAI_MIND_NODE_ID_INVALID ((yai_node_id_t)0)
#define YAI_MIND_EDGE_ID_INVALID ((yai_edge_id_t)0)

typedef struct yai_memory_query {
  char workspace_id[64];
  char query[256];
  int limit;
} yai_memory_query_t;

typedef struct yai_memory_result {
  int match_count;
  char summary[256];
} yai_memory_result_t;

#ifdef __cplusplus
}
#endif
