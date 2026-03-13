/* SPDX-License-Identifier: Apache-2.0 */

#include <yai/graph/materialization.h>
#include <yai/cognition/memory.h>

#include <ctype.h>
#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <time.h>
#include <unistd.h>

#define YAI_MIND_PATH_MAX 1024

static void yai_format_iso8601_utc(time_t t, char *out, size_t out_cap)
{
  struct tm tmv;
  if (!out || out_cap == 0) return;
  if (gmtime_r(&t, &tmv) == NULL) {
    out[0] = '\0';
    return;
  }
  (void)strftime(out, out_cap, "%Y-%m-%dT%H:%M:%SZ", &tmv);
}

static int yai_mkdirs(const char *path)
{
  char tmp[YAI_MIND_PATH_MAX];
  size_t i;

  if (!path || !path[0]) return -1;
  if (strlen(path) >= sizeof(tmp)) return -1;

  (void)snprintf(tmp, sizeof(tmp), "%s", path);
  for (i = 1; tmp[i]; ++i) {
    if (tmp[i] == '/') {
      tmp[i] = '\0';
      if (tmp[0] != '\0' && mkdir(tmp, 0775) != 0 && errno != EEXIST) return -1;
      tmp[i] = '/';
    }
  }
  if (mkdir(tmp, 0775) != 0 && errno != EEXIST) return -1;
  return 0;
}

static int yai_append_json_line(const char *path, const char *line)
{
  FILE *f;
  char dir[YAI_MIND_PATH_MAX];
  char *slash;

  if (!path || !line || !line[0]) return -1;
  if (strlen(path) >= sizeof(dir)) return -1;

  (void)snprintf(dir, sizeof(dir), "%s", path);
  slash = strrchr(dir, '/');
  if (!slash) return -1;
  *slash = '\0';
  if (yai_mkdirs(dir) != 0) return -1;

  f = fopen(path, "a");
  if (!f) return -1;
  (void)fprintf(f, "%s\n", line);
  fclose(f);
  return 0;
}

static int yai_read_text(const char *path, char *out, size_t out_cap)
{
  FILE *f;
  size_t n;
  if (!path || !out || out_cap == 0) return -1;
  out[0] = '\0';
  f = fopen(path, "r");
  if (!f) return -1;
  n = fread(out, 1, out_cap - 1, f);
  out[n] = '\0';
  fclose(f);
  return 0;
}

static int yai_extract_json_string(const char *json,
                                        const char *key,
                                        char *out,
                                        size_t out_cap)
{
  char pattern[128];
  const char *p;
  const char *start;
  const char *end;
  size_t len;

  if (!json || !key || !out || out_cap == 0) return -1;
  out[0] = '\0';

  (void)snprintf(pattern, sizeof(pattern), "\"%s\":\"", key);
  p = strstr(json, pattern);
  if (!p) return -1;
  start = p + strlen(pattern);
  end = strchr(start, '"');
  if (!end) return -1;
  len = (size_t)(end - start);
  if (len >= out_cap) len = out_cap - 1;
  memcpy(out, start, len);
  out[len] = '\0';
  return 0;
}

static void yai_slugify_token(const char *in, char *out, size_t out_cap)
{
  size_t i;
  size_t j = 0;
  if (!out || out_cap == 0) return;
  out[0] = '\0';
  if (!in || !in[0]) {
    (void)snprintf(out, out_cap, "%s", "unknown");
    return;
  }
  for (i = 0; in[i] && j + 1 < out_cap; ++i) {
    unsigned char c = (unsigned char)in[i];
    if (isalnum(c)) out[j++] = (char)tolower(c);
    else if (c == '-' || c == '_' || c == '.') out[j++] = (char)c;
    else out[j++] = '_';
  }
  out[j] = '\0';
}

