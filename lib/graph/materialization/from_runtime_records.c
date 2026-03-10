/* SPDX-License-Identifier: Apache-2.0 */

#include <yai/graph/materialization.h>

#include <stdlib.h>
#include <stdio.h>
#include <string.h>

#if defined(YAI_HAVE_HIREDIS)
#include <hiredis/hiredis.h>
#endif

typedef struct yai_graph_workspace_counts {
  char workspace_id[64];
  size_t node_count;
  size_t edge_count;
} yai_graph_workspace_counts_t;

static yai_graph_workspace_counts_t g_workspace_counts[128];
static size_t g_workspace_count_n = 0;

static yai_graph_workspace_counts_t *workspace_counts_slot(const char *workspace_id)
{
  size_t i;
  if (!workspace_id || !workspace_id[0]) return NULL;
  for (i = 0; i < g_workspace_count_n; ++i) {
    if (strcmp(g_workspace_counts[i].workspace_id, workspace_id) == 0) return &g_workspace_counts[i];
  }
  if (g_workspace_count_n >= (sizeof(g_workspace_counts) / sizeof(g_workspace_counts[0]))) return NULL;
  snprintf(g_workspace_counts[g_workspace_count_n].workspace_id,
           sizeof(g_workspace_counts[g_workspace_count_n].workspace_id),
           "%s",
           workspace_id);
  g_workspace_counts[g_workspace_count_n].node_count = 0;
  g_workspace_counts[g_workspace_count_n].edge_count = 0;
  g_workspace_count_n++;
  return &g_workspace_counts[g_workspace_count_n - 1];
}

static void yai_graph_slug(const char *in, char *out, size_t out_cap)
{
  size_t j = 0;
  if (!out || out_cap == 0) return;
  if (!in || !in[0]) {
    snprintf(out, out_cap, "%s", "none");
    return;
  }
  for (size_t i = 0; in[i] && j + 1 < out_cap; i++) {
    unsigned char c = (unsigned char)in[i];
    if ((c >= 'a' && c <= 'z') || (c >= '0' && c <= '9')) out[j++] = (char)c;
    else if (c >= 'A' && c <= 'Z') out[j++] = (char)(c + 32);
    else if (c == '-' || c == '_' || c == '.' || c == '/') out[j++] = '-';
  }
  out[j] = '\0';
  if (j == 0) snprintf(out, out_cap, "%s", "none");
}

static void yai_graph_redis_emit(const char *workspace_id,
                                 const yai_graph_materialization_result_t *m)
{
#if defined(YAI_HAVE_HIREDIS)
  redisContext *ctx;
  redisReply *reply = NULL;
  char key[192];
  char payload[384];
  const char *enable = getenv("YAI_REDIS_ENABLE");
  const char *host = getenv("YAI_REDIS_HOST");
  const char *port_env = getenv("YAI_REDIS_PORT");
  int port = 6379;
  if (!enable || strcmp(enable, "1") != 0) return;
  if (!workspace_id || !workspace_id[0] || !m) return;
  if (port_env && port_env[0]) port = atoi(port_env);
  if (!host || !host[0]) host = "127.0.0.1";
  ctx = redisConnect(host, port);
  if (!ctx || ctx->err) {
    if (ctx) redisFree(ctx);
    return;
  }
  (void)snprintf(key, sizeof(key), "yai:graph:%s:materialization", workspace_id);
  (void)snprintf(payload,
                 sizeof(payload),
                 "{\"workspace_id\":\"%s\",\"last_graph_node_ref\":\"%s\",\"last_graph_edge_ref\":\"%s\"}",
                 workspace_id,
                 m->last_graph_node_ref,
                 m->last_graph_edge_ref);
  reply = redisCommand(ctx, "LPUSH %s %s", key, payload);
  if (reply) freeReplyObject(reply);
  reply = redisCommand(ctx, "LTRIM %s 0 255", key);
  if (reply) freeReplyObject(reply);
  redisFree(ctx);
#else
  (void)workspace_id;
  (void)m;
#endif
}

