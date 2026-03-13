/* SPDX-License-Identifier: Apache-2.0 */

#include <yai/graph/materialization.h>
#include <yai/protocol/control/source_plane.h>

#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <cJSON.h>

#if defined(YAI_HAVE_HIREDIS)
#include <hiredis/hiredis.h>
#endif

typedef struct yai_graph_workspace_counts {
  char workspace_id[64];
  size_t node_count;
  size_t edge_count;
  size_t source_node_count;
  size_t source_edge_count;
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
  g_workspace_counts[g_workspace_count_n].source_node_count = 0;
  g_workspace_counts[g_workspace_count_n].source_edge_count = 0;
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

  (void)yai_graph_node_create("workspace", workspace_id, "runtime_scope", &out->workspace_node);
  (void)yai_graph_node_create("governance", governance_slug, governance_ref && governance_ref[0] ? governance_ref : "none", &out->governance_node);
  (void)yai_graph_node_create("decision", decision_slug, effect && effect[0] ? effect : "unknown", &out->decision_node);
  (void)yai_graph_node_create("evidence", evidence_slug, evidence_ref && evidence_ref[0] ? evidence_ref : "none", &out->evidence_node);
  (void)yai_graph_node_create("authority", authority_slug, authority_profile && authority_profile[0] ? authority_profile : "unknown", &out->authority_node);
  (void)yai_graph_node_create("artifact", artifact_slug, resource_hint && resource_hint[0] ? resource_hint : "unknown", &out->artifact_node);
  (void)yai_graph_node_create("episode", event_slug, family_id && family_id[0] ? family_id : "unknown", &out->episode_node);

  (void)yai_graph_edge_create(out->decision_node, out->workspace_node, "decision_in_workspace", 1.0f, &out->edge_decision_in_workspace);
  (void)yai_graph_edge_create(out->decision_node, out->governance_node, "decision_under_governance", 1.0f, &out->edge_decision_under_governance);
  (void)yai_graph_edge_create(out->decision_node, out->authority_node, "decision_under_authority", 1.0f, &out->edge_decision_under_authority);
  (void)yai_graph_edge_create(out->decision_node, out->artifact_node, "decision_on_artifact", 1.0f, &out->edge_decision_on_artifact);
  (void)yai_graph_edge_create(out->evidence_node, out->decision_node, "evidence_for_decision", 1.0f, &out->edge_evidence_for_decision);
  (void)yai_graph_edge_create(out->artifact_node, out->governance_node, "artifact_governed_by", 1.0f, &out->edge_artifact_governed_by);
  (void)yai_graph_edge_create(out->workspace_node, out->governance_node, "workspace_uses_governance", 1.0f, &out->edge_workspace_uses_governance);
  (void)yai_graph_edge_create(out->episode_node, out->decision_node, "episode_yielded_decision", 1.0f, &out->edge_episode_yielded_decision);

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

int yai_graph_materialization_workspace_source_counts(const char *workspace_id,
                                                      size_t *source_node_count_out,
                                                      size_t *source_edge_count_out)
{
  yai_graph_workspace_counts_t *slot;
  if (source_node_count_out) *source_node_count_out = 0;
  if (source_edge_count_out) *source_edge_count_out = 0;
  if (!workspace_id || !workspace_id[0]) return -1;
  slot = workspace_counts_slot(workspace_id);
  if (!slot) return -1;
  if (source_node_count_out) *source_node_count_out = slot->source_node_count;
  if (source_edge_count_out) *source_edge_count_out = slot->source_edge_count;
  return 0;
}

static const char *json_string(cJSON *root, const char *key)
{
  cJSON *v = NULL;
  if (!root || !key) return NULL;
  v = cJSON_GetObjectItemCaseSensitive(root, key);
  if (cJSON_IsString(v) && v->valuestring && v->valuestring[0]) return v->valuestring;
  return NULL;
}

int yai_graph_materialize_source_record(const char *workspace_id,
                                        const char *record_class,
                                        const char *record_json,
                                        char *out_node_ref,
                                        size_t out_node_ref_cap,
                                        char *out_edge_ref,
                                        size_t out_edge_ref_cap,
                                        char *err,
                                        size_t err_cap)
{
  yai_graph_workspace_counts_t *slot = NULL;
  cJSON *root = NULL;
  const char *source_node_id = NULL;
  const char *daemon_instance_id = NULL;
  const char *source_binding_id = NULL;
  const char *source_asset_id = NULL;
  const char *source_event_id = NULL;
  const char *source_candidate_id = NULL;
  const char *owner_ws_id = NULL;
  yai_node_id_t ws_node = 0;
  yai_node_id_t src_node = 0;
  yai_node_id_t daemon_node = 0;
  yai_node_id_t binding_node = 0;
  yai_node_id_t scope_node = 0;
  yai_node_id_t asset_node = 0;
  yai_node_id_t event_node = 0;
  yai_node_id_t candidate_node = 0;
  yai_edge_id_t rel_edge = 0;
  char ws_slug[96];
  char src_slug[128];
  char a_slug[128];
  char b_slug[128];
  char c_slug[128];

  if (err && err_cap > 0) err[0] = '\0';
  if (out_node_ref && out_node_ref_cap > 0) out_node_ref[0] = '\0';
  if (out_edge_ref && out_edge_ref_cap > 0) out_edge_ref[0] = '\0';
  if (!workspace_id || !workspace_id[0] || !record_class || !record_class[0] || !record_json || !record_json[0]) {
    if (err && err_cap > 0) snprintf(err, err_cap, "%s", "source_graph_bad_args");
    return -1;
  }
  if (!yai_source_record_class_is_known(record_class)) {
    if (err && err_cap > 0) snprintf(err, err_cap, "%s", "source_graph_unknown_record_class");
    return -1;
  }

  root = cJSON_Parse(record_json);
  if (!root) {
    if (err && err_cap > 0) snprintf(err, err_cap, "%s", "source_graph_record_not_json");
    return -1;
  }

  source_node_id = json_string(root, "source_node_id");
  daemon_instance_id = json_string(root, "daemon_instance_id");
  source_binding_id = json_string(root, "source_binding_id");
  source_asset_id = json_string(root, "source_asset_id");
  source_event_id = json_string(root, "source_acquisition_event_id");
  source_candidate_id = json_string(root, "source_evidence_candidate_id");
  owner_ws_id = json_string(root, "owner_workspace_id");
  if (!owner_ws_id || !owner_ws_id[0]) owner_ws_id = workspace_id;

  yai_graph_slug(owner_ws_id, ws_slug, sizeof(ws_slug));
  (void)yai_graph_node_create("owner_workspace", ws_slug, owner_ws_id, &ws_node);
  slot = workspace_counts_slot(workspace_id);

  if (strcmp(record_class, YAI_SOURCE_RECORD_CLASS_NODE) == 0)
  {
    yai_graph_slug(source_node_id, src_slug, sizeof(src_slug));
    (void)yai_graph_node_create(YAI_GRAPH_SOURCE_NODE_CLASS,
                                     src_slug,
                                     source_node_id ? source_node_id : "source_node_unset",
                                     &src_node);
    (void)yai_graph_edge_create(src_node, ws_node, "attached_to", 1.0f, &rel_edge);
    if (slot) { slot->source_node_count += 2; slot->source_edge_count += 1; }
    if (out_node_ref && out_node_ref_cap > 0) snprintf(out_node_ref, out_node_ref_cap, "bgn-%s-%s", YAI_GRAPH_SOURCE_NODE_CLASS, src_slug);
    if (out_edge_ref && out_edge_ref_cap > 0) snprintf(out_edge_ref, out_edge_ref_cap, "bge-attached-to-%s", src_slug);
  }
  else if (strcmp(record_class, YAI_SOURCE_RECORD_CLASS_DAEMON_INSTANCE) == 0)
  {
    yai_graph_slug(source_node_id, src_slug, sizeof(src_slug));
    yai_graph_slug(daemon_instance_id, a_slug, sizeof(a_slug));
    (void)yai_graph_node_create(YAI_GRAPH_SOURCE_NODE_CLASS, src_slug, source_node_id ? source_node_id : "source_node_unset", &src_node);
    (void)yai_graph_node_create(YAI_GRAPH_SOURCE_DAEMON_INSTANCE_CLASS, a_slug, daemon_instance_id ? daemon_instance_id : "daemon_instance_unset", &daemon_node);
    (void)yai_graph_edge_create(daemon_node, src_node, "runs_on", 1.0f, &rel_edge);
    if (slot) { slot->source_node_count += 2; slot->source_edge_count += 1; }
    if (out_node_ref && out_node_ref_cap > 0) snprintf(out_node_ref, out_node_ref_cap, "bgn-%s-%s", YAI_GRAPH_SOURCE_DAEMON_INSTANCE_CLASS, a_slug);
    if (out_edge_ref && out_edge_ref_cap > 0) snprintf(out_edge_ref, out_edge_ref_cap, "bge-runs-on-%s", a_slug);
  }
  else if (strcmp(record_class, YAI_SOURCE_RECORD_CLASS_BINDING) == 0)
  {
    yai_graph_slug(source_binding_id, a_slug, sizeof(a_slug));
    yai_graph_slug(source_node_id, src_slug, sizeof(src_slug));
    (void)yai_graph_node_create(YAI_GRAPH_SOURCE_BINDING_CLASS, a_slug, source_binding_id ? source_binding_id : "source_binding_unset", &binding_node);
    (void)yai_graph_node_create(YAI_GRAPH_SOURCE_NODE_CLASS, src_slug, source_node_id ? source_node_id : "source_node_unset", &src_node);
    (void)yai_graph_edge_create(binding_node, src_node, "bound_on", 1.0f, &rel_edge);
    (void)yai_graph_edge_create(binding_node, ws_node, "targets_workspace", 1.0f, &rel_edge);
    if (slot) { slot->source_node_count += 3; slot->source_edge_count += 2; }
    if (out_node_ref && out_node_ref_cap > 0) snprintf(out_node_ref, out_node_ref_cap, "bgn-%s-%s", YAI_GRAPH_SOURCE_BINDING_CLASS, a_slug);
    if (out_edge_ref && out_edge_ref_cap > 0) snprintf(out_edge_ref, out_edge_ref_cap, "bge-targets-workspace-%s", a_slug);
  }
  else if (strcmp(record_class, YAI_SOURCE_RECORD_CLASS_ASSET) == 0)
  {
    yai_graph_slug(source_asset_id, a_slug, sizeof(a_slug));
    yai_graph_slug(source_binding_id, b_slug, sizeof(b_slug));
    (void)yai_graph_node_create(YAI_GRAPH_SOURCE_ASSET_CLASS, a_slug, source_asset_id ? source_asset_id : "source_asset_unset", &asset_node);
    (void)yai_graph_node_create(YAI_GRAPH_SOURCE_BINDING_CLASS, b_slug, source_binding_id ? source_binding_id : "source_binding_unset", &binding_node);
    (void)yai_graph_edge_create(asset_node, binding_node, "discovered_via", 1.0f, &rel_edge);
    if (slot) { slot->source_node_count += 2; slot->source_edge_count += 1; }
    if (out_node_ref && out_node_ref_cap > 0) snprintf(out_node_ref, out_node_ref_cap, "bgn-%s-%s", YAI_GRAPH_SOURCE_ASSET_CLASS, a_slug);
    if (out_edge_ref && out_edge_ref_cap > 0) snprintf(out_edge_ref, out_edge_ref_cap, "bge-discovered-via-%s", a_slug);
  }
  else if (strcmp(record_class, YAI_SOURCE_RECORD_CLASS_ACQUISITION_EVENT) == 0)
  {
    yai_graph_slug(source_event_id, a_slug, sizeof(a_slug));
    yai_graph_slug(source_asset_id, b_slug, sizeof(b_slug));
    yai_graph_slug(source_node_id, src_slug, sizeof(src_slug));
    (void)yai_graph_node_create(YAI_GRAPH_SOURCE_ACQUISITION_EVENT_CLASS, a_slug, source_event_id ? source_event_id : "source_event_unset", &event_node);
    (void)yai_graph_node_create(YAI_GRAPH_SOURCE_ASSET_CLASS, b_slug, source_asset_id ? source_asset_id : "source_asset_unset", &asset_node);
    (void)yai_graph_node_create(YAI_GRAPH_SOURCE_NODE_CLASS, src_slug, source_node_id ? source_node_id : "source_node_unset", &src_node);
    (void)yai_graph_edge_create(event_node, asset_node, "observed", 1.0f, &rel_edge);
    (void)yai_graph_edge_create(event_node, src_node, "emitted_by", 1.0f, &rel_edge);
    if (slot) { slot->source_node_count += 3; slot->source_edge_count += 2; }
    if (out_node_ref && out_node_ref_cap > 0) snprintf(out_node_ref, out_node_ref_cap, "bgn-%s-%s", YAI_GRAPH_SOURCE_ACQUISITION_EVENT_CLASS, a_slug);
    if (out_edge_ref && out_edge_ref_cap > 0) snprintf(out_edge_ref, out_edge_ref_cap, "bge-observed-%s", a_slug);
  }
  else if (strcmp(record_class, YAI_SOURCE_RECORD_CLASS_EVIDENCE_CANDIDATE) == 0)
  {
    const char *src_event_ref = json_string(root, "source_acquisition_event_id");
    yai_graph_slug(source_candidate_id, a_slug, sizeof(a_slug));
    yai_graph_slug(src_event_ref, b_slug, sizeof(b_slug));
    (void)yai_graph_node_create(YAI_GRAPH_SOURCE_EVIDENCE_CANDIDATE_CLASS, a_slug, source_candidate_id ? source_candidate_id : "source_candidate_unset", &candidate_node);
    (void)yai_graph_node_create(YAI_GRAPH_SOURCE_ACQUISITION_EVENT_CLASS, b_slug, src_event_ref ? src_event_ref : "source_event_unset", &event_node);
    (void)yai_graph_edge_create(candidate_node, event_node, "derived_from", 1.0f, &rel_edge);
    if (slot) { slot->source_node_count += 2; slot->source_edge_count += 1; }
    if (out_node_ref && out_node_ref_cap > 0) snprintf(out_node_ref, out_node_ref_cap, "bgn-%s-%s", YAI_GRAPH_SOURCE_EVIDENCE_CANDIDATE_CLASS, a_slug);
    if (out_edge_ref && out_edge_ref_cap > 0) snprintf(out_edge_ref, out_edge_ref_cap, "bge-derived-from-%s", a_slug);
  }
  else if (strcmp(record_class, YAI_SOURCE_RECORD_CLASS_OWNER_LINK) == 0)
  {
    const char *owner_ref = json_string(root, "owner_ref");
    const char *source_owner_link_id = json_string(root, "source_owner_link_id");
    yai_graph_slug(source_owner_link_id, a_slug, sizeof(a_slug));
    yai_graph_slug(source_node_id, src_slug, sizeof(src_slug));
    (void)yai_graph_node_create(YAI_GRAPH_SOURCE_OWNER_LINK_CLASS, a_slug, source_owner_link_id ? source_owner_link_id : "source_owner_link_unset", &candidate_node);
    (void)yai_graph_node_create(YAI_GRAPH_SOURCE_NODE_CLASS, src_slug, source_node_id ? source_node_id : "source_node_unset", &src_node);
    (void)yai_graph_edge_create(src_node, ws_node, "attached_to", 1.0f, &rel_edge);
    if (owner_ref && owner_ref[0]) {
      (void)yai_graph_edge_create(candidate_node, src_node, "source_owner_link", 1.0f, &rel_edge);
      if (slot) { slot->source_edge_count += 1; }
    }
    if (slot) { slot->source_node_count += 3; slot->source_edge_count += 1; }
    if (out_node_ref && out_node_ref_cap > 0) snprintf(out_node_ref, out_node_ref_cap, "bgn-%s-%s", YAI_GRAPH_SOURCE_OWNER_LINK_CLASS, a_slug);
    if (out_edge_ref && out_edge_ref_cap > 0) snprintf(out_edge_ref, out_edge_ref_cap, "bge-attached-to-%s", src_slug);
  }
  else if (strcmp(record_class, YAI_SOURCE_RECORD_CLASS_POLICY_SNAPSHOT) == 0)
  {
    const char *source_policy_snapshot_id = json_string(root, "source_policy_snapshot_id");
    const char *daemon_instance_id = json_string(root, "daemon_instance_id");
    const char *distribution_target_ref = json_string(root, "distribution_target_ref");
    yai_graph_slug(source_policy_snapshot_id, a_slug, sizeof(a_slug));
    yai_graph_slug(source_node_id, src_slug, sizeof(src_slug));
    yai_graph_slug(daemon_instance_id, b_slug, sizeof(b_slug));
    (void)yai_graph_node_create(YAI_GRAPH_SOURCE_POLICY_SNAPSHOT_CLASS,
                                     a_slug,
                                     source_policy_snapshot_id ? source_policy_snapshot_id : "source_policy_snapshot_unset",
                                     &candidate_node);
    (void)yai_graph_node_create(YAI_GRAPH_SOURCE_NODE_CLASS,
                                     src_slug,
                                     source_node_id ? source_node_id : "source_node_unset",
                                     &src_node);
    (void)yai_graph_node_create(YAI_GRAPH_SOURCE_DAEMON_INSTANCE_CLASS,
                                     b_slug,
                                     daemon_instance_id ? daemon_instance_id : "source_daemon_instance_unset",
                                     &event_node);
    (void)yai_graph_edge_create(candidate_node, src_node, "snapshot_for_node", 1.0f, &rel_edge);
    (void)yai_graph_edge_create(candidate_node, event_node, "snapshot_for_daemon", 1.0f, &rel_edge);
    (void)yai_graph_edge_create(candidate_node, ws_node, "distributed_by_workspace", 1.0f, &rel_edge);
    if (distribution_target_ref && distribution_target_ref[0])
    {
      yai_graph_slug(distribution_target_ref, c_slug, sizeof(c_slug));
      (void)yai_graph_node_create(YAI_GRAPH_SOURCE_SCOPE_CLASS,
                                       c_slug,
                                       distribution_target_ref,
                                       &scope_node);
      (void)yai_graph_edge_create(candidate_node, scope_node, "distribution_target", 1.0f, &rel_edge);
      if (slot) { slot->source_node_count += 1; slot->source_edge_count += 1; }
    }
    if (slot) { slot->source_node_count += 4; slot->source_edge_count += 3; }
    if (out_node_ref && out_node_ref_cap > 0) snprintf(out_node_ref, out_node_ref_cap, "bgn-%s-%s", YAI_GRAPH_SOURCE_POLICY_SNAPSHOT_CLASS, a_slug);
    if (out_edge_ref && out_edge_ref_cap > 0) snprintf(out_edge_ref, out_edge_ref_cap, "bge-distributed-by-workspace-%s", a_slug);
  }
  else if (strcmp(record_class, YAI_SOURCE_RECORD_CLASS_CAPABILITY_ENVELOPE) == 0)
  {
    const char *source_capability_envelope_id = json_string(root, "source_capability_envelope_id");
    const char *daemon_instance_id = json_string(root, "daemon_instance_id");
    const char *distribution_target_ref = json_string(root, "distribution_target_ref");
    const char *observation_scope = json_string(root, "observation_scope");
    const char *mediation_scope = json_string(root, "mediation_scope");
    const char *enforcement_scope = json_string(root, "enforcement_scope");
    yai_graph_slug(source_capability_envelope_id, a_slug, sizeof(a_slug));
    yai_graph_slug(source_node_id, src_slug, sizeof(src_slug));
    yai_graph_slug(source_binding_id, b_slug, sizeof(b_slug));
    (void)yai_graph_node_create(YAI_GRAPH_SOURCE_CAPABILITY_ENVELOPE_CLASS,
                                     a_slug,
                                     source_capability_envelope_id ? source_capability_envelope_id : "source_capability_envelope_unset",
                                     &candidate_node);
    (void)yai_graph_node_create(YAI_GRAPH_SOURCE_NODE_CLASS,
                                     src_slug,
                                     source_node_id ? source_node_id : "source_node_unset",
                                     &src_node);
    (void)yai_graph_node_create(YAI_GRAPH_SOURCE_BINDING_CLASS,
                                     b_slug,
                                     source_binding_id ? source_binding_id : "source_binding_unset",
                                     &binding_node);
    (void)yai_graph_edge_create(candidate_node, src_node, "envelope_for_node", 1.0f, &rel_edge);
    (void)yai_graph_edge_create(candidate_node, binding_node, "envelope_for_binding", 1.0f, &rel_edge);
    (void)yai_graph_edge_create(candidate_node, ws_node, "distributed_by_workspace", 1.0f, &rel_edge);
    if (daemon_instance_id && daemon_instance_id[0])
    {
      yai_graph_slug(daemon_instance_id, c_slug, sizeof(c_slug));
      (void)yai_graph_node_create(YAI_GRAPH_SOURCE_DAEMON_INSTANCE_CLASS,
                                       c_slug,
                                       daemon_instance_id,
                                       &event_node);
      (void)yai_graph_edge_create(candidate_node, event_node, "envelope_for_daemon", 1.0f, &rel_edge);
      if (slot) { slot->source_node_count += 1; slot->source_edge_count += 1; }
    }
    if (distribution_target_ref && distribution_target_ref[0])
    {
      yai_graph_slug(distribution_target_ref, c_slug, sizeof(c_slug));
      (void)yai_graph_node_create(YAI_GRAPH_SOURCE_SCOPE_CLASS, c_slug, distribution_target_ref, &scope_node);
      (void)yai_graph_edge_create(candidate_node, scope_node, "distribution_target", 1.0f, &rel_edge);
      if (slot) { slot->source_node_count += 1; slot->source_edge_count += 1; }
    }
    if (observation_scope && observation_scope[0])
    {
      yai_graph_slug(observation_scope, c_slug, sizeof(c_slug));
      (void)yai_graph_node_create(YAI_GRAPH_SOURCE_SCOPE_CLASS, c_slug, observation_scope, &scope_node);
      (void)yai_graph_edge_create(candidate_node, scope_node, "delegated_observation_scope", 1.0f, &rel_edge);
      if (slot) { slot->source_node_count += 1; slot->source_edge_count += 1; }
    }
    if (mediation_scope && mediation_scope[0])
    {
      yai_graph_slug(mediation_scope, c_slug, sizeof(c_slug));
      (void)yai_graph_node_create(YAI_GRAPH_SOURCE_SCOPE_CLASS, c_slug, mediation_scope, &scope_node);
      (void)yai_graph_edge_create(candidate_node, scope_node, "delegated_mediation_scope", 1.0f, &rel_edge);
      if (slot) { slot->source_node_count += 1; slot->source_edge_count += 1; }
    }
    if (enforcement_scope && enforcement_scope[0])
    {
      yai_graph_slug(enforcement_scope, c_slug, sizeof(c_slug));
      (void)yai_graph_node_create(YAI_GRAPH_SOURCE_SCOPE_CLASS, c_slug, enforcement_scope, &scope_node);
      (void)yai_graph_edge_create(candidate_node, scope_node, "delegated_enforcement_scope", 1.0f, &rel_edge);
      if (slot) { slot->source_node_count += 1; slot->source_edge_count += 1; }
    }
    if (slot) { slot->source_node_count += 4; slot->source_edge_count += 3; }
    if (out_node_ref && out_node_ref_cap > 0) snprintf(out_node_ref, out_node_ref_cap, "bgn-%s-%s", YAI_GRAPH_SOURCE_CAPABILITY_ENVELOPE_CLASS, a_slug);
    if (out_edge_ref && out_edge_ref_cap > 0) snprintf(out_edge_ref, out_edge_ref_cap, "bge-envelope-for-binding-%s", a_slug);
  }
  else if (strcmp(record_class, YAI_SOURCE_RECORD_CLASS_WORKSPACE_PEER_MEMBERSHIP) == 0)
  {
    const char *membership_id = json_string(root, "workspace_peer_membership_id");
    const char *coverage_ref = json_string(root, "coverage_ref");
    const char *overlap_state = json_string(root, "overlap_state");
    yai_graph_slug(membership_id, a_slug, sizeof(a_slug));
    yai_graph_slug(source_node_id, src_slug, sizeof(src_slug));
    yai_graph_slug(source_binding_id, b_slug, sizeof(b_slug));
    (void)yai_graph_node_create(YAI_GRAPH_WORKSPACE_PEER_MEMBERSHIP_CLASS,
                                     a_slug,
                                     membership_id ? membership_id : "workspace_peer_membership_unset",
                                     &candidate_node);
    (void)yai_graph_node_create(YAI_GRAPH_SOURCE_NODE_CLASS,
                                     src_slug,
                                     source_node_id ? source_node_id : "source_node_unset",
                                     &src_node);
    (void)yai_graph_node_create(YAI_GRAPH_SOURCE_BINDING_CLASS,
                                     b_slug,
                                     source_binding_id ? source_binding_id : "source_binding_unset",
                                     &binding_node);
    (void)yai_graph_edge_create(candidate_node, ws_node, "member_of_workspace", 1.0f, &rel_edge);
    (void)yai_graph_edge_create(candidate_node, src_node, "membership_source_node", 1.0f, &rel_edge);
    (void)yai_graph_edge_create(candidate_node, binding_node, "membership_binding", 1.0f, &rel_edge);
    if (coverage_ref && coverage_ref[0])
    {
      yai_graph_slug(coverage_ref, c_slug, sizeof(c_slug));
      (void)yai_graph_node_create(YAI_GRAPH_SOURCE_SCOPE_CLASS,
                                       c_slug,
                                       coverage_ref,
                                       &scope_node);
      (void)yai_graph_edge_create(candidate_node, scope_node, "membership_covers_scope", 1.0f, &rel_edge);
      (void)yai_graph_edge_create(binding_node, scope_node, "binding_scope", 1.0f, &rel_edge);
      if (overlap_state && overlap_state[0] &&
          (strcmp(overlap_state, "overlap_possible") == 0 ||
           strcmp(overlap_state, "overlap_confirmed") == 0))
      {
        (void)yai_graph_edge_create(src_node, scope_node, "overlap_on_scope", 1.0f, &rel_edge);
        if (slot) { slot->source_edge_count += 1; }
      }
      if (slot) { slot->source_node_count += 1; slot->source_edge_count += 2; }
    }
    if (slot) { slot->source_node_count += 4; slot->source_edge_count += 3; }
    if (out_node_ref && out_node_ref_cap > 0) snprintf(out_node_ref, out_node_ref_cap, "bgn-%s-%s", YAI_GRAPH_WORKSPACE_PEER_MEMBERSHIP_CLASS, a_slug);
    if (out_edge_ref && out_edge_ref_cap > 0) snprintf(out_edge_ref, out_edge_ref_cap, "bge-member-of-workspace-%s", a_slug);
  }
  else if (strcmp(record_class, YAI_SOURCE_RECORD_CLASS_INGEST_OUTCOME) == 0)
  {
    const char *outcome_id = json_string(root, "source_ingest_outcome_id");
    yai_graph_slug(outcome_id, a_slug, sizeof(a_slug));
    yai_graph_slug(source_node_id, src_slug, sizeof(src_slug));
    yai_graph_slug(source_binding_id, b_slug, sizeof(b_slug));
    (void)yai_graph_node_create(YAI_GRAPH_SOURCE_INGEST_OUTCOME_CLASS,
                                     a_slug,
                                     outcome_id ? outcome_id : "source_ingest_outcome_unset",
                                     &candidate_node);
    (void)yai_graph_node_create(YAI_GRAPH_SOURCE_NODE_CLASS,
                                     src_slug,
                                     source_node_id ? source_node_id : "source_node_unset",
                                     &src_node);
    (void)yai_graph_node_create(YAI_GRAPH_SOURCE_BINDING_CLASS,
                                     b_slug,
                                     source_binding_id ? source_binding_id : "source_binding_unset",
                                     &binding_node);
    (void)yai_graph_edge_create(candidate_node, src_node, "ingest_outcome_for_node", 1.0f, &rel_edge);
    (void)yai_graph_edge_create(candidate_node, binding_node, "ingest_outcome_for_binding", 1.0f, &rel_edge);
    if (slot) { slot->source_node_count += 3; slot->source_edge_count += 2; }
    if (out_node_ref && out_node_ref_cap > 0) snprintf(out_node_ref, out_node_ref_cap, "bgn-%s-%s", YAI_GRAPH_SOURCE_INGEST_OUTCOME_CLASS, a_slug);
    if (out_edge_ref && out_edge_ref_cap > 0) snprintf(out_edge_ref, out_edge_ref_cap, "bge-ingest-outcome-for-node-%s", a_slug);
  }

  cJSON_Delete(root);
  return 0;
}