static int yai_ws_brain_paths(const char *workspace_id,
                                   char *graph_nodes_log,
                                   size_t graph_nodes_log_cap,
                                   char *graph_edges_log,
                                   size_t graph_edges_log_cap,
                                   char *graph_index,
                                   size_t graph_index_cap,
                                   char *transient_activation_log,
                                   size_t transient_activation_log_cap,
                                   char *transient_working_set_log,
                                   size_t transient_working_set_log_cap,
                                   char *transient_index,
                                   size_t transient_index_cap)
{
  const char *home;
  char base[YAI_MIND_PATH_MAX];

  if (!workspace_id || !workspace_id[0]) return -1;
  if (!graph_nodes_log || !graph_edges_log || !graph_index ||
      !transient_activation_log || !transient_working_set_log || !transient_index) return -1;

  home = getenv("HOME");
  if (!home || !home[0]) home = ".";

  if (snprintf(base, sizeof(base), "%s/.yai/run/%s/runtime", home, workspace_id) <= 0) return -1;

  if (snprintf(graph_nodes_log, graph_nodes_log_cap, "%s/graph/persistent-nodes.v1.ndjson", base) <= 0) return -1;
  if (snprintf(graph_edges_log, graph_edges_log_cap, "%s/graph/persistent-edges.v1.ndjson", base) <= 0) return -1;
  if (snprintf(graph_index, graph_index_cap, "%s/graph/index.v1.json", base) <= 0) return -1;

  if (snprintf(transient_activation_log, transient_activation_log_cap, "%s/transient/activation-state.v1.ndjson", base) <= 0) return -1;
  if (snprintf(transient_working_set_log, transient_working_set_log_cap, "%s/transient/working-set.v1.ndjson", base) <= 0) return -1;
  if (snprintf(transient_index, transient_index_cap, "%s/transient/index.v1.json", base) <= 0) return -1;

  return 0;
}

/* Baseline storage-bridge query hook for memory summaries. */
int yai_storage_bridge_query(const yai_memory_query_t *query,
                                  yai_memory_result_t *result)
{
  if (!query || !result) return YAI_MIND_ERR_INVALID_ARG;

  result->match_count = 0;
  result->summary[0] = '\0';
  return YAI_MIND_OK;
}

static void yai_first_csv_token(const char *csv, char *out, size_t out_cap)
{
  const char *end;
  size_t len;
  if (!out || out_cap == 0) return;
  out[0] = '\0';
  if (!csv || !csv[0]) return;
  end = strchr(csv, ',');
  len = end ? (size_t)(end - csv) : strlen(csv);
  if (len >= out_cap) len = out_cap - 1;
  memcpy(out, csv, len);
  out[len] = '\0';
}

static int yai_log_contains_ref(const char *path, const char *json_key, const char *ref)
{
  char buf[16384];
  char needle[256];
  if (!path || !json_key || !ref || !ref[0]) return 0;
  if (yai_read_text(path, buf, sizeof(buf)) != 0) return 0;
  if (snprintf(needle, sizeof(needle), "\"%s\":\"%s\"", json_key, ref) <= 0) return 0;
  return strstr(buf, needle) != NULL;
}