int yai_graph_materialize_runtime_record_bundle(const char *workspace_id,
                                                const char *family_id,
                                                const char *specialization_id,
                                                const char *effect,
                                                const char *authority_profile,
                                                const char *resource_hint,
                                                const char *governance_ref,
                                                const char *authority_ref,
                                                const char *artifact_ref,
                                                const char *event_ref,
                                                const char *decision_ref,
                                                const char *evidence_ref,
                                                yai_graph_materialization_result_t *out,
                                                char *err,
                                                size_t err_cap)
{
  char governance_slug[96];
  char decision_slug[96];
  char evidence_slug[96];
  char event_slug[96];
  char authority_slug[96];
  char artifact_slug[96];
  (void)family_id;
  (void)specialization_id;
  if (err && err_cap > 0) err[0] = '\0';
  if (!workspace_id || !workspace_id[0] || !out) {
    if (err && err_cap > 0) snprintf(err, err_cap, "%s", "graph_materialization_bad_args");
    return YAI_MIND_ERR_INVALID_ARG;
  }
  memset(out, 0, sizeof(*out));

  yai_graph_slug(governance_ref, governance_slug, sizeof(governance_slug));
  yai_graph_slug(decision_ref, decision_slug, sizeof(decision_slug));
  yai_graph_slug(evidence_ref, evidence_slug, sizeof(evidence_slug));
  yai_graph_slug(event_ref, event_slug, sizeof(event_slug));
  yai_graph_slug(authority_ref, authority_slug, sizeof(authority_slug));
  yai_graph_slug(artifact_ref, artifact_slug, sizeof(artifact_slug));

  (void)yai_mind_graph_node_create("workspace", workspace_id, "runtime_scope", &out->workspace_node);
  (void)yai_mind_graph_node_create("governance", governance_slug, governance_ref && governance_ref[0] ? governance_ref : "none", &out->governance_node);
  (void)yai_mind_graph_node_create("decision", decision_slug, effect && effect[0] ? effect : "unknown", &out->decision_node);
  (void)yai_mind_graph_node_create("evidence", evidence_slug, evidence_ref && evidence_ref[0] ? evidence_ref : "none", &out->evidence_node);
  (void)yai_mind_graph_node_create("authority", authority_slug, authority_profile && authority_profile[0] ? authority_profile : "unknown", &out->authority_node);
  (void)yai_mind_graph_node_create("artifact", artifact_slug, resource_hint && resource_hint[0] ? resource_hint : "unknown", &out->artifact_node);
  (void)yai_mind_graph_node_create("episode", event_slug, family_id && family_id[0] ? family_id : "unknown", &out->episode_node);

  (void)yai_mind_graph_edge_create(out->decision_node, out->workspace_node, "decision_in_workspace", 1.0f, &out->edge_decision_in_workspace);
  (void)yai_mind_graph_edge_create(out->decision_node, out->governance_node, "decision_under_governance", 1.0f, &out->edge_decision_under_governance);
  (void)yai_mind_graph_edge_create(out->decision_node, out->authority_node, "decision_under_authority", 1.0f, &out->edge_decision_under_authority);
  (void)yai_mind_graph_edge_create(out->decision_node, out->artifact_node, "decision_on_artifact", 1.0f, &out->edge_decision_on_artifact);
  (void)yai_mind_graph_edge_create(out->evidence_node, out->decision_node, "evidence_for_decision", 1.0f, &out->edge_evidence_for_decision);
  (void)yai_mind_graph_edge_create(out->artifact_node, out->governance_node, "artifact_governed_by", 1.0f, &out->edge_artifact_governed_by);
  (void)yai_mind_graph_edge_create(out->workspace_node, out->governance_node, "workspace_uses_governance", 1.0f, &out->edge_workspace_uses_governance);
  (void)yai_mind_graph_edge_create(out->episode_node, out->decision_node, "episode_yielded_decision", 1.0f, &out->edge_episode_yielded_decision);

  snprintf(out->last_graph_node_ref, sizeof(out->last_graph_node_ref), "bgn-decision-%s-%s", workspace_id, decision_slug);
  snprintf(out->last_graph_edge_ref, sizeof(out->last_graph_edge_ref), "bge-evidence-for-decision-%s-%s", workspace_id, decision_slug);

  {
    yai_graph_workspace_counts_t *slot = workspace_counts_slot(workspace_id);
    if (slot) {
      slot->node_count += 7;
      slot->edge_count += 8;
    }
  }

  yai_graph_redis_emit(workspace_id, out);
  return YAI_MIND_OK;
}

int yai_graph_materialization_workspace_counts(const char *workspace_id,
                                               size_t *node_count_out,
                                               size_t *edge_count_out)
{
  yai_graph_workspace_counts_t *slot;
  if (node_count_out) *node_count_out = 0;
  if (edge_count_out) *edge_count_out = 0;
  if (!workspace_id || !workspace_id[0]) return -1;
  slot = workspace_counts_slot(workspace_id);
  if (!slot) return -1;
  if (node_count_out) *node_count_out = slot->node_count;
  if (edge_count_out) *edge_count_out = slot->edge_count;
  return 0;
}
