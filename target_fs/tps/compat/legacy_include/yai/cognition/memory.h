/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <stddef.h>

#include <yai/cognition/errors.h>
#include <yai/cognition/types.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct yai_arena {
  unsigned char *buffer;
  size_t size;
  size_t offset;
} yai_arena_t;

typedef struct yai_graph_node {
  yai_node_id_t node_id;
  char domain[24];
  char key[64];
  char value[256];
} yai_graph_node_t;

typedef struct yai_graph_edge {
  yai_edge_id_t edge_id;
  yai_node_id_t from_node;
  yai_node_id_t to_node;
  char relation[48];
  float weight;
} yai_graph_edge_t;

typedef struct yai_graph_stats {
  size_t node_count;
  size_t edge_count;
} yai_graph_stats_t;

typedef struct yai_activation_record {
  yai_node_id_t node_id;
  float score;
  char source[64];
} yai_activation_record_t;

typedef struct yai_activation_trace {
  unsigned long tick;
  char detail[128];
} yai_activation_trace_t;

typedef struct yai_authority_record {
  yai_node_id_t node_id;
  char policy[64];
  int level;
} yai_authority_record_t;

typedef struct yai_episodic_record {
  char episode_id[64];
  yai_node_id_t node_id;
  char summary[128];
} yai_episodic_record_t;

typedef struct yai_semantic_record {
  char term[64];
  char definition[256];
  yai_node_id_t node_id;
} yai_semantic_record_t;

#define YAI_MIND_VECTOR_MAX_DIM 16

typedef struct yai_vector_item {
  yai_node_id_t node_id;
  size_t dim;
  float values[YAI_MIND_VECTOR_MAX_DIM];
} yai_vector_item_t;

int yai_memory_init(void);
int yai_memory_shutdown(void);
/* Canonical knowledge-owned memory lifecycle entrypoints. */
int yai_knowledge_memory_start(void);
int yai_knowledge_memory_stop(void);

int yai_arena_init(yai_arena_t *arena, size_t size);
void *yai_arena_alloc(yai_arena_t *arena, size_t size, size_t alignment);
void yai_arena_reset(yai_arena_t *arena);
void yai_arena_destroy(yai_arena_t *arena);

int yai_node_id_is_valid(yai_node_id_t node_id);
int yai_edge_id_is_valid(yai_edge_id_t edge_id);

int yai_graph_node_create(const char *domain,
                               const char *key,
                               const char *value,
                               yai_node_id_t *node_id_out);
int yai_graph_edge_create(yai_node_id_t from_node,
                               yai_node_id_t to_node,
                               const char *relation,
                               float weight,
                               yai_edge_id_t *edge_id_out);
int yai_graph_node_get(yai_node_id_t node_id, yai_graph_node_t *node_out);
int yai_graph_edge_get(yai_edge_id_t edge_id, yai_graph_edge_t *edge_out);
int yai_graph_stats_get(yai_graph_stats_t *stats_out);
int yai_memory_query_run(const yai_memory_query_t *query,
                              yai_memory_result_t *result_out);

int yai_storage_bridge_query(const yai_memory_query_t *query,
                                  yai_memory_result_t *result);
int yai_storage_bridge_resolution_hook(const char *workspace_id,
                                            const char *family_id,
                                            const char *specialization_id,
                                            const char *effect,
                                            const char *authority_profile,
                                            const char *resource_hint,
                                            const char *governance_refs_csv,
                                            const char *authority_ref,
                                            const char *artifact_ref,
                                            const char *event_ref,
                                            const char *decision_ref,
                                            const char *evidence_ref,
                                            char *err,
                                            size_t err_cap);
int yai_storage_bridge_last_refs(const char *workspace_id,
                                      char *graph_node_ref,
                                      size_t graph_node_ref_cap,
                                      char *graph_edge_ref,
                                      size_t graph_edge_ref_cap,
                                      char *transient_state_ref,
                                      size_t transient_state_ref_cap,
                                      char *transient_working_set_ref,
                                      size_t transient_working_set_ref_cap,
                                      char *graph_store_ref,
                                      size_t graph_store_ref_cap,
                                      char *transient_store_ref,
                                      size_t transient_store_ref_cap);

int yai_graph_backend_use_inmemory(void);
int yai_graph_backend_use_rpc(const char *endpoint);
const char *yai_graph_backend_name(void);

int yai_domain_activation_record(yai_node_id_t node_id,
                                      float score,
                                      const char *source);
int yai_domain_activation_last(yai_activation_record_t *record_out,
                                    yai_activation_trace_t *trace_out);

int yai_domain_authority_grant(yai_node_id_t node_id,
                                    const char *policy,
                                    int level);
int yai_domain_authority_get(yai_node_id_t node_id,
                                  yai_authority_record_t *record_out);

int yai_domain_episodic_append(const char *episode_id,
                                    yai_node_id_t node_id,
                                    const char *summary);
int yai_domain_episodic_latest(yai_episodic_record_t *record_out);

int yai_domain_semantic_put(const char *term,
                                 const char *definition,
                                 yai_node_id_t node_id);
int yai_domain_semantic_get(const char *term,
                                 yai_semantic_record_t *record_out);

int yai_domain_vector_upsert(yai_node_id_t node_id,
                                  const float *values,
                                  size_t dim);
int yai_domain_vector_nearest(const float *values,
                                   size_t dim,
                                   yai_node_id_t *node_id_out,
                                   float *distance_out);

#ifdef __cplusplus
}
#endif