static int yai_append_unique_json_line(const char *path,
                                            const char *json_key,
                                            const char *ref,
                                            const char *line)
{
  if (yai_log_contains_ref(path, json_key, ref)) return 0;
  return yai_append_json_line(path, line);
}

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
                                            size_t err_cap)
{
  char graph_nodes_log[YAI_MIND_PATH_MAX];
  char graph_edges_log[YAI_MIND_PATH_MAX];
  char graph_index[YAI_MIND_PATH_MAX];
  char transient_activation_log[YAI_MIND_PATH_MAX];
  char transient_working_set_log[YAI_MIND_PATH_MAX];
  char transient_index[YAI_MIND_PATH_MAX];
  char family_slug[64];
  char spec_slug[96];
  char decision_slug[96];
  char evidence_slug[96];
  char event_slug[96];
  char governance_ref_local[128];
  char governance_slug[96];
  char authority_slug[96];
  char artifact_slug[96];
  char resource_slug[96];
  char ts_iso[48];
  char graph_node_ref[160];
  char graph_edge_ref[160];
  char transient_state_ref[160];
  char transient_working_set_ref[160];
  char node_row[4096];
  char edge_row[4096];
  char transient_state_row[2048];
  char transient_working_set_row[2048];
  char graph_index_row[3072];
  char transient_index_row[2048];
  yai_graph_materialization_result_t graph_mat;
  yai_node_id_t workspace_node = YAI_MIND_NODE_ID_INVALID;
  yai_node_id_t governance_node = YAI_MIND_NODE_ID_INVALID;
  yai_node_id_t decision_node = YAI_MIND_NODE_ID_INVALID;
  yai_node_id_t evidence_node = YAI_MIND_NODE_ID_INVALID;
  yai_node_id_t authority_node = YAI_MIND_NODE_ID_INVALID;
  yai_node_id_t artifact_node = YAI_MIND_NODE_ID_INVALID;
  yai_node_id_t episode_node = YAI_MIND_NODE_ID_INVALID;
  yai_edge_id_t edge_decision_in_workspace = YAI_MIND_EDGE_ID_INVALID;
  yai_edge_id_t edge_decision_under_governance = YAI_MIND_EDGE_ID_INVALID;
  yai_edge_id_t edge_decision_under_authority = YAI_MIND_EDGE_ID_INVALID;
  yai_edge_id_t edge_decision_on_artifact = YAI_MIND_EDGE_ID_INVALID;
  yai_edge_id_t edge_evidence_for_decision = YAI_MIND_EDGE_ID_INVALID;
  yai_edge_id_t edge_artifact_governed_by = YAI_MIND_EDGE_ID_INVALID;
  yai_edge_id_t edge_workspace_uses_governance = YAI_MIND_EDGE_ID_INVALID;
  yai_edge_id_t edge_episode_yielded_decision = YAI_MIND_EDGE_ID_INVALID;
  char node_ref_workspace[192];
  char node_ref_governance[192];
  char node_ref_decision[192];
  char node_ref_evidence[192];
  char node_ref_authority[192];
  char node_ref_artifact[192];
  char node_ref_episode[192];
  char edge_ref_decision_in_workspace[192];
  char edge_ref_decision_under_governance[192];
  char edge_ref_decision_under_authority[192];
  char edge_ref_decision_on_artifact[192];
  char edge_ref_evidence_for_decision[192];
  char edge_ref_artifact_governed_by[192];
  char edge_ref_workspace_uses_governance[192];
  char edge_ref_episode_yielded_decision[192];
  yai_graph_stats_t stats = {0};
  time_t now;

  if (err && err_cap > 0) err[0] = '\0';

  if (!workspace_id || !workspace_id[0] || !family_id || !family_id[0]) {
    if (err && err_cap > 0) (void)snprintf(err, err_cap, "%s", "brain_storage_bad_args");
    return YAI_MIND_ERR_INVALID_ARG;
  }

  if (yai_ws_brain_paths(workspace_id,
                              graph_nodes_log,
                              sizeof(graph_nodes_log),
                              graph_edges_log,
                              sizeof(graph_edges_log),
                              graph_index,
                              sizeof(graph_index),
                              transient_activation_log,
                              sizeof(transient_activation_log),
                              transient_working_set_log,
                              sizeof(transient_working_set_log),
                              transient_index,
                              sizeof(transient_index)) != 0) {
    if (err && err_cap > 0) (void)snprintf(err, err_cap, "%s", "brain_storage_path_failed");
    return YAI_MIND_ERR_STATE;
  }

  yai_slugify_token(family_id, family_slug, sizeof(family_slug));
  yai_slugify_token(specialization_id && specialization_id[0] ? specialization_id : "none", spec_slug, sizeof(spec_slug));
  yai_slugify_token(decision_ref && decision_ref[0] ? decision_ref : "none", decision_slug, sizeof(decision_slug));
  yai_slugify_token(evidence_ref && evidence_ref[0] ? evidence_ref : "none", evidence_slug, sizeof(evidence_slug));
  yai_slugify_token(event_ref && event_ref[0] ? event_ref : "none", event_slug, sizeof(event_slug));
  yai_first_csv_token(governance_refs_csv, governance_ref_local, sizeof(governance_ref_local));
  yai_slugify_token(governance_ref_local[0] ? governance_ref_local : "none", governance_slug, sizeof(governance_slug));
  yai_slugify_token(authority_ref && authority_ref[0] ? authority_ref : "none", authority_slug, sizeof(authority_slug));
  yai_slugify_token(artifact_ref && artifact_ref[0] ? artifact_ref : "none", artifact_slug, sizeof(artifact_slug));
  yai_slugify_token(resource_hint && resource_hint[0] ? resource_hint : "none", resource_slug, sizeof(resource_slug));

  now = time(NULL);
  yai_format_iso8601_utc(now, ts_iso, sizeof(ts_iso));

  (void)snprintf(node_ref_workspace, sizeof(node_ref_workspace), "bgn-ws-%s", workspace_id);
  (void)snprintf(node_ref_governance, sizeof(node_ref_governance), "bgn-gov-%s-%s", workspace_id, governance_slug);
  (void)snprintf(node_ref_decision, sizeof(node_ref_decision), "bgn-decision-%s-%s", workspace_id, decision_slug);
  (void)snprintf(node_ref_evidence, sizeof(node_ref_evidence), "bgn-evidence-%s-%s", workspace_id, evidence_slug);
  (void)snprintf(node_ref_authority, sizeof(node_ref_authority), "bgn-authority-%s-%s", workspace_id, authority_slug);
  (void)snprintf(node_ref_artifact, sizeof(node_ref_artifact), "bgn-artifact-%s-%s", workspace_id, artifact_slug);
  (void)snprintf(node_ref_episode, sizeof(node_ref_episode), "bgn-episode-%s-%s", workspace_id, event_slug);

  (void)snprintf(edge_ref_decision_in_workspace, sizeof(edge_ref_decision_in_workspace), "bge-decision-in-workspace-%s-%s", workspace_id, decision_slug);
  (void)snprintf(edge_ref_decision_under_governance, sizeof(edge_ref_decision_under_governance), "bge-decision-under-governance-%s-%s", workspace_id, decision_slug);
  (void)snprintf(edge_ref_decision_under_authority, sizeof(edge_ref_decision_under_authority), "bge-decision-under-authority-%s-%s", workspace_id, decision_slug);
  (void)snprintf(edge_ref_decision_on_artifact, sizeof(edge_ref_decision_on_artifact), "bge-decision-on-artifact-%s-%s", workspace_id, decision_slug);
  (void)snprintf(edge_ref_evidence_for_decision, sizeof(edge_ref_evidence_for_decision), "bge-evidence-for-decision-%s-%s", workspace_id, decision_slug);
  (void)snprintf(edge_ref_artifact_governed_by, sizeof(edge_ref_artifact_governed_by), "bge-artifact-governed-by-%s-%s", workspace_id, decision_slug);
  (void)snprintf(edge_ref_workspace_uses_governance, sizeof(edge_ref_workspace_uses_governance), "bge-workspace-uses-governance-%s-%s", workspace_id, governance_slug);
  (void)snprintf(edge_ref_episode_yielded_decision, sizeof(edge_ref_episode_yielded_decision), "bge-episode-yielded-decision-%s-%s", workspace_id, decision_slug);

  memset(&graph_mat, 0, sizeof(graph_mat));
  if (yai_graph_materialize_runtime_record_bundle(workspace_id,
                                                  family_id,
                                                  specialization_id,
                                                  effect,
                                                  authority_profile,
                                                  resource_hint,
                                                  governance_ref_local[0] ? governance_ref_local : "none",
                                                  authority_ref,
                                                  artifact_ref,
                                                  event_ref,
                                                  decision_ref,
                                                  evidence_ref,
                                                  &graph_mat,
                                                  err,
                                                  err_cap) != YAI_MIND_OK) {
    if (err && err_cap > 0 && err[0] == '\0') (void)snprintf(err, err_cap, "%s", "graph_materialization_failed");
    return YAI_MIND_ERR_STATE;
  }

  workspace_node = graph_mat.workspace_node;
  governance_node = graph_mat.governance_node;
  decision_node = graph_mat.decision_node;
  evidence_node = graph_mat.evidence_node;
  authority_node = graph_mat.authority_node;
  artifact_node = graph_mat.artifact_node;
  episode_node = graph_mat.episode_node;
  edge_decision_in_workspace = graph_mat.edge_decision_in_workspace;
  edge_decision_under_governance = graph_mat.edge_decision_under_governance;
  edge_decision_under_authority = graph_mat.edge_decision_under_authority;
  edge_decision_on_artifact = graph_mat.edge_decision_on_artifact;
  edge_evidence_for_decision = graph_mat.edge_evidence_for_decision;
  edge_artifact_governed_by = graph_mat.edge_artifact_governed_by;
  edge_workspace_uses_governance = graph_mat.edge_workspace_uses_governance;
  edge_episode_yielded_decision = graph_mat.edge_episode_yielded_decision;

  (void)snprintf(graph_node_ref, sizeof(graph_node_ref), "%s",
                 graph_mat.last_graph_node_ref[0] ? graph_mat.last_graph_node_ref : node_ref_decision);
  (void)snprintf(graph_edge_ref, sizeof(graph_edge_ref), "%s",
                 graph_mat.last_graph_edge_ref[0] ? graph_mat.last_graph_edge_ref : edge_ref_evidence_for_decision);
  (void)snprintf(transient_state_ref, sizeof(transient_state_ref), "btc-%s-%s", workspace_id, decision_slug);
  (void)snprintf(transient_working_set_ref, sizeof(transient_working_set_ref), "bws-%s-%s", workspace_id, decision_slug);

  (void)yai_domain_authority_grant(authority_node,
                                        authority_profile && authority_profile[0] ? authority_profile : "unknown",
                                        1);
  (void)yai_domain_episodic_append(decision_ref && decision_ref[0] ? decision_ref : decision_slug,
                                        decision_node,
                                        effect && effect[0] ? effect : "unknown");
  (void)yai_domain_activation_record(decision_node, 0.85f, "resolution_hook");
  (void)yai_graph_stats_get(&stats);

  #define APPEND_NODE_ROW(REF, CLS, NID, ORIGIN_DOMAIN, SOURCE_REF) \
    do { \
      if (snprintf(node_row, sizeof(node_row), \
                   "{\"type\":\"yai.graph_node.v1\",\"schema_version\":\"v1\"," \
                   "\"graph_node_ref\":\"%s\",\"workspace_id\":\"%s\",\"node_class\":\"%s\"," \
                   "\"node_id\":%lu,\"origin_domain\":\"%s\",\"source_record_ref\":\"%s\"," \
                   "\"family_id\":\"%s\",\"specialization_id\":\"%s\",\"event_ref\":\"%s\"," \
                   "\"decision_ref\":\"%s\",\"evidence_ref\":\"%s\",\"governance_ref\":\"%s\"," \
                   "\"authority_ref\":\"%s\",\"artifact_ref\":\"%s\",\"effect\":\"%s\",\"created_at\":\"%s\"}", \
                   REF, workspace_id, CLS, (unsigned long)(NID), ORIGIN_DOMAIN, SOURCE_REF, \
                   family_id, specialization_id && specialization_id[0] ? specialization_id : "none", \
                   event_ref && event_ref[0] ? event_ref : "", \
                   decision_ref && decision_ref[0] ? decision_ref : "", \
                   evidence_ref && evidence_ref[0] ? evidence_ref : "", \
                   governance_ref_local[0] ? governance_ref_local : "", \
                   authority_ref && authority_ref[0] ? authority_ref : "", \
                   artifact_ref && artifact_ref[0] ? artifact_ref : "", \
                   effect && effect[0] ? effect : "unknown", ts_iso) <= 0) { \
        if (err && err_cap > 0) (void)snprintf(err, err_cap, "%s", "brain_graph_node_encode_failed"); \
        return YAI_MIND_ERR_STATE; \
      } \
      if (yai_append_unique_json_line(graph_nodes_log, "graph_node_ref", REF, node_row) != 0) { \
        if (err && err_cap > 0) (void)snprintf(err, err_cap, "%s", "brain_graph_node_append_failed"); \
        return YAI_MIND_ERR_STATE; \
      } \
    } while (0)

  #define APPEND_EDGE_ROW(REF, CLS, EID, FROM, TO) \
    do { \
      if (snprintf(edge_row, sizeof(edge_row), \
                   "{\"type\":\"yai.graph_edge.v1\",\"schema_version\":\"v1\"," \
                   "\"graph_edge_ref\":\"%s\",\"workspace_id\":\"%s\",\"edge_class\":\"%s\"," \
                   "\"edge_id\":%lu,\"source_node_id\":%lu,\"target_node_id\":%lu,\"weight\":1.0," \
                   "\"event_ref\":\"%s\",\"decision_ref\":\"%s\",\"evidence_ref\":\"%s\",\"created_at\":\"%s\"}", \
                   REF, workspace_id, CLS, (unsigned long)(EID), (unsigned long)(FROM), (unsigned long)(TO), \
                   event_ref && event_ref[0] ? event_ref : "", \
                   decision_ref && decision_ref[0] ? decision_ref : "", \
                   evidence_ref && evidence_ref[0] ? evidence_ref : "", \
                   ts_iso) <= 0) { \
        if (err && err_cap > 0) (void)snprintf(err, err_cap, "%s", "brain_graph_edge_encode_failed"); \
        return YAI_MIND_ERR_STATE; \
      } \
      if (yai_append_unique_json_line(graph_edges_log, "graph_edge_ref", REF, edge_row) != 0) { \
        if (err && err_cap > 0) (void)snprintf(err, err_cap, "%s", "brain_graph_edge_append_failed"); \
        return YAI_MIND_ERR_STATE; \
      } \
    } while (0)

  APPEND_NODE_ROW(node_ref_workspace, "workspace_node", workspace_node, "workspace_state", workspace_id);
  APPEND_NODE_ROW(node_ref_governance, "governance_object_node", governance_node, "governance_persistence", governance_ref_local[0] ? governance_ref_local : "none");
  APPEND_NODE_ROW(node_ref_decision, "decision_node", decision_node, "decision_record", decision_ref && decision_ref[0] ? decision_ref : decision_slug);
  APPEND_NODE_ROW(node_ref_evidence, "evidence_node", evidence_node, "evidence_record", evidence_ref && evidence_ref[0] ? evidence_ref : evidence_slug);
  APPEND_NODE_ROW(node_ref_authority, "authority_node", authority_node, "authority_state", authority_ref && authority_ref[0] ? authority_ref : "none");
  APPEND_NODE_ROW(node_ref_artifact, "artifact_node", artifact_node, "artifact_metadata", artifact_ref && artifact_ref[0] ? artifact_ref : "none");
  APPEND_NODE_ROW(node_ref_episode, "runtime_episode_node", episode_node, "runtime_event_record", event_ref && event_ref[0] ? event_ref : "none");

  APPEND_EDGE_ROW(edge_ref_decision_in_workspace, "decision_in_workspace", edge_decision_in_workspace, decision_node, workspace_node);
  APPEND_EDGE_ROW(edge_ref_decision_under_governance, "decision_under_governance", edge_decision_under_governance, decision_node, governance_node);
  APPEND_EDGE_ROW(edge_ref_decision_under_authority, "decision_under_authority", edge_decision_under_authority, decision_node, authority_node);
  APPEND_EDGE_ROW(edge_ref_decision_on_artifact, "decision_on_artifact", edge_decision_on_artifact, decision_node, artifact_node);
  APPEND_EDGE_ROW(edge_ref_evidence_for_decision, "evidence_for_decision", edge_evidence_for_decision, evidence_node, decision_node);
  APPEND_EDGE_ROW(edge_ref_artifact_governed_by, "artifact_governed_by", edge_artifact_governed_by, artifact_node, governance_node);
  APPEND_EDGE_ROW(edge_ref_workspace_uses_governance, "workspace_uses_governance", edge_workspace_uses_governance, workspace_node, governance_node);
  APPEND_EDGE_ROW(edge_ref_episode_yielded_decision, "episode_yielded_decision", edge_episode_yielded_decision, episode_node, decision_node);

  #undef APPEND_NODE_ROW
  #undef APPEND_EDGE_ROW

  if (snprintf(transient_state_row,
               sizeof(transient_state_row),
               "{\"type\":\"yai.transient_cognition_state.v1\",\"schema_version\":\"v1\","
               "\"cognition_state_ref\":\"%s\",\"workspace_id\":\"%s\",\"authoritative\":false,"
               "\"hot_node_refs\":\"%lu,%lu\",\"activation_summary\":\"resolution_hook:%s/%s\","
               "\"ttl_seconds\":900,\"decision_ref\":\"%s\",\"event_ref\":\"%s\",\"updated_at\":\"%s\"}",
               transient_state_ref,
               workspace_id,
               (unsigned long)decision_node,
               (unsigned long)evidence_node,
               family_slug,
               spec_slug,
               decision_ref && decision_ref[0] ? decision_ref : "",
               event_ref && event_ref[0] ? event_ref : "",
               ts_iso) <= 0) {
    if (err && err_cap > 0) (void)snprintf(err, err_cap, "%s", "brain_transient_state_encode_failed");
    return YAI_MIND_ERR_STATE;
  }

  if (snprintf(transient_working_set_row,
               sizeof(transient_working_set_row),
               "{\"type\":\"yai.transient_working_set.v1\",\"schema_version\":\"v1\","
               "\"working_set_ref\":\"%s\",\"workspace_id\":\"%s\",\"authoritative\":false,"
               "\"focus_domain\":\"%s\",\"focus_specialization\":\"%s\",\"focus_resource\":\"%s\","
               "\"decision_ref\":\"%s\",\"evidence_ref\":\"%s\",\"updated_at\":\"%s\"}",
               transient_working_set_ref,
               workspace_id,
               family_id,
               specialization_id && specialization_id[0] ? specialization_id : "none",
               resource_hint && resource_hint[0] ? resource_hint : "unknown",
               decision_ref && decision_ref[0] ? decision_ref : "",
               evidence_ref && evidence_ref[0] ? evidence_ref : "",
               ts_iso) <= 0) {
    if (err && err_cap > 0) (void)snprintf(err, err_cap, "%s", "brain_transient_ws_encode_failed");
    return YAI_MIND_ERR_STATE;
  }

  if (yai_append_json_line(transient_activation_log, transient_state_row) != 0 ||
      yai_append_json_line(transient_working_set_log, transient_working_set_row) != 0) {
    if (err && err_cap > 0) (void)snprintf(err, err_cap, "%s", "brain_storage_append_failed");
    return YAI_MIND_ERR_STATE;
  }

  if (snprintf(graph_index_row,
               sizeof(graph_index_row),
               "{\"type\":\"yai.graph.index.v1\",\"workspace_id\":\"%s\","
               "\"graph_truth_authoritative\":true,\"backend_role\":\"BR-3\","
               "\"materialization_mode\":\"runtime_record_driven\","
               "\"materialization_status\":\"complete\","
               "\"source_record_set\":\"runtime_event,decision,evidence,governance,authority,artifact\","
               "\"node_classes\":\"workspace_node,governance_object_node,decision_node,evidence_node,authority_node,artifact_node,runtime_episode_node\","
               "\"edge_classes\":\"decision_in_workspace,decision_under_governance,decision_under_authority,decision_on_artifact,evidence_for_decision,artifact_governed_by,workspace_uses_governance,episode_yielded_decision\","
               "\"last_graph_node_ref\":\"%s\",\"last_graph_edge_ref\":\"%s\","
               "\"last_event_ref\":\"%s\",\"last_decision_ref\":\"%s\",\"last_evidence_ref\":\"%s\","
               "\"graph_backend\":\"%s\",\"graph_node_count\":%zu,\"graph_edge_count\":%zu,"
               "\"updated_at\":\"%s\",\"stores\":{\"nodes\":\"%s\",\"edges\":\"%s\"}}",
               workspace_id,
               graph_node_ref,
               graph_edge_ref,
               event_ref && event_ref[0] ? event_ref : "",
               decision_ref && decision_ref[0] ? decision_ref : "",
               evidence_ref && evidence_ref[0] ? evidence_ref : "",
               yai_graph_backend_name(),
               stats.node_count,
               stats.edge_count,
               ts_iso,
               graph_nodes_log,
               graph_edges_log) <= 0) {
    if (err && err_cap > 0) (void)snprintf(err, err_cap, "%s", "brain_graph_index_encode_failed");
    return YAI_MIND_ERR_STATE;
  }

  if (snprintf(transient_index_row,
               sizeof(transient_index_row),
               "{\"type\":\"yai.transient.index.v1\",\"workspace_id\":\"%s\","
               "\"authoritative\":false,\"backend_role\":\"BR-4\",\"backend_candidate\":\"redis_class\","
               "\"last_transient_state_ref\":\"%s\",\"last_working_set_ref\":\"%s\","
               "\"last_decision_ref\":\"%s\",\"last_event_ref\":\"%s\",\"updated_at\":\"%s\","
               "\"stores\":{\"activation\":\"%s\",\"working_set\":\"%s\"}}",
               workspace_id,
               transient_state_ref,
               transient_working_set_ref,
               decision_ref && decision_ref[0] ? decision_ref : "",
               event_ref && event_ref[0] ? event_ref : "",
               ts_iso,
               transient_activation_log,
               transient_working_set_log) <= 0) {
    if (err && err_cap > 0) (void)snprintf(err, err_cap, "%s", "brain_transient_index_encode_failed");
    return YAI_MIND_ERR_STATE;
  }

  {
    FILE *f = fopen(graph_index, "w");
    if (!f) {
      if (err && err_cap > 0) (void)snprintf(err, err_cap, "%s", "brain_graph_index_write_failed");
      return YAI_MIND_ERR_STATE;
    }
    (void)fprintf(f, "%s\n", graph_index_row);
    fclose(f);
  }
  {
    FILE *f = fopen(transient_index, "w");
    if (!f) {
      if (err && err_cap > 0) (void)snprintf(err, err_cap, "%s", "brain_transient_index_write_failed");
      return YAI_MIND_ERR_STATE;
    }
    (void)fprintf(f, "%s\n", transient_index_row);
    fclose(f);
  }

  return YAI_MIND_OK;
}

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
                                      size_t transient_store_ref_cap)
{
  char graph_nodes_log[YAI_MIND_PATH_MAX];
  char graph_edges_log[YAI_MIND_PATH_MAX];
  char graph_index[YAI_MIND_PATH_MAX];
  char transient_activation_log[YAI_MIND_PATH_MAX];
  char transient_working_set_log[YAI_MIND_PATH_MAX];
  char transient_index[YAI_MIND_PATH_MAX];
  char buf[4096];

  if (graph_node_ref && graph_node_ref_cap > 0) graph_node_ref[0] = '\0';
  if (graph_edge_ref && graph_edge_ref_cap > 0) graph_edge_ref[0] = '\0';
  if (transient_state_ref && transient_state_ref_cap > 0) transient_state_ref[0] = '\0';
  if (transient_working_set_ref && transient_working_set_ref_cap > 0) transient_working_set_ref[0] = '\0';
  if (graph_store_ref && graph_store_ref_cap > 0) graph_store_ref[0] = '\0';
  if (transient_store_ref && transient_store_ref_cap > 0) transient_store_ref[0] = '\0';

  if (!workspace_id || !workspace_id[0]) return YAI_MIND_ERR_INVALID_ARG;
  if (yai_ws_brain_paths(workspace_id,
                              graph_nodes_log,
                              sizeof(graph_nodes_log),
                              graph_edges_log,
                              sizeof(graph_edges_log),
                              graph_index,
                              sizeof(graph_index),
                              transient_activation_log,
                              sizeof(transient_activation_log),
                              transient_working_set_log,
                              sizeof(transient_working_set_log),
                              transient_index,
                              sizeof(transient_index)) != 0) {
    return YAI_MIND_ERR_STATE;
  }

  if (graph_store_ref && graph_store_ref_cap > 0)
    (void)snprintf(graph_store_ref, graph_store_ref_cap, "%s", graph_nodes_log);
  if (transient_store_ref && transient_store_ref_cap > 0)
    (void)snprintf(transient_store_ref, transient_store_ref_cap, "%s", transient_activation_log);

  if (yai_read_text(graph_index, buf, sizeof(buf)) == 0) {
    if (graph_node_ref && graph_node_ref_cap > 0)
      (void)yai_extract_json_string(buf, "last_graph_node_ref", graph_node_ref, graph_node_ref_cap);
    if (graph_edge_ref && graph_edge_ref_cap > 0)
      (void)yai_extract_json_string(buf, "last_graph_edge_ref", graph_edge_ref, graph_edge_ref_cap);
  }

  if (yai_read_text(transient_index, buf, sizeof(buf)) == 0) {
    if (transient_state_ref && transient_state_ref_cap > 0)
      (void)yai_extract_json_string(buf, "last_transient_state_ref", transient_state_ref, transient_state_ref_cap);
    if (transient_working_set_ref && transient_working_set_ref_cap > 0)
      (void)yai_extract_json_string(buf, "last_working_set_ref", transient_working_set_ref, transient_working_set_ref_cap);
  }

  return YAI_MIND_OK;
}
