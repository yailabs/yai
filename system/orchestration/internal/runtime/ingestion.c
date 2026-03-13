#define _POSIX_C_SOURCE 200809L

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#include <yai/container/scope_runtime.h>
#include <yai/data/records.h>
#include <yai/daemon/runtime_source_ids.h>
#include <yai/protocol/control/source_plane.h>
#include <yai/orchestration/ingestion.h>
#include <yai/orchestration/internal/peer_registry_bridge.h>
#include <yai/orchestration/network_bridge.h>
#include <yai/graph/materialization.h>
#include <yai/protocol/control/source_plane.h>

#include "cJSON.h"

static int set_reason(char *out_reason, size_t reason_cap, const char *reason)
{
  if (!out_reason || reason_cap == 0)
  {
    return -1;
  }
  if (!reason)
  {
    out_reason[0] = '\0';
    return 0;
  }
  if (snprintf(out_reason, reason_cap, "%s", reason) >= (int)reason_cap)
  {
    return -1;
  }
  return 0;
}

static const char *json_string(cJSON *root, const char *key)
{
  cJSON *v = NULL;
  if (!root || !key)
  {
    return NULL;
  }
  v = cJSON_GetObjectItem(root, key);
  if (v && cJSON_IsString(v) && v->valuestring && v->valuestring[0])
  {
    return v->valuestring;
  }
  return NULL;
}

static int64_t json_i64(cJSON *root, const char *key, int64_t fallback)
{
  cJSON *v = NULL;
  if (!root || !key)
  {
    return fallback;
  }
  v = cJSON_GetObjectItem(root, key);
  if (v && cJSON_IsNumber(v))
  {
    return (int64_t)v->valuedouble;
  }
  return fallback;
}

static int json_bool(cJSON *root, const char *key, int fallback)
{
  cJSON *v = NULL;
  if (!root || !key)
  {
    return fallback;
  }
  v = cJSON_GetObjectItem(root, key);
  if (cJSON_IsBool(v))
  {
    return cJSON_IsTrue(v) ? 1 : 0;
  }
  if (cJSON_IsNumber(v))
  {
    return v->valueint ? 1 : 0;
  }
  return fallback;
}

static const char *json_string_or(cJSON *root, const char *key, const char *fallback)
{
  const char *v = json_string(root, key);
  return (v && v[0]) ? v : fallback;
}

static const char *enrollment_decision_from_payload(cJSON *payload)
{
  const char *hint = json_string_or(payload, "enrollment_decision_hint", "accept");
  if (strcmp(hint, "reject") == 0)
  {
    return "rejected";
  }
  if (strcmp(hint, "pending") == 0 || strcmp(hint, "review") == 0)
  {
    return "pending";
  }
  return "accepted";
}

static int make_owner_trust_token(const char *source_node_id,
                                  const char *owner_link_id,
                                  char *out,
                                  size_t out_cap)
{
  unsigned long h = 5381UL;
  const unsigned char *p = (const unsigned char *)owner_link_id;
  if (!source_node_id || !source_node_id[0] || !owner_link_id || !owner_link_id[0] || !out || out_cap == 0)
  {
    return -1;
  }
  while (*p)
  {
    h = ((h << 5) + h) + (unsigned long)(*p);
    ++p;
  }
  if (snprintf(out, out_cap, "ogt:%s:%lx", source_node_id, h) >= (int)out_cap)
  {
    return -1;
  }
  return 0;
}

static int owner_trust_token_valid_for_node(cJSON *payload,
                                            const char *source_node_id)
{
  const char *token = json_string(payload, "owner_trust_artifact_token");
  char prefix[144];
  if (!payload || !source_node_id || !source_node_id[0] || !token || !token[0])
  {
    return 0;
  }
  if (snprintf(prefix, sizeof(prefix), "ogt:%s:", source_node_id) >= (int)sizeof(prefix))
  {
    return 0;
  }
  return strncmp(token, prefix, strlen(prefix)) == 0;
}

static int append_source_record(const char *ws_id,
                                const char *record_class,
                                cJSON *obj,
                                char *err,
                                size_t err_cap)
{
  char *raw = NULL;
  char ref[160];
  char graph_node_ref[192];
  char graph_edge_ref[192];
  char graph_err[128];
  if (!ws_id || !record_class || !obj)
  {
    return -1;
  }

  raw = cJSON_PrintUnformatted(obj);
  if (!raw)
  {
    if (err && err_cap > 0)
    {
      (void)snprintf(err, err_cap, "%s", "record_encode_failed");
    }
    return -1;
  }

  if (yai_data_records_append_source_class(ws_id,
                                           record_class,
                                           raw,
                                           ref,
                                           sizeof(ref),
                                           err,
                                           err_cap) != 0)
  {
    cJSON_free(raw);
    return -1;
  }

  if (yai_graph_materialize_source_record(ws_id,
                                          record_class,
                                          raw,
                                          graph_node_ref,
                                          sizeof(graph_node_ref),
                                          graph_edge_ref,
                                          sizeof(graph_edge_ref),
                                          graph_err,
                                          sizeof(graph_err)) != 0)
  {
    if (err && err_cap > 0)
    {
      (void)snprintf(err, err_cap, "%s", graph_err[0] ? graph_err : "source_graph_materialization_failed");
    }
    cJSON_free(raw);
    return -1;
  }
  cJSON_free(raw);
  return 0;
}

#define YAI_SOURCE_EMIT_SEEN_MAX 2048
#define YAI_SOURCE_EMIT_ASSET_MAX 2048

typedef struct yai_source_emit_seen {
  int used;
  char workspace_id[64];
  char source_node_id[YAI_SOURCE_NODE_ID_MAX];
  char source_binding_id[YAI_SOURCE_BINDING_ID_MAX];
  char idempotency_key[YAI_SOURCE_HASH_MAX];
  int64_t first_seen_epoch;
  int64_t last_seen_epoch;
  int replay_count;
} yai_source_emit_seen_t;

typedef struct yai_source_asset_seen {
  int used;
  char workspace_id[64];
  char asset_key[256];
  char source_node_id[YAI_SOURCE_NODE_ID_MAX];
  char source_binding_id[YAI_SOURCE_BINDING_ID_MAX];
  int64_t last_observed_epoch;
  int64_t last_seen_epoch;
  int observation_count;
} yai_source_asset_seen_t;

typedef struct yai_emit_integrity_result {
  char classification[48];
  char handling_action[48];
  char ordering_status[32];
  char replay_status[32];
  char overlap_status[32];
  char asset_key_ref[256];
  int replay_detected;
  int overlap_detected;
  int conflict_detected;
  int ordering_late;
  int review_required;
} yai_emit_integrity_result_t;

static yai_source_emit_seen_t g_emit_seen[YAI_SOURCE_EMIT_SEEN_MAX];
static yai_source_asset_seen_t g_asset_seen[YAI_SOURCE_EMIT_ASSET_MAX];
static unsigned long g_ingest_outcome_seq = 0;

static void emit_integrity_default(yai_emit_integrity_result_t *out)
{
  if (!out)
  {
    return;
  }
  memset(out, 0, sizeof(*out));
  (void)snprintf(out->classification, sizeof(out->classification), "%s", "clean");
  (void)snprintf(out->handling_action, sizeof(out->handling_action), "%s", "accept");
  (void)snprintf(out->ordering_status, sizeof(out->ordering_status), "%s", "in_order");
  (void)snprintf(out->replay_status, sizeof(out->replay_status), "%s", "first_ingest");
  (void)snprintf(out->overlap_status, sizeof(out->overlap_status), "%s", "distinct");
}

static int find_emit_seen_slot(const char *workspace_id,
                               const char *idempotency_key,
                               int *first_free)
{
  size_t i = 0;
  int free_idx = -1;
  for (i = 0; i < YAI_SOURCE_EMIT_SEEN_MAX; ++i)
  {
    if (!g_emit_seen[i].used)
    {
      if (free_idx < 0)
      {
        free_idx = (int)i;
      }
      continue;
    }
    if (strcmp(g_emit_seen[i].workspace_id, workspace_id) == 0 &&
        strcmp(g_emit_seen[i].idempotency_key, idempotency_key) == 0)
    {
      if (first_free)
      {
        *first_free = free_idx;
      }
      return (int)i;
    }
  }
  if (first_free)
  {
    *first_free = free_idx;
  }
  return -1;
}

static int find_asset_seen_slot(const char *workspace_id,
                                const char *asset_key,
                                int *first_free)
{
  size_t i = 0;
  int free_idx = -1;
  for (i = 0; i < YAI_SOURCE_EMIT_ASSET_MAX; ++i)
  {
    if (!g_asset_seen[i].used)
    {
      if (free_idx < 0)
      {
        free_idx = (int)i;
      }
      continue;
    }
    if (strcmp(g_asset_seen[i].workspace_id, workspace_id) == 0 &&
        strcmp(g_asset_seen[i].asset_key, asset_key) == 0)
    {
      if (first_free)
      {
        *first_free = free_idx;
      }
      return (int)i;
    }
  }
  if (first_free)
  {
    *first_free = free_idx;
  }
  return -1;
}

static int build_asset_key_from_payload(cJSON *payload,
                                        char *out,
                                        size_t out_cap)
{
  cJSON *assets = NULL;
  cJSON *first = NULL;
  const char *fingerprint = NULL;
  const char *locator = NULL;
  const char *asset_id = NULL;
  if (!out || out_cap == 0)
  {
    return -1;
  }
  out[0] = '\0';
  if (!payload)
  {
    return 0;
  }
  assets = cJSON_GetObjectItem(payload, "source_assets");
  if (assets && cJSON_IsArray(assets) && cJSON_GetArraySize(assets) > 0)
  {
    first = cJSON_GetArrayItem(assets, 0);
  }
  if (!first)
  {
    first = cJSON_GetObjectItem(payload, "source_asset");
  }
  if (!first || !cJSON_IsObject(first))
  {
    return 0;
  }
  fingerprint = json_string(first, "provenance_fingerprint");
  locator = json_string(first, "locator");
  asset_id = json_string(first, "source_asset_id");
  if (fingerprint && fingerprint[0])
  {
    (void)snprintf(out, out_cap, "fingerprint:%s", fingerprint);
  }
  else if (locator && locator[0])
  {
    (void)snprintf(out, out_cap, "locator:%s", locator);
  }
  else if (asset_id && asset_id[0])
  {
    (void)snprintf(out, out_cap, "asset_id:%s", asset_id);
  }
  return 0;
}

static int64_t payload_latest_observed_epoch(cJSON *payload)
{
  cJSON *events = NULL;
  cJSON *single = NULL;
  int64_t latest = -1;
  int i = 0;
  if (!payload)
  {
    return -1;
  }
  events = cJSON_GetObjectItem(payload, "source_acquisition_events");
  if (events && cJSON_IsArray(events))
  {
    int n = cJSON_GetArraySize(events);
    for (i = 0; i < n; ++i)
    {
      cJSON *ev = cJSON_GetArrayItem(events, i);
      int64_t ts = json_i64(ev, "observed_at_epoch", -1);
      if (ts > latest) latest = ts;
    }
  }
  single = cJSON_GetObjectItem(payload, "source_acquisition_event");
  if (single && cJSON_IsObject(single))
  {
    int64_t ts = json_i64(single, "observed_at_epoch", -1);
    if (ts > latest) latest = ts;
  }
  return latest;
}

static int classify_emit_integrity(const char *workspace_id,
                                   const char *source_node_id,
                                   const char *source_binding_id,
                                   const char *idempotency_key,
                                   cJSON *payload,
                                   int accepted_total,
                                   yai_emit_integrity_result_t *out)
{
  int emit_idx = -1;
  int emit_free = -1;
  int asset_idx = -1;
  int asset_free = -1;
  int64_t now = (int64_t)time(NULL);
  int64_t observed_latest = -1;
  char asset_key[256];
  emit_integrity_default(out);
  asset_key[0] = '\0';

  if (!workspace_id || !source_node_id || !source_binding_id || !idempotency_key || !out)
  {
    return -1;
  }

  (void)build_asset_key_from_payload(payload, asset_key, sizeof(asset_key));
  if (asset_key[0])
  {
    (void)snprintf(out->asset_key_ref, sizeof(out->asset_key_ref), "%s", asset_key);
  }
  observed_latest = payload_latest_observed_epoch(payload);

  emit_idx = find_emit_seen_slot(workspace_id, idempotency_key, &emit_free);
  if (emit_idx >= 0)
  {
    out->replay_detected = 1;
    g_emit_seen[emit_idx].replay_count += 1;
    g_emit_seen[emit_idx].last_seen_epoch = now;
    if (strcmp(g_emit_seen[emit_idx].source_node_id, source_node_id) == 0 &&
        strcmp(g_emit_seen[emit_idx].source_binding_id, source_binding_id) == 0)
    {
      (void)snprintf(out->classification, sizeof(out->classification), "%s", "duplicate_replay");
      (void)snprintf(out->replay_status, sizeof(out->replay_status), "%s", "replay_same_peer");
      (void)snprintf(out->handling_action, sizeof(out->handling_action), "%s", "accept_with_flag");
    }
    else
    {
      (void)snprintf(out->classification, sizeof(out->classification), "%s", "overlap_informational");
      (void)snprintf(out->replay_status, sizeof(out->replay_status), "%s", "replay_cross_peer");
      (void)snprintf(out->overlap_status, sizeof(out->overlap_status), "%s", "informational");
      (void)snprintf(out->handling_action, sizeof(out->handling_action), "%s", "accept_with_flag");
      out->overlap_detected = 1;
    }
  }
  else if (emit_free >= 0)
  {
    memset(&g_emit_seen[emit_free], 0, sizeof(g_emit_seen[emit_free]));
    g_emit_seen[emit_free].used = 1;
    snprintf(g_emit_seen[emit_free].workspace_id, sizeof(g_emit_seen[emit_free].workspace_id), "%s", workspace_id);
    snprintf(g_emit_seen[emit_free].source_node_id, sizeof(g_emit_seen[emit_free].source_node_id), "%s", source_node_id);
    snprintf(g_emit_seen[emit_free].source_binding_id, sizeof(g_emit_seen[emit_free].source_binding_id), "%s", source_binding_id);
    snprintf(g_emit_seen[emit_free].idempotency_key, sizeof(g_emit_seen[emit_free].idempotency_key), "%s", idempotency_key);
    g_emit_seen[emit_free].first_seen_epoch = now;
    g_emit_seen[emit_free].last_seen_epoch = now;
    g_emit_seen[emit_free].replay_count = 0;
  }

  if (asset_key[0])
  {
    asset_idx = find_asset_seen_slot(workspace_id, asset_key, &asset_free);
    if (asset_idx >= 0)
    {
      if (strcmp(g_asset_seen[asset_idx].source_node_id, source_node_id) != 0)
      {
        out->overlap_detected = 1;
        if (out->replay_detected)
        {
          (void)snprintf(out->classification, sizeof(out->classification), "%s", "conflict_requires_review");
          (void)snprintf(out->handling_action, sizeof(out->handling_action), "%s", "review_stub");
          (void)snprintf(out->overlap_status, sizeof(out->overlap_status), "%s", "ambiguous");
          out->review_required = 1;
          out->conflict_detected = 1;
        }
        else
        {
          (void)snprintf(out->classification, sizeof(out->classification), "%s", "overlap_ambiguous");
          (void)snprintf(out->handling_action, sizeof(out->handling_action), "%s", "accept_with_flag");
          (void)snprintf(out->overlap_status, sizeof(out->overlap_status), "%s", "ambiguous");
        }
      }
      if (observed_latest > 0 && g_asset_seen[asset_idx].last_observed_epoch > 0 &&
          observed_latest + 30 < g_asset_seen[asset_idx].last_observed_epoch)
      {
        out->ordering_late = 1;
        (void)snprintf(out->ordering_status, sizeof(out->ordering_status), "%s", "late");
        if (strcmp(out->classification, "clean") == 0)
        {
          (void)snprintf(out->classification, sizeof(out->classification), "%s", "ordering_late");
          (void)snprintf(out->handling_action, sizeof(out->handling_action), "%s", "accept_with_flag");
        }
      }
      if (observed_latest > g_asset_seen[asset_idx].last_observed_epoch)
      {
        g_asset_seen[asset_idx].last_observed_epoch = observed_latest;
      }
      g_asset_seen[asset_idx].last_seen_epoch = now;
      g_asset_seen[asset_idx].observation_count += 1;
    }
    else if (asset_free >= 0)
    {
      memset(&g_asset_seen[asset_free], 0, sizeof(g_asset_seen[asset_free]));
      g_asset_seen[asset_free].used = 1;
      snprintf(g_asset_seen[asset_free].workspace_id, sizeof(g_asset_seen[asset_free].workspace_id), "%s", workspace_id);
      snprintf(g_asset_seen[asset_free].asset_key, sizeof(g_asset_seen[asset_free].asset_key), "%s", asset_key);
      snprintf(g_asset_seen[asset_free].source_node_id, sizeof(g_asset_seen[asset_free].source_node_id), "%s", source_node_id);
      snprintf(g_asset_seen[asset_free].source_binding_id, sizeof(g_asset_seen[asset_free].source_binding_id), "%s", source_binding_id);
      g_asset_seen[asset_free].last_observed_epoch = observed_latest;
      g_asset_seen[asset_free].last_seen_epoch = now;
      g_asset_seen[asset_free].observation_count = 1;
    }
  }

  if (accepted_total >= 25)
  {
    (void)snprintf(out->replay_status, sizeof(out->replay_status), "%s", "backlog_flush");
    if (strcmp(out->classification, "clean") == 0)
    {
      (void)snprintf(out->classification, sizeof(out->classification), "%s", "overlap_informational");
      (void)snprintf(out->handling_action, sizeof(out->handling_action), "%s", "accept_with_flag");
    }
  }
  return 0;
}

static int make_ingest_outcome_id(char *out, size_t out_cap)
{
  if (!out || out_cap == 0)
  {
    return -1;
  }
  g_ingest_outcome_seq += 1;
  if (snprintf(out, out_cap, "sio-%lld-%lu", (long long)time(NULL), g_ingest_outcome_seq) >= (int)out_cap)
  {
    return -1;
  }
  return 0;
}

static int make_distribution_target_ref(char *out,
                                        size_t out_cap,
                                        const char *workspace_id,
                                        const char *source_node_id,
                                        const char *daemon_instance_id,
                                        const char *source_binding_id)
{
  if (!out || out_cap == 0 || !workspace_id || !source_node_id || !daemon_instance_id)
  {
    return -1;
  }
  if (source_binding_id && source_binding_id[0])
  {
    if (snprintf(out,
                 out_cap,
                 "workspace:%s/node:%s/daemon:%s/binding:%s",
                 workspace_id,
                 source_node_id,
                 daemon_instance_id,
                 source_binding_id) >= (int)out_cap)
    {
      return -1;
    }
    return 0;
  }
  if (snprintf(out,
               out_cap,
               "workspace:%s/node:%s/daemon:%s",
               workspace_id,
               source_node_id,
               daemon_instance_id) >= (int)out_cap)
  {
    return -1;
  }
  return 0;
}

static void mediation_json(const yai_source_plane_mediation_state_t *med,
                           char *out,
                           size_t out_cap)
{
  if (!out || out_cap == 0)
  {
    return;
  }
  if (!med)
  {
    (void)snprintf(out, out_cap, "%s", "{\"layer\":\"exec\",\"stage\":\"unknown\"}");
    return;
  }
  (void)snprintf(out,
                 out_cap,
                 "{"
                 "\"layer\":\"exec\","
                 "\"stage\":\"%s\","
                 "\"topology\":\"%s\","
                 "\"route\":\"%s\","
                 "\"owner_canonical\":%s,"
                 "\"transport_ready\":%s,"
                 "\"network_gate_ready\":%s,"
                 "\"resource_gate_ready\":%s,"
                 "\"storage_gate_ready\":%s"
                 "}",
                 med->stage[0] ? med->stage : "exec.runtime.source_plane_mediation.v1",
                 yai_source_plane_topology_id(),
                 med->route[0] ? med->route : "owner_ingest_v1",
                 med->owner_canonical ? "true" : "false",
                 med->transport_ready ? "true" : "false",
                 med->network_gate_ready ? "true" : "false",
                 med->resource_gate_ready ? "true" : "false",
                 med->storage_gate_ready ? "true" : "false");
}

static int handle_enroll(const char *workspace_id,
                         cJSON *payload,
                         const yai_source_plane_mediation_state_t *med,
                         char *out_json,
                         size_t out_cap,
                         char *reason,
                         size_t reason_cap)
{
  const char *source_label = json_string(payload, "source_label");
  const char *owner_ref = json_string(payload, "owner_ref");
  const char *node_id_in = json_string(payload, "source_node_id");
  const char *daemon_id_in = json_string(payload, "daemon_instance_id");
  const char *decision = enrollment_decision_from_payload(payload);
  char node_id[YAI_SOURCE_NODE_ID_MAX];
  char daemon_id[YAI_SOURCE_DAEMON_INSTANCE_ID_MAX];
  char owner_link_id[YAI_SOURCE_OWNER_LINK_ID_MAX];
  char enrollment_grant_id[YAI_SOURCE_ENROLLMENT_GRANT_ID_MAX];
  char source_policy_snapshot_id[YAI_SOURCE_POLICY_SNAPSHOT_ID_MAX];
  char source_capability_envelope_id[YAI_SOURCE_CAPABILITY_ENVELOPE_ID_MAX];
  char distribution_target_ref[YAI_SOURCE_REF_MAX];
  const char *snapshot_version = "ws-policy-snapshot-v1";
  const char *observation_scope = "workspace/default";
  const char *mediation_scope = "none";
  const char *enforcement_scope = "none";
  int64_t now_epoch = (int64_t)time(NULL);
  int64_t valid_from_epoch = now_epoch;
  int64_t refresh_after_epoch = now_epoch + 300;
  int64_t expires_at_epoch = now_epoch + 1800;
  const char *validity_state = strcmp(decision, "accepted") == 0 ? "valid" : "pending";
  char trust_artifact_id[128];
  char trust_artifact_token[128];
  char med_json[384];
  cJSON *node = NULL;
  cJSON *inst = NULL;
  cJSON *link = NULL;
  cJSON *grant = NULL;
  cJSON *snapshot = NULL;
  cJSON *capability = NULL;
  char err[160];

  if (!workspace_id || !yai_ws_id_is_valid(workspace_id) || !payload || !out_json || out_cap == 0)
  {
    (void)set_reason(reason, reason_cap, "source_enroll_bad_args");
    return -2;
  }

  if (!source_label || !source_label[0])
  {
    source_label = "source-node";
  }
  if (!owner_ref)
  {
    owner_ref = "owner-ref-unset";
  }

  if (node_id_in && node_id_in[0])
  {
    (void)snprintf(node_id, sizeof(node_id), "%s", node_id_in);
  }
  else if (yai_source_id_node(node_id, sizeof(node_id), source_label) != 0)
  {
    (void)set_reason(reason, reason_cap, "source_node_id_generation_failed");
    return -4;
  }

  if (daemon_id_in && daemon_id_in[0])
  {
    (void)snprintf(daemon_id, sizeof(daemon_id), "%s", daemon_id_in);
  }
  else if (yai_source_id_daemon_instance(daemon_id, sizeof(daemon_id), node_id) != 0)
  {
    (void)set_reason(reason, reason_cap, "source_daemon_id_generation_failed");
    return -4;
  }

  if (yai_source_id_owner_link(owner_link_id, sizeof(owner_link_id), node_id, owner_ref) != 0)
  {
    (void)set_reason(reason, reason_cap, "source_owner_link_id_generation_failed");
    return -4;
  }
  if (yai_source_id_enrollment_grant(enrollment_grant_id, sizeof(enrollment_grant_id), node_id, daemon_id) != 0)
  {
    (void)set_reason(reason, reason_cap, "source_enrollment_grant_id_generation_failed");
    return -4;
  }
  if (yai_source_id_policy_snapshot(source_policy_snapshot_id, sizeof(source_policy_snapshot_id), node_id, daemon_id, workspace_id) != 0)
  {
    (void)set_reason(reason, reason_cap, "source_policy_snapshot_id_generation_failed");
    return -4;
  }
  if (yai_source_id_capability_envelope(source_capability_envelope_id, sizeof(source_capability_envelope_id), "enroll-baseline", node_id, workspace_id) != 0)
  {
    (void)set_reason(reason, reason_cap, "source_capability_envelope_id_generation_failed");
    return -4;
  }
  if (make_distribution_target_ref(distribution_target_ref,
                                   sizeof(distribution_target_ref),
                                   workspace_id,
                                   node_id,
                                   daemon_id,
                                   NULL) != 0)
  {
    (void)set_reason(reason, reason_cap, "source_distribution_target_encode_failed");
    return -4;
  }
  if (snprintf(trust_artifact_id, sizeof(trust_artifact_id), "owner-grant:%s", enrollment_grant_id) >= (int)sizeof(trust_artifact_id))
  {
    (void)set_reason(reason, reason_cap, "source_enrollment_artifact_id_encode_failed");
    return -4;
  }
  if (make_owner_trust_token(node_id, owner_link_id, trust_artifact_token, sizeof(trust_artifact_token)) != 0)
  {
    (void)set_reason(reason, reason_cap, "source_enrollment_trust_token_encode_failed");
    return -4;
  }

  node = cJSON_CreateObject();
  inst = cJSON_CreateObject();
  link = cJSON_CreateObject();
  grant = cJSON_CreateObject();
  snapshot = cJSON_CreateObject();
  capability = cJSON_CreateObject();
  if (!node || !inst || !link || !grant || !snapshot || !capability)
  {
    cJSON_Delete(node);
    cJSON_Delete(inst);
    cJSON_Delete(link);
    cJSON_Delete(grant);
    cJSON_Delete(snapshot);
    cJSON_Delete(capability);
    (void)set_reason(reason, reason_cap, "source_enroll_allocation_failed");
    return -4;
  }

  cJSON_AddStringToObject(node, "type", "yai.source_node.v1");
  cJSON_AddStringToObject(node, "source_node_id", node_id);
  cJSON_AddStringToObject(node, "source_label", source_label);
  cJSON_AddStringToObject(node, "trust_level", "baseline");
  cJSON_AddStringToObject(node, "owner_ref", owner_ref);
  cJSON_AddStringToObject(node, "state", strcmp(decision, "accepted") == 0 ? "enrolled" : "pending_enrollment");

  cJSON_AddStringToObject(inst, "type", "yai.source_daemon_instance.v1");
  cJSON_AddStringToObject(inst, "daemon_instance_id", daemon_id);
  cJSON_AddStringToObject(inst, "source_node_id", node_id);
  cJSON_AddStringToObject(inst, "session_marker", "owner_ingest_v1");
  cJSON_AddNumberToObject(inst, "started_at_epoch", (double)time(NULL));
  cJSON_AddStringToObject(inst, "health", "registered");
  cJSON_AddStringToObject(inst, "build_marker", "yd4-v1");

  cJSON_AddStringToObject(link, "type", "yai.source_owner_link.v1");
  cJSON_AddStringToObject(link, "source_owner_link_id", owner_link_id);
  cJSON_AddStringToObject(link, "source_node_id", node_id);
  cJSON_AddStringToObject(link, "owner_ref", owner_ref);
  cJSON_AddStringToObject(link, "registration_status", strcmp(decision, "accepted") == 0 ? "registered" : decision);
  cJSON_AddStringToObject(link, "enrollment_decision", decision);
  cJSON_AddStringToObject(link, "trust_artifact_id", trust_artifact_id);
  cJSON_AddNumberToObject(link, "registered_at_epoch", (double)time(NULL));

  cJSON_AddStringToObject(grant, "type", "yai.source_enrollment_grant.v1");
  cJSON_AddStringToObject(grant, "source_enrollment_grant_id", enrollment_grant_id);
  cJSON_AddStringToObject(grant, "source_node_id", node_id);
  cJSON_AddStringToObject(grant, "daemon_instance_id", daemon_id);
  cJSON_AddStringToObject(grant, "owner_ref", owner_ref);
  cJSON_AddStringToObject(grant, "enrollment_decision", decision);
  cJSON_AddStringToObject(grant, "trust_artifact_id", trust_artifact_id);
  cJSON_AddStringToObject(grant, "trust_artifact_token", strcmp(decision, "accepted") == 0 ? trust_artifact_token : "pending");
  cJSON_AddStringToObject(grant, "trust_bootstrap_model", "owner_issued_v1");
  cJSON_AddStringToObject(grant, "validity_state", validity_state);
  cJSON_AddNumberToObject(grant, "valid_from_epoch", (double)valid_from_epoch);
  cJSON_AddNumberToObject(grant, "refresh_after_epoch", (double)refresh_after_epoch);
  cJSON_AddNumberToObject(grant, "expires_at_epoch", (double)expires_at_epoch);
  cJSON_AddBoolToObject(grant, "revoked", 0);
  cJSON_AddNumberToObject(grant, "issued_at_epoch", (double)now_epoch);

  cJSON_AddStringToObject(snapshot, "type", "yai.source_policy_snapshot.v1");
  cJSON_AddStringToObject(snapshot, "source_policy_snapshot_id", source_policy_snapshot_id);
  cJSON_AddStringToObject(snapshot, "source_node_id", node_id);
  cJSON_AddStringToObject(snapshot, "daemon_instance_id", daemon_id);
  cJSON_AddStringToObject(snapshot, "owner_workspace_id", workspace_id);
  cJSON_AddStringToObject(snapshot, "source_enrollment_grant_id", enrollment_grant_id);
  cJSON_AddStringToObject(snapshot, "snapshot_version", snapshot_version);
  cJSON_AddStringToObject(snapshot, "distribution_target_ref", distribution_target_ref);
  cJSON_AddStringToObject(snapshot, "validity_state", validity_state);
  cJSON_AddNumberToObject(snapshot, "valid_from_epoch", (double)valid_from_epoch);
  cJSON_AddNumberToObject(snapshot, "refresh_after_epoch", (double)refresh_after_epoch);
  cJSON_AddNumberToObject(snapshot, "expires_at_epoch", (double)expires_at_epoch);
  cJSON_AddBoolToObject(snapshot, "revoked", 0);
  cJSON_AddNumberToObject(snapshot, "issued_at_epoch", (double)now_epoch);

  cJSON_AddStringToObject(capability, "type", "yai.source_capability_envelope.v1");
  cJSON_AddStringToObject(capability, "source_capability_envelope_id", source_capability_envelope_id);
  cJSON_AddStringToObject(capability, "source_node_id", node_id);
  cJSON_AddStringToObject(capability, "daemon_instance_id", daemon_id);
  cJSON_AddStringToObject(capability, "source_binding_id", "enroll-baseline");
  cJSON_AddStringToObject(capability, "owner_workspace_id", workspace_id);
  cJSON_AddStringToObject(capability, "source_enrollment_grant_id", enrollment_grant_id);
  cJSON_AddStringToObject(capability, "observation_scope", observation_scope);
  cJSON_AddStringToObject(capability, "mediation_scope", mediation_scope);
  cJSON_AddStringToObject(capability, "enforcement_scope", enforcement_scope);
  cJSON_AddStringToObject(capability, "distribution_target_ref", distribution_target_ref);
  cJSON_AddStringToObject(capability, "validity_state", validity_state);
  cJSON_AddNumberToObject(capability, "valid_from_epoch", (double)valid_from_epoch);
  cJSON_AddNumberToObject(capability, "refresh_after_epoch", (double)refresh_after_epoch);
  cJSON_AddNumberToObject(capability, "expires_at_epoch", (double)expires_at_epoch);
  cJSON_AddBoolToObject(capability, "revoked", 0);
  cJSON_AddNumberToObject(capability, "issued_at_epoch", (double)now_epoch);

  if (append_source_record(workspace_id, YAI_SOURCE_RECORD_CLASS_NODE, node, err, sizeof(err)) != 0 ||
      append_source_record(workspace_id, YAI_SOURCE_RECORD_CLASS_DAEMON_INSTANCE, inst, err, sizeof(err)) != 0 ||
      append_source_record(workspace_id, YAI_SOURCE_RECORD_CLASS_OWNER_LINK, link, err, sizeof(err)) != 0 ||
      append_source_record(workspace_id, YAI_SOURCE_RECORD_CLASS_ENROLLMENT_GRANT, grant, err, sizeof(err)) != 0 ||
      append_source_record(workspace_id, YAI_SOURCE_RECORD_CLASS_POLICY_SNAPSHOT, snapshot, err, sizeof(err)) != 0 ||
      append_source_record(workspace_id, YAI_SOURCE_RECORD_CLASS_CAPABILITY_ENVELOPE, capability, err, sizeof(err)) != 0)
  {
    cJSON_Delete(node);
    cJSON_Delete(inst);
    cJSON_Delete(link);
    cJSON_Delete(grant);
    cJSON_Delete(snapshot);
    cJSON_Delete(capability);
    (void)set_reason(reason, reason_cap, err[0] ? err : "source_enroll_persistence_failed");
    return -4;
  }

  cJSON_Delete(node);
  cJSON_Delete(inst);
  cJSON_Delete(link);
  cJSON_Delete(grant);
  cJSON_Delete(snapshot);
  cJSON_Delete(capability);

  mediation_json(med, med_json, sizeof(med_json));

  if (snprintf(out_json,
               out_cap,
               "{\"type\":\"yai.source.enroll.reply.v1\",\"workspace_id\":\"%s\",\"source_node_id\":\"%s\",\"daemon_instance_id\":\"%s\",\"owner_link_id\":\"%s\",\"source_enrollment_grant_id\":\"%s\",\"source_policy_snapshot_id\":\"%s\",\"source_capability_envelope_id\":\"%s\",\"policy_snapshot_version\":\"%s\",\"distribution_target_ref\":\"%s\",\"delegated_observation_scope\":\"%s\",\"delegated_mediation_scope\":\"%s\",\"delegated_enforcement_scope\":\"%s\",\"delegated_validity_state\":\"%s\",\"delegated_refresh_state\":\"%s\",\"delegated_revoke_state\":\"%s\",\"grant_valid_from_epoch\":%lld,\"grant_refresh_after_epoch\":%lld,\"grant_expires_at_epoch\":%lld,\"policy_snapshot_issued_at_epoch\":%lld,\"policy_snapshot_refresh_after_epoch\":%lld,\"policy_snapshot_expires_at_epoch\":%lld,\"capability_issued_at_epoch\":%lld,\"capability_refresh_after_epoch\":%lld,\"capability_expires_at_epoch\":%lld,\"owner_trust_artifact_id\":\"%s\",\"owner_trust_artifact_token\":\"%s\",\"enrollment_decision\":\"%s\",\"ready_for_attach\":%s,\"registered\":%s,\"mediation\":%s}",
               workspace_id,
               node_id,
               daemon_id,
               owner_link_id,
               enrollment_grant_id,
               source_policy_snapshot_id,
               source_capability_envelope_id,
               snapshot_version,
               distribution_target_ref,
               observation_scope,
               mediation_scope,
               enforcement_scope,
               validity_state,
               strcmp(validity_state, "valid") == 0 ? "not_required" : "required",
               "active",
               (long long)valid_from_epoch,
               (long long)refresh_after_epoch,
               (long long)expires_at_epoch,
               (long long)valid_from_epoch,
               (long long)refresh_after_epoch,
               (long long)expires_at_epoch,
               (long long)valid_from_epoch,
               (long long)refresh_after_epoch,
               (long long)expires_at_epoch,
               trust_artifact_id,
               strcmp(decision, "accepted") == 0 ? trust_artifact_token : "pending",
               decision,
               strcmp(decision, "accepted") == 0 ? "true" : "false",
               strcmp(decision, "accepted") == 0 ? "true" : "false",
               med_json) <= 0)
  {
    (void)set_reason(reason, reason_cap, "source_enroll_response_encode_failed");
    return -4;
  }

  (void)set_reason(reason, reason_cap, strcmp(decision, "accepted") == 0 ? "source_enroll_accepted" : "source_enroll_pending_or_rejected");
  return 0;
}

static int handle_attach(const char *workspace_id,
                         cJSON *payload,
                         const yai_source_plane_mediation_state_t *med,
                         char *out_json,
                         size_t out_cap,
                         char *reason,
                         size_t reason_cap)
{
  const char *source_node_id = json_string(payload, "source_node_id");
  const char *binding_scope = json_string(payload, "binding_scope");
  const char *constraints_ref = json_string(payload, "constraints_ref");
  const char *binding_kind = json_string_or(payload, "binding_kind", "observational");
  const char *observation_scope = json_string_or(payload, "observation_scope", "workspace/default");
  const char *mediation_scope = json_string_or(payload, "mediation_scope", "none");
  const char *enforcement_scope = json_string_or(payload, "enforcement_scope", "none");
  const char *mediation_mode = json_string_or(payload, "mediation_mode", "none");
  const char *snapshot_version = json_string_or(payload, "policy_snapshot_version", "ws-policy-snapshot-v1");
  const char *source_enrollment_grant_id = json_string_or(payload, "source_enrollment_grant_id", "grant-unset");
  int64_t now_epoch = (int64_t)time(NULL);
  int64_t valid_from_epoch = json_i64(payload, "grant_valid_from_epoch", now_epoch);
  int64_t refresh_after_epoch = json_i64(payload, "grant_refresh_after_epoch", now_epoch + 300);
  int64_t expires_at_epoch = json_i64(payload, "grant_expires_at_epoch", now_epoch + 1800);
  int revoked = json_bool(payload, "delegated_revoked", 0);
  const char *delegated_validity_state = json_string_or(payload, "delegated_validity_state", "valid");
  const char *delegated_refresh_state = json_string_or(payload, "delegated_refresh_state", "not_required");
  const char *delegated_revoke_state = revoked ? "revoked" : json_string_or(payload, "delegated_revoke_state", "active");
  const char *owner_workspace_id = json_string(payload, "owner_workspace_id");
  char source_policy_snapshot_id[YAI_SOURCE_POLICY_SNAPSHOT_ID_MAX];
  char source_capability_envelope_id[YAI_SOURCE_CAPABILITY_ENVELOPE_ID_MAX];
  char distribution_target_ref[YAI_SOURCE_REF_MAX];
  char source_binding_id[YAI_SOURCE_BINDING_ID_MAX];
  char workspace_peer_membership_id[YAI_SOURCE_WORKSPACE_PEER_MEMBERSHIP_ID_MAX];
  const char *peer_role = json_string_or(payload, "peer_role", "general");
  const char *peer_scope = json_string_or(payload, "peer_scope", "workspace/default");
  const char *peer_state = json_string_or(payload, "peer_state", "active");
  const char *daemon_instance_id = json_string_or(payload, "daemon_instance_id", "daemon-instance-unset");
  const char *coverage_ref = json_string_or(payload, "coverage_ref", "coverage://workspace/default");
  const char *overlap_state = json_string_or(payload, "overlap_state", "distinct");
  char med_json[384];
  cJSON *binding = NULL;
  cJSON *membership = NULL;
  cJSON *snapshot = NULL;
  cJSON *capability = NULL;
  char err[160];
  const char *target_ws = owner_workspace_id && owner_workspace_id[0] ? owner_workspace_id : workspace_id;
  yai_owner_peer_registry_entry_t reg;

  if (!workspace_id || !yai_ws_id_is_valid(workspace_id) || !payload || !source_node_id || !source_node_id[0])
  {
    (void)set_reason(reason, reason_cap, "source_attach_bad_args");
    return -2;
  }
  if (!owner_trust_token_valid_for_node(payload, source_node_id))
  {
    (void)set_reason(reason, reason_cap, "source_attach_trust_bootstrap_required");
    return -3;
  }
  if (!yai_ws_id_is_valid(target_ws))
  {
    (void)set_reason(reason, reason_cap, "source_attach_invalid_workspace");
    return -2;
  }
  if (!binding_scope)
  {
    binding_scope = "workspace";
  }
  if (!constraints_ref)
  {
    constraints_ref = "none";
  }

  if (yai_source_id_binding(source_binding_id, sizeof(source_binding_id), source_node_id, target_ws) != 0)
  {
    (void)set_reason(reason, reason_cap, "source_attach_binding_id_generation_failed");
    return -4;
  }
  if (yai_source_id_workspace_peer_membership(workspace_peer_membership_id,
                                              sizeof(workspace_peer_membership_id),
                                              target_ws,
                                              source_node_id,
                                              source_binding_id) != 0)
  {
    (void)set_reason(reason, reason_cap, "workspace_peer_membership_id_generation_failed");
    return -4;
  }
  if (yai_source_id_policy_snapshot(source_policy_snapshot_id,
                                    sizeof(source_policy_snapshot_id),
                                    source_node_id,
                                    daemon_instance_id,
                                    target_ws) != 0)
  {
    (void)set_reason(reason, reason_cap, "source_policy_snapshot_id_generation_failed");
    return -4;
  }
  if (yai_source_id_capability_envelope(source_capability_envelope_id,
                                        sizeof(source_capability_envelope_id),
                                        source_binding_id,
                                        source_node_id,
                                        target_ws) != 0)
  {
    (void)set_reason(reason, reason_cap, "source_capability_envelope_id_generation_failed");
    return -4;
  }
  if (make_distribution_target_ref(distribution_target_ref,
                                   sizeof(distribution_target_ref),
                                   target_ws,
                                   source_node_id,
                                   daemon_instance_id,
                                   source_binding_id) != 0)
  {
    (void)set_reason(reason, reason_cap, "source_distribution_target_encode_failed");
    return -4;
  }

  binding = cJSON_CreateObject();
  if (!binding)
  {
    (void)set_reason(reason, reason_cap, "source_attach_allocation_failed");
    return -4;
  }
  cJSON_AddStringToObject(binding, "type", "yai.source_binding.v1");
  cJSON_AddStringToObject(binding, "source_binding_id", source_binding_id);
  cJSON_AddStringToObject(binding, "source_node_id", source_node_id);
  cJSON_AddStringToObject(binding, "owner_workspace_id", target_ws);
  cJSON_AddStringToObject(binding, "binding_scope", binding_scope);
  cJSON_AddStringToObject(binding, "binding_kind", binding_kind);
  cJSON_AddStringToObject(binding, "observation_scope", observation_scope);
  cJSON_AddStringToObject(binding, "mediation_scope", mediation_scope);
  cJSON_AddStringToObject(binding, "enforcement_scope", enforcement_scope);
  cJSON_AddStringToObject(binding, "mediation_mode", mediation_mode);
  cJSON_AddNumberToObject(binding, "action_point_count", (double)json_i64(payload, "action_point_count", 0));
  cJSON_AddStringToObject(binding, "action_points_ref", json_string_or(payload, "action_points_ref", "unset"));
  cJSON_AddStringToObject(binding, "attachment_status", "attached");
  cJSON_AddStringToObject(binding, "constraints_ref", constraints_ref);

  membership = cJSON_CreateObject();
  snapshot = cJSON_CreateObject();
  capability = cJSON_CreateObject();
  if (!membership || !snapshot || !capability)
  {
    cJSON_Delete(binding);
    cJSON_Delete(membership);
    cJSON_Delete(snapshot);
    cJSON_Delete(capability);
    (void)set_reason(reason, reason_cap, "workspace_peer_membership_allocation_failed");
    return -4;
  }
  cJSON_AddStringToObject(membership, "type", "yai.workspace_peer_membership.v1");
  cJSON_AddStringToObject(membership, "workspace_peer_membership_id", workspace_peer_membership_id);
  cJSON_AddStringToObject(membership, "owner_workspace_id", target_ws);
  cJSON_AddStringToObject(membership, "source_node_id", source_node_id);
  cJSON_AddStringToObject(membership, "source_binding_id", source_binding_id);
  cJSON_AddStringToObject(membership, "daemon_instance_id", daemon_instance_id);
  cJSON_AddStringToObject(membership, "peer_role", peer_role);
  cJSON_AddStringToObject(membership, "peer_scope", peer_scope);
  cJSON_AddStringToObject(membership, "peer_state", peer_state);
  cJSON_AddNumberToObject(membership, "backlog_queued", 0);
  cJSON_AddNumberToObject(membership, "backlog_retry_due", 0);
  cJSON_AddNumberToObject(membership, "backlog_failed", 0);
  cJSON_AddStringToObject(membership, "coverage_ref", coverage_ref);
  cJSON_AddStringToObject(membership, "overlap_state", overlap_state);
  cJSON_AddNumberToObject(membership, "updated_at_epoch", (double)time(NULL));

  cJSON_AddStringToObject(snapshot, "type", "yai.source_policy_snapshot.v1");
  cJSON_AddStringToObject(snapshot, "source_policy_snapshot_id", source_policy_snapshot_id);
  cJSON_AddStringToObject(snapshot, "source_node_id", source_node_id);
  cJSON_AddStringToObject(snapshot, "daemon_instance_id", daemon_instance_id);
  cJSON_AddStringToObject(snapshot, "owner_workspace_id", target_ws);
  cJSON_AddStringToObject(snapshot, "source_enrollment_grant_id", source_enrollment_grant_id);
  cJSON_AddStringToObject(snapshot, "snapshot_version", snapshot_version);
  cJSON_AddStringToObject(snapshot, "distribution_target_ref", distribution_target_ref);
  cJSON_AddStringToObject(snapshot, "validity_state", delegated_validity_state);
  cJSON_AddStringToObject(snapshot, "refresh_state", delegated_refresh_state);
  cJSON_AddStringToObject(snapshot, "revoke_state", delegated_revoke_state);
  cJSON_AddNumberToObject(snapshot, "valid_from_epoch", (double)valid_from_epoch);
  cJSON_AddNumberToObject(snapshot, "refresh_after_epoch", (double)refresh_after_epoch);
  cJSON_AddNumberToObject(snapshot, "expires_at_epoch", (double)expires_at_epoch);
  cJSON_AddBoolToObject(snapshot, "revoked", revoked ? 1 : 0);
  cJSON_AddNumberToObject(snapshot, "issued_at_epoch", (double)now_epoch);

  cJSON_AddStringToObject(capability, "type", "yai.source_capability_envelope.v1");
  cJSON_AddStringToObject(capability, "source_capability_envelope_id", source_capability_envelope_id);
  cJSON_AddStringToObject(capability, "source_node_id", source_node_id);
  cJSON_AddStringToObject(capability, "daemon_instance_id", daemon_instance_id);
  cJSON_AddStringToObject(capability, "source_binding_id", source_binding_id);
  cJSON_AddStringToObject(capability, "owner_workspace_id", target_ws);
  cJSON_AddStringToObject(capability, "source_enrollment_grant_id", source_enrollment_grant_id);
  cJSON_AddStringToObject(capability, "observation_scope", observation_scope);
  cJSON_AddStringToObject(capability, "mediation_scope", mediation_scope);
  cJSON_AddStringToObject(capability, "enforcement_scope", enforcement_scope);
  cJSON_AddStringToObject(capability, "distribution_target_ref", distribution_target_ref);
  cJSON_AddStringToObject(capability, "validity_state", delegated_validity_state);
  cJSON_AddStringToObject(capability, "refresh_state", delegated_refresh_state);
  cJSON_AddStringToObject(capability, "revoke_state", delegated_revoke_state);
  cJSON_AddNumberToObject(capability, "valid_from_epoch", (double)valid_from_epoch);
  cJSON_AddNumberToObject(capability, "refresh_after_epoch", (double)refresh_after_epoch);
  cJSON_AddNumberToObject(capability, "expires_at_epoch", (double)expires_at_epoch);
  cJSON_AddBoolToObject(capability, "revoked", revoked ? 1 : 0);
  cJSON_AddNumberToObject(capability, "issued_at_epoch", (double)now_epoch);

  if (append_source_record(workspace_id, YAI_SOURCE_RECORD_CLASS_BINDING, binding, err, sizeof(err)) != 0)
  {
    cJSON_Delete(binding);
    cJSON_Delete(membership);
    cJSON_Delete(snapshot);
    cJSON_Delete(capability);
    (void)set_reason(reason, reason_cap, err[0] ? err : "source_attach_persistence_failed");
    return -4;
  }
  if (append_source_record(workspace_id,
                           YAI_SOURCE_RECORD_CLASS_WORKSPACE_PEER_MEMBERSHIP,
                           membership,
                           err,
                           sizeof(err)) != 0)
  {
    cJSON_Delete(binding);
    cJSON_Delete(membership);
    cJSON_Delete(snapshot);
    cJSON_Delete(capability);
    (void)set_reason(reason, reason_cap, err[0] ? err : "workspace_peer_membership_persistence_failed");
    return -4;
  }
  if (append_source_record(workspace_id,
                           YAI_SOURCE_RECORD_CLASS_POLICY_SNAPSHOT,
                           snapshot,
                           err,
                           sizeof(err)) != 0)
  {
    cJSON_Delete(binding);
    cJSON_Delete(membership);
    cJSON_Delete(snapshot);
    cJSON_Delete(capability);
    (void)set_reason(reason, reason_cap, err[0] ? err : "source_attach_policy_snapshot_persistence_failed");
    return -4;
  }
  if (append_source_record(workspace_id,
                           YAI_SOURCE_RECORD_CLASS_CAPABILITY_ENVELOPE,
                           capability,
                           err,
                           sizeof(err)) != 0)
  {
    cJSON_Delete(binding);
    cJSON_Delete(membership);
    cJSON_Delete(snapshot);
    cJSON_Delete(capability);
    (void)set_reason(reason, reason_cap, err[0] ? err : "source_attach_capability_envelope_persistence_failed");
    return -4;
  }
  {
    int action_point_count = (int)json_i64(payload, "action_point_count", 0);
    cJSON *aps = cJSON_GetObjectItem(payload, "action_points");
    if (aps && cJSON_IsArray(aps))
    {
      int ap_n = cJSON_GetArraySize(aps);
      int ap_i = 0;
      for (ap_i = 0; ap_i < ap_n; ++ap_i)
      {
        cJSON *ap = cJSON_GetArrayItem(aps, ap_i);
        cJSON *ap_record = NULL;
        const char *action_ref = NULL;
        const char *action_kind = NULL;
        const char *ap_mediation_scope = NULL;
        const char *ap_enforcement_scope = NULL;
        const char *controllability_state = NULL;
        char source_action_point_id[YAI_SOURCE_ACTION_POINT_ID_MAX];
        if (!ap || !cJSON_IsObject(ap))
        {
          continue;
        }
        action_ref = json_string(ap, "action_ref");
        action_kind = json_string_or(ap, "action_kind", "unknown");
        ap_mediation_scope = json_string_or(ap, "mediation_scope", mediation_scope);
        ap_enforcement_scope = json_string_or(ap, "enforcement_scope", enforcement_scope);
        controllability_state = json_string_or(ap, "controllability_state", "delegated_candidate");
        if (!action_ref || !action_ref[0])
        {
          continue;
        }
        if (yai_source_id_action_point(source_action_point_id,
                                       sizeof(source_action_point_id),
                                       source_binding_id,
                                       action_ref) != 0)
        {
          continue;
        }
        ap_record = cJSON_CreateObject();
        if (!ap_record)
        {
          continue;
        }
        cJSON_AddStringToObject(ap_record, "type", "yai.source_action_point.v1");
        cJSON_AddStringToObject(ap_record, "source_action_point_id", source_action_point_id);
        cJSON_AddStringToObject(ap_record, "source_node_id", source_node_id);
        cJSON_AddStringToObject(ap_record, "source_binding_id", source_binding_id);
        cJSON_AddStringToObject(ap_record, "action_kind", action_kind);
        cJSON_AddStringToObject(ap_record, "action_ref", action_ref);
        cJSON_AddStringToObject(ap_record, "mediation_scope", ap_mediation_scope);
        cJSON_AddStringToObject(ap_record, "enforcement_scope", ap_enforcement_scope);
        cJSON_AddStringToObject(ap_record, "controllability_state", controllability_state);
        cJSON_AddNumberToObject(ap_record, "updated_at_epoch", (double)time(NULL));
        if (append_source_record(workspace_id,
                                 YAI_SOURCE_RECORD_CLASS_ACTION_POINT,
                                 ap_record,
                                 err,
                                 sizeof(err)) != 0)
        {
          cJSON_Delete(ap_record);
          cJSON_Delete(binding);
          cJSON_Delete(membership);
          cJSON_Delete(snapshot);
          cJSON_Delete(capability);
          (void)set_reason(reason, reason_cap, err[0] ? err : "source_attach_action_point_persistence_failed");
          return -4;
        }
        cJSON_Delete(ap_record);
      }
    }
    else if (action_point_count > 0)
    {
      int ap_i = 0;
      const char *action_points_ref = json_string_or(payload, "action_points_ref", "action-points://unset");
      int capped = action_point_count > 16 ? 16 : action_point_count;
      for (ap_i = 0; ap_i < capped; ++ap_i)
      {
        cJSON *ap_record = NULL;
        char action_ref[256];
        char source_action_point_id[YAI_SOURCE_ACTION_POINT_ID_MAX];
        if (snprintf(action_ref, sizeof(action_ref), "%s/%d", action_points_ref, ap_i) >= (int)sizeof(action_ref))
        {
          continue;
        }
        if (yai_source_id_action_point(source_action_point_id,
                                       sizeof(source_action_point_id),
                                       source_binding_id,
                                       action_ref) != 0)
        {
          continue;
        }
        ap_record = cJSON_CreateObject();
        if (!ap_record)
        {
          continue;
        }
        cJSON_AddStringToObject(ap_record, "type", "yai.source_action_point.v1");
        cJSON_AddStringToObject(ap_record, "source_action_point_id", source_action_point_id);
        cJSON_AddStringToObject(ap_record, "source_node_id", source_node_id);
        cJSON_AddStringToObject(ap_record, "source_binding_id", source_binding_id);
        cJSON_AddStringToObject(ap_record, "action_kind", "unknown");
        cJSON_AddStringToObject(ap_record, "action_ref", action_ref);
        cJSON_AddStringToObject(ap_record, "mediation_scope", mediation_scope);
        cJSON_AddStringToObject(ap_record, "enforcement_scope", enforcement_scope);
        cJSON_AddStringToObject(ap_record, "controllability_state", "delegated_candidate");
        cJSON_AddNumberToObject(ap_record, "updated_at_epoch", (double)time(NULL));
        if (append_source_record(workspace_id,
                                 YAI_SOURCE_RECORD_CLASS_ACTION_POINT,
                                 ap_record,
                                 err,
                                 sizeof(err)) != 0)
        {
          cJSON_Delete(ap_record);
          cJSON_Delete(binding);
          cJSON_Delete(membership);
          cJSON_Delete(snapshot);
          cJSON_Delete(capability);
          (void)set_reason(reason, reason_cap, err[0] ? err : "source_attach_action_point_persistence_failed");
          return -4;
        }
        cJSON_Delete(ap_record);
      }
    }
  }
  cJSON_Delete(binding);
  cJSON_Delete(membership);
  cJSON_Delete(snapshot);
  cJSON_Delete(capability);

  memset(&reg, 0, sizeof(reg));
  (void)snprintf(reg.workspace_id, sizeof(reg.workspace_id), "%s", target_ws);
  (void)snprintf(reg.source_node_id, sizeof(reg.source_node_id), "%s", source_node_id);
  (void)snprintf(reg.source_binding_id, sizeof(reg.source_binding_id), "%s", source_binding_id);
  (void)snprintf(reg.daemon_instance_id, sizeof(reg.daemon_instance_id), "%s", daemon_instance_id);
  (void)snprintf(reg.peer_role, sizeof(reg.peer_role), "%s", peer_role);
  (void)snprintf(reg.peer_scope, sizeof(reg.peer_scope), "%s", peer_scope);
  (void)snprintf(reg.peer_state, sizeof(reg.peer_state), "%s", peer_state);
  reg.backlog_queued = 0;
  reg.backlog_retry_due = 0;
  reg.backlog_failed = 0;
  (void)snprintf(reg.coverage_ref, sizeof(reg.coverage_ref), "%s", coverage_ref);
  (void)snprintf(reg.overlap_state, sizeof(reg.overlap_state), "%s", overlap_state);
  reg.last_seen_epoch = (int64_t)time(NULL);
  reg.last_activity_epoch = 0;
  reg.updated_at_epoch = (int64_t)time(NULL);
  (void)yai_owner_peer_registry_upsert(&reg, err, sizeof(err));

  mediation_json(med, med_json, sizeof(med_json));

  if (snprintf(out_json,
               out_cap,
               "{\"type\":\"yai.source.attach.reply.v1\",\"workspace_id\":\"%s\",\"owner_workspace_id\":\"%s\",\"source_node_id\":\"%s\",\"source_binding_id\":\"%s\",\"workspace_peer_membership_id\":\"%s\",\"source_policy_snapshot_id\":\"%s\",\"source_capability_envelope_id\":\"%s\",\"policy_snapshot_version\":\"%s\",\"distribution_target_ref\":\"%s\",\"delegated_observation_scope\":\"%s\",\"delegated_mediation_scope\":\"%s\",\"delegated_enforcement_scope\":\"%s\",\"delegated_validity_state\":\"%s\",\"delegated_refresh_state\":\"%s\",\"delegated_revoke_state\":\"%s\",\"grant_valid_from_epoch\":%lld,\"grant_refresh_after_epoch\":%lld,\"grant_expires_at_epoch\":%lld,\"policy_snapshot_issued_at_epoch\":%lld,\"policy_snapshot_refresh_after_epoch\":%lld,\"policy_snapshot_expires_at_epoch\":%lld,\"capability_issued_at_epoch\":%lld,\"capability_refresh_after_epoch\":%lld,\"capability_expires_at_epoch\":%lld,\"peer_role\":\"%s\",\"peer_scope\":\"%s\",\"peer_state\":\"%s\",\"attachment_status\":\"attached\",\"mediation\":%s}",
               workspace_id,
               target_ws,
               source_node_id,
               source_binding_id,
               workspace_peer_membership_id,
               source_policy_snapshot_id,
               source_capability_envelope_id,
               snapshot_version,
               distribution_target_ref,
               observation_scope,
               mediation_scope,
               enforcement_scope,
               delegated_validity_state,
               delegated_refresh_state,
               delegated_revoke_state,
               (long long)valid_from_epoch,
               (long long)refresh_after_epoch,
               (long long)expires_at_epoch,
               (long long)valid_from_epoch,
               (long long)refresh_after_epoch,
               (long long)expires_at_epoch,
               (long long)valid_from_epoch,
               (long long)refresh_after_epoch,
               (long long)expires_at_epoch,
               peer_role,
               peer_scope,
               peer_state,
               med_json) <= 0)
  {
    (void)set_reason(reason, reason_cap, "source_attach_response_encode_failed");
    return -4;
  }

  (void)set_reason(reason, reason_cap, "source_attach_accepted");
  return 0;
}

static int append_emit_array(const char *workspace_id,
                             cJSON *root,
                             const char *array_key,
                             const char *single_key,
                             const char *record_class,
                             int *accepted,
                             char *err,
                             size_t err_cap)
{
  cJSON *arr = cJSON_GetObjectItem(root, array_key);
  cJSON *single = cJSON_GetObjectItem(root, single_key);
  int i = 0;

  if (!accepted)
  {
    return -1;
  }

  if (arr && cJSON_IsArray(arr))
  {
    int n = cJSON_GetArraySize(arr);
    for (i = 0; i < n; ++i)
    {
      cJSON *obj = cJSON_GetArrayItem(arr, i);
      if (!obj || !cJSON_IsObject(obj))
      {
        if (err && err_cap > 0)
        {
          (void)snprintf(err, err_cap, "%s_item_invalid", array_key);
        }
        return -1;
      }
      if (append_source_record(workspace_id, record_class, obj, err, err_cap) != 0)
      {
        return -1;
      }
      *accepted += 1;
    }
    return 0;
  }

  if (single && cJSON_IsObject(single))
  {
    if (append_source_record(workspace_id, record_class, single, err, err_cap) != 0)
    {
      return -1;
    }
    *accepted += 1;
  }
  return 0;
}

static int handle_emit(const char *workspace_id,
                       cJSON *payload,
                       const yai_source_plane_mediation_state_t *med,
                       char *out_json,
                       size_t out_cap,
                       char *reason,
                       size_t reason_cap)
{
  const char *source_node_id = json_string(payload, "source_node_id");
  const char *source_binding_id = json_string(payload, "source_binding_id");
  const char *idempotency_key = json_string(payload, "idempotency_key");
  char med_json[384];
  int accepted_assets = 0;
  int accepted_events = 0;
  int accepted_candidates = 0;
  int accepted_total = 0;
  char outcome_id[96];
  yai_emit_integrity_result_t integrity;
  cJSON *outcome = NULL;
  char err[160];

  if (!workspace_id || !yai_ws_id_is_valid(workspace_id) || !payload)
  {
    (void)set_reason(reason, reason_cap, "source_emit_bad_args");
    return -2;
  }

  if (!source_node_id || !source_node_id[0] || !source_binding_id || !source_binding_id[0])
  {
    (void)set_reason(reason, reason_cap, "source_emit_missing_source_or_binding");
    return -2;
  }
  if (!owner_trust_token_valid_for_node(payload, source_node_id))
  {
    (void)set_reason(reason, reason_cap, "source_emit_trust_bootstrap_required");
    return -3;
  }

  if (append_emit_array(workspace_id,
                        payload,
                        "source_assets",
                        "source_asset",
                        YAI_SOURCE_RECORD_CLASS_ASSET,
                        &accepted_assets,
                        err,
                        sizeof(err)) != 0)
  {
    (void)set_reason(reason, reason_cap, err[0] ? err : "source_emit_asset_append_failed");
    return -4;
  }

  if (append_emit_array(workspace_id,
                        payload,
                        "source_acquisition_events",
                        "source_acquisition_event",
                        YAI_SOURCE_RECORD_CLASS_ACQUISITION_EVENT,
                        &accepted_events,
                        err,
                        sizeof(err)) != 0)
  {
    (void)set_reason(reason, reason_cap, err[0] ? err : "source_emit_event_append_failed");
    return -4;
  }

  if (append_emit_array(workspace_id,
                        payload,
                        "source_evidence_candidates",
                        "source_evidence_candidate",
                        YAI_SOURCE_RECORD_CLASS_EVIDENCE_CANDIDATE,
                        &accepted_candidates,
                        err,
                        sizeof(err)) != 0)
  {
    (void)set_reason(reason, reason_cap, err[0] ? err : "source_emit_candidate_append_failed");
    return -4;
  }

  if (accepted_assets == 0 && accepted_events == 0 && accepted_candidates == 0)
  {
    (void)set_reason(reason, reason_cap, "source_emit_empty_payload");
    return -2;
  }
  accepted_total = accepted_assets + accepted_events + accepted_candidates;

  if (!idempotency_key)
  {
    idempotency_key = "unset";
  }
  emit_integrity_default(&integrity);
  if (classify_emit_integrity(workspace_id,
                              source_node_id,
                              source_binding_id,
                              idempotency_key,
                              payload,
                              accepted_total,
                              &integrity) != 0)
  {
    (void)set_reason(reason, reason_cap, "source_emit_integrity_classification_failed");
    return -4;
  }
  if (make_ingest_outcome_id(outcome_id, sizeof(outcome_id)) != 0)
  {
    (void)set_reason(reason, reason_cap, "source_emit_outcome_id_generation_failed");
    return -4;
  }
  outcome = cJSON_CreateObject();
  if (!outcome)
  {
    (void)set_reason(reason, reason_cap, "source_emit_outcome_allocation_failed");
    return -4;
  }
  cJSON_AddStringToObject(outcome, "type", "yai.source_ingest_outcome.v1");
  cJSON_AddStringToObject(outcome, "source_ingest_outcome_id", outcome_id);
  cJSON_AddStringToObject(outcome, "owner_workspace_id", workspace_id);
  cJSON_AddStringToObject(outcome, "source_node_id", source_node_id);
  cJSON_AddStringToObject(outcome, "source_binding_id", source_binding_id);
  cJSON_AddStringToObject(outcome, "idempotency_key", idempotency_key);
  cJSON_AddStringToObject(outcome, "classification", integrity.classification);
  cJSON_AddStringToObject(outcome, "handling_action", integrity.handling_action);
  cJSON_AddStringToObject(outcome, "ordering_status", integrity.ordering_status);
  cJSON_AddStringToObject(outcome, "replay_status", integrity.replay_status);
  cJSON_AddStringToObject(outcome, "overlap_status", integrity.overlap_status);
  cJSON_AddStringToObject(outcome, "asset_key_ref", integrity.asset_key_ref[0] ? integrity.asset_key_ref : "unset");
  cJSON_AddNumberToObject(outcome, "accepted_asset_count", accepted_assets);
  cJSON_AddNumberToObject(outcome, "accepted_event_count", accepted_events);
  cJSON_AddNumberToObject(outcome, "accepted_candidate_count", accepted_candidates);
  cJSON_AddNumberToObject(outcome, "accepted_total_count", accepted_total);
  cJSON_AddBoolToObject(outcome, "review_required", integrity.review_required ? 1 : 0);
  cJSON_AddNumberToObject(outcome, "observed_at_epoch", (double)time(NULL));
  if (append_source_record(workspace_id,
                           YAI_SOURCE_RECORD_CLASS_INGEST_OUTCOME,
                           outcome,
                           err,
                           sizeof(err)) != 0)
  {
    cJSON_Delete(outcome);
    (void)set_reason(reason, reason_cap, err[0] ? err : "source_emit_outcome_persistence_failed");
    return -4;
  }
  cJSON_Delete(outcome);
  (void)yai_owner_peer_registry_note_emit(workspace_id,
                                          source_node_id,
                                          source_binding_id,
                                          1,
                                          err,
                                          sizeof(err));
  (void)yai_owner_peer_registry_note_integrity(workspace_id,
                                               source_node_id,
                                               source_binding_id,
                                               integrity.classification,
                                               integrity.handling_action,
                                               integrity.ordering_late,
                                               integrity.replay_detected,
                                               integrity.overlap_detected,
                                               integrity.conflict_detected,
                                               integrity.review_required,
                                               err,
                                               sizeof(err));
  mediation_json(med, med_json, sizeof(med_json));

  if (snprintf(out_json,
               out_cap,
               "{\"type\":\"yai.source.emit.reply.v1\",\"workspace_id\":\"%s\",\"source_node_id\":\"%s\",\"source_binding_id\":\"%s\",\"idempotency_key\":\"%s\",\"accepted\":{\"source_asset\":%d,\"source_acquisition_event\":%d,\"source_evidence_candidate\":%d},\"integrity\":{\"classification\":\"%s\",\"handling_action\":\"%s\",\"ordering_status\":\"%s\",\"replay_status\":\"%s\",\"overlap_status\":\"%s\",\"review_required\":%s},\"mediation\":%s}",
               workspace_id,
               source_node_id,
               source_binding_id,
               idempotency_key,
               accepted_assets,
               accepted_events,
               accepted_candidates,
               integrity.classification,
               integrity.handling_action,
               integrity.ordering_status,
               integrity.replay_status,
               integrity.overlap_status,
               integrity.review_required ? "true" : "false",
               med_json) <= 0)
  {
    (void)set_reason(reason, reason_cap, "source_emit_response_encode_failed");
    return -4;
  }

  (void)set_reason(reason, reason_cap, "source_emit_accepted");
  return 0;
}

static int handle_status(const char *workspace_id,
                         cJSON *payload,
                         const yai_source_plane_mediation_state_t *med,
                         char *out_json,
                         size_t out_cap,
                         char *reason,
                         size_t reason_cap)
{
  const char *source_node_id = json_string(payload, "source_node_id");
  const char *source_binding_id = json_string_or(payload, "source_binding_id", "binding-unset");
  const char *daemon_instance_id = json_string(payload, "daemon_instance_id");
  const char *health = json_string(payload, "health");
  const char *peer_role = json_string_or(payload, "peer_role", "general");
  const char *peer_scope = json_string_or(payload, "peer_scope", "workspace/default");
  const char *coverage_ref = json_string_or(payload, "coverage_ref", "coverage://workspace/default");
  const char *overlap_state = json_string_or(payload, "overlap_state", "unknown");
  const char *source_enrollment_grant_id = json_string_or(payload, "source_enrollment_grant_id", "grant-unset");
  const char *source_policy_snapshot_id_in = json_string(payload, "source_policy_snapshot_id");
  const char *source_capability_envelope_id_in = json_string(payload, "source_capability_envelope_id");
  const char *policy_snapshot_version = json_string_or(payload, "policy_snapshot_version", "ws-policy-snapshot-v1");
  const char *delegated_observation_scope = json_string_or(payload, "delegated_observation_scope", "workspace/default");
  const char *delegated_mediation_scope = json_string_or(payload, "delegated_mediation_scope", "none");
  const char *delegated_enforcement_scope = json_string_or(payload, "delegated_enforcement_scope", "none");
  int64_t now_epoch = (int64_t)time(NULL);
  int64_t grant_valid_from_epoch = json_i64(payload, "grant_valid_from_epoch", now_epoch);
  int64_t grant_refresh_after_epoch = json_i64(payload, "grant_refresh_after_epoch", now_epoch + 300);
  int64_t grant_expires_at_epoch = json_i64(payload, "grant_expires_at_epoch", now_epoch + 1800);
  int64_t snapshot_issued_at_epoch = json_i64(payload, "policy_snapshot_issued_at_epoch", now_epoch);
  int64_t snapshot_refresh_after_epoch = json_i64(payload, "policy_snapshot_refresh_after_epoch", now_epoch + 300);
  int64_t snapshot_expires_at_epoch = json_i64(payload, "policy_snapshot_expires_at_epoch", now_epoch + 1800);
  int64_t capability_issued_at_epoch = json_i64(payload, "capability_issued_at_epoch", now_epoch);
  int64_t capability_refresh_after_epoch = json_i64(payload, "capability_refresh_after_epoch", now_epoch + 300);
  int64_t capability_expires_at_epoch = json_i64(payload, "capability_expires_at_epoch", now_epoch + 1800);
  int delegated_revoked = json_bool(payload, "delegated_revoked", 0);
  const char *delegated_validity_state = json_string_or(payload, "delegated_validity_state", "valid");
  const char *delegated_refresh_state = json_string_or(payload, "delegated_refresh_state", "not_required");
  const char *delegated_revoke_state = delegated_revoked ? "revoked" : json_string_or(payload, "delegated_revoke_state", "active");
  const char *delegated_fallback_mode = json_string_or(payload, "delegated_fallback_mode", "full_delegated");
  const char *delegated_stale_reason = json_string_or(payload, "delegated_stale_reason", "none");
  int64_t backlog_queued = json_i64(payload, "backlog_queued", -1);
  int64_t backlog_retry_due = json_i64(payload, "backlog_retry_due", -1);
  int64_t backlog_failed = json_i64(payload, "backlog_failed", -1);
  char source_policy_snapshot_id[YAI_SOURCE_POLICY_SNAPSHOT_ID_MAX];
  char source_capability_envelope_id[YAI_SOURCE_CAPABILITY_ENVELOPE_ID_MAX];
  char distribution_target_ref[YAI_SOURCE_REF_MAX];
  char workspace_peer_membership_id[YAI_SOURCE_WORKSPACE_PEER_MEMBERSHIP_ID_MAX];
  char med_json[384];
  cJSON *inst = NULL;
  cJSON *membership = NULL;
  cJSON *snapshot = NULL;
  cJSON *capability = NULL;
  yai_owner_peer_registry_entry_t reg;
  char err[160];

  if (!workspace_id || !yai_ws_id_is_valid(workspace_id) || !payload ||
      !source_node_id || !source_node_id[0])
  {
    (void)set_reason(reason, reason_cap, "source_status_bad_args");
    return -2;
  }
  if (!owner_trust_token_valid_for_node(payload, source_node_id))
  {
    (void)set_reason(reason, reason_cap, "source_status_trust_bootstrap_required");
    return -3;
  }

  if (!daemon_instance_id)
  {
    daemon_instance_id = "daemon-instance-unset";
  }
  if (!health)
  {
    health = "unknown";
  }
  if (yai_source_id_workspace_peer_membership(workspace_peer_membership_id,
                                              sizeof(workspace_peer_membership_id),
                                              workspace_id,
                                              source_node_id,
                                              source_binding_id) != 0)
  {
    (void)set_reason(reason, reason_cap, "workspace_peer_membership_id_generation_failed");
    return -4;
  }
  if (source_policy_snapshot_id_in && source_policy_snapshot_id_in[0])
  {
    (void)snprintf(source_policy_snapshot_id, sizeof(source_policy_snapshot_id), "%s", source_policy_snapshot_id_in);
  }
  else if (yai_source_id_policy_snapshot(source_policy_snapshot_id,
                                         sizeof(source_policy_snapshot_id),
                                         source_node_id,
                                         daemon_instance_id,
                                         workspace_id) != 0)
  {
    (void)set_reason(reason, reason_cap, "source_policy_snapshot_id_generation_failed");
    return -4;
  }
  if (source_capability_envelope_id_in && source_capability_envelope_id_in[0])
  {
    (void)snprintf(source_capability_envelope_id,
                   sizeof(source_capability_envelope_id),
                   "%s",
                   source_capability_envelope_id_in);
  }
  else if (yai_source_id_capability_envelope(source_capability_envelope_id,
                                             sizeof(source_capability_envelope_id),
                                             source_binding_id,
                                             source_node_id,
                                             workspace_id) != 0)
  {
    (void)set_reason(reason, reason_cap, "source_capability_envelope_id_generation_failed");
    return -4;
  }
  if (make_distribution_target_ref(distribution_target_ref,
                                   sizeof(distribution_target_ref),
                                   workspace_id,
                                   source_node_id,
                                   daemon_instance_id,
                                   source_binding_id) != 0)
  {
    (void)set_reason(reason, reason_cap, "source_distribution_target_encode_failed");
    return -4;
  }

  inst = cJSON_CreateObject();
  membership = cJSON_CreateObject();
  snapshot = cJSON_CreateObject();
  capability = cJSON_CreateObject();
  if (!inst || !membership || !snapshot || !capability)
  {
    cJSON_Delete(inst);
    cJSON_Delete(membership);
    cJSON_Delete(snapshot);
    cJSON_Delete(capability);
    (void)set_reason(reason, reason_cap, "source_status_allocation_failed");
    return -4;
  }
  cJSON_AddStringToObject(inst, "type", "yai.source_daemon_instance.v1");
  cJSON_AddStringToObject(inst, "daemon_instance_id", daemon_instance_id);
  cJSON_AddStringToObject(inst, "source_node_id", source_node_id);
  cJSON_AddStringToObject(inst, "session_marker", "status_update");
  cJSON_AddNumberToObject(inst, "started_at_epoch", (double)time(NULL));
  cJSON_AddStringToObject(inst, "health", health);
  cJSON_AddStringToObject(inst, "build_marker", "yd4-v1");

  cJSON_AddStringToObject(membership, "type", "yai.workspace_peer_membership.v1");
  cJSON_AddStringToObject(membership, "workspace_peer_membership_id", workspace_peer_membership_id);
  cJSON_AddStringToObject(membership, "owner_workspace_id", workspace_id);
  cJSON_AddStringToObject(membership, "source_node_id", source_node_id);
  cJSON_AddStringToObject(membership, "source_binding_id", source_binding_id);
  cJSON_AddStringToObject(membership, "daemon_instance_id", daemon_instance_id);
  cJSON_AddStringToObject(membership, "peer_role", peer_role);
  cJSON_AddStringToObject(membership, "peer_scope", peer_scope);
  cJSON_AddStringToObject(membership, "peer_state", health);
  cJSON_AddNumberToObject(membership, "backlog_queued", (double)backlog_queued);
  cJSON_AddNumberToObject(membership, "backlog_retry_due", (double)backlog_retry_due);
  cJSON_AddNumberToObject(membership, "backlog_failed", (double)backlog_failed);
  cJSON_AddStringToObject(membership, "coverage_ref", coverage_ref);
  cJSON_AddStringToObject(membership, "overlap_state", overlap_state);
  cJSON_AddNumberToObject(membership, "updated_at_epoch", (double)time(NULL));

  cJSON_AddStringToObject(snapshot, "type", "yai.source_policy_snapshot.v1");
  cJSON_AddStringToObject(snapshot, "source_policy_snapshot_id", source_policy_snapshot_id);
  cJSON_AddStringToObject(snapshot, "source_node_id", source_node_id);
  cJSON_AddStringToObject(snapshot, "daemon_instance_id", daemon_instance_id);
  cJSON_AddStringToObject(snapshot, "owner_workspace_id", workspace_id);
  cJSON_AddStringToObject(snapshot, "source_enrollment_grant_id", source_enrollment_grant_id);
  cJSON_AddStringToObject(snapshot, "snapshot_version", policy_snapshot_version);
  cJSON_AddStringToObject(snapshot, "distribution_target_ref", distribution_target_ref);
  cJSON_AddStringToObject(snapshot, "validity_state", delegated_validity_state);
  cJSON_AddStringToObject(snapshot, "refresh_state", delegated_refresh_state);
  cJSON_AddStringToObject(snapshot, "revoke_state", delegated_revoke_state);
  cJSON_AddStringToObject(snapshot, "fallback_mode", delegated_fallback_mode);
  cJSON_AddStringToObject(snapshot, "stale_reason", delegated_stale_reason);
  cJSON_AddNumberToObject(snapshot, "valid_from_epoch", (double)grant_valid_from_epoch);
  cJSON_AddNumberToObject(snapshot, "refresh_after_epoch", (double)snapshot_refresh_after_epoch);
  cJSON_AddNumberToObject(snapshot, "expires_at_epoch", (double)snapshot_expires_at_epoch);
  cJSON_AddBoolToObject(snapshot, "revoked", delegated_revoked ? 1 : 0);
  cJSON_AddNumberToObject(snapshot, "issued_at_epoch", (double)snapshot_issued_at_epoch);

  cJSON_AddStringToObject(capability, "type", "yai.source_capability_envelope.v1");
  cJSON_AddStringToObject(capability, "source_capability_envelope_id", source_capability_envelope_id);
  cJSON_AddStringToObject(capability, "source_node_id", source_node_id);
  cJSON_AddStringToObject(capability, "daemon_instance_id", daemon_instance_id);
  cJSON_AddStringToObject(capability, "source_binding_id", source_binding_id);
  cJSON_AddStringToObject(capability, "owner_workspace_id", workspace_id);
  cJSON_AddStringToObject(capability, "source_enrollment_grant_id", source_enrollment_grant_id);
  cJSON_AddStringToObject(capability, "observation_scope", delegated_observation_scope);
  cJSON_AddStringToObject(capability, "mediation_scope", delegated_mediation_scope);
  cJSON_AddStringToObject(capability, "enforcement_scope", delegated_enforcement_scope);
  cJSON_AddStringToObject(capability, "distribution_target_ref", distribution_target_ref);
  cJSON_AddStringToObject(capability, "validity_state", delegated_validity_state);
  cJSON_AddStringToObject(capability, "refresh_state", delegated_refresh_state);
  cJSON_AddStringToObject(capability, "revoke_state", delegated_revoke_state);
  cJSON_AddStringToObject(capability, "fallback_mode", delegated_fallback_mode);
  cJSON_AddStringToObject(capability, "stale_reason", delegated_stale_reason);
  cJSON_AddNumberToObject(capability, "valid_from_epoch", (double)grant_valid_from_epoch);
  cJSON_AddNumberToObject(capability, "refresh_after_epoch", (double)capability_refresh_after_epoch);
  cJSON_AddNumberToObject(capability, "expires_at_epoch", (double)capability_expires_at_epoch);
  cJSON_AddBoolToObject(capability, "revoked", delegated_revoked ? 1 : 0);
  cJSON_AddNumberToObject(capability, "issued_at_epoch", (double)capability_issued_at_epoch);

  if (append_source_record(workspace_id, YAI_SOURCE_RECORD_CLASS_DAEMON_INSTANCE, inst, err, sizeof(err)) != 0)
  {
    cJSON_Delete(inst);
    cJSON_Delete(membership);
    cJSON_Delete(snapshot);
    cJSON_Delete(capability);
    (void)set_reason(reason, reason_cap, err[0] ? err : "source_status_persistence_failed");
    return -4;
  }
  if (append_source_record(workspace_id,
                           YAI_SOURCE_RECORD_CLASS_WORKSPACE_PEER_MEMBERSHIP,
                           membership,
                           err,
                           sizeof(err)) != 0)
  {
    cJSON_Delete(inst);
    cJSON_Delete(membership);
    cJSON_Delete(snapshot);
    cJSON_Delete(capability);
    (void)set_reason(reason, reason_cap, err[0] ? err : "workspace_peer_membership_persistence_failed");
    return -4;
  }
  if (append_source_record(workspace_id,
                           YAI_SOURCE_RECORD_CLASS_POLICY_SNAPSHOT,
                           snapshot,
                           err,
                           sizeof(err)) != 0)
  {
    cJSON_Delete(inst);
    cJSON_Delete(membership);
    cJSON_Delete(snapshot);
    cJSON_Delete(capability);
    (void)set_reason(reason, reason_cap, err[0] ? err : "source_status_policy_snapshot_persistence_failed");
    return -4;
  }
  if (append_source_record(workspace_id,
                           YAI_SOURCE_RECORD_CLASS_CAPABILITY_ENVELOPE,
                           capability,
                           err,
                           sizeof(err)) != 0)
  {
    cJSON_Delete(inst);
    cJSON_Delete(membership);
    cJSON_Delete(snapshot);
    cJSON_Delete(capability);
    (void)set_reason(reason, reason_cap, err[0] ? err : "source_status_capability_envelope_persistence_failed");
    return -4;
  }
  cJSON_Delete(inst);
  cJSON_Delete(membership);
  cJSON_Delete(snapshot);
  cJSON_Delete(capability);

  memset(&reg, 0, sizeof(reg));
  (void)snprintf(reg.workspace_id, sizeof(reg.workspace_id), "%s", workspace_id);
  (void)snprintf(reg.source_node_id, sizeof(reg.source_node_id), "%s", source_node_id);
  (void)snprintf(reg.source_binding_id, sizeof(reg.source_binding_id), "%s", source_binding_id);
  (void)snprintf(reg.daemon_instance_id, sizeof(reg.daemon_instance_id), "%s", daemon_instance_id);
  (void)snprintf(reg.peer_role, sizeof(reg.peer_role), "%s", peer_role);
  (void)snprintf(reg.peer_scope, sizeof(reg.peer_scope), "%s", peer_scope);
  (void)snprintf(reg.peer_state, sizeof(reg.peer_state), "%s", health);
  reg.backlog_queued = backlog_queued;
  reg.backlog_retry_due = backlog_retry_due;
  reg.backlog_failed = backlog_failed;
  (void)snprintf(reg.coverage_ref, sizeof(reg.coverage_ref), "%s", coverage_ref);
  (void)snprintf(reg.overlap_state, sizeof(reg.overlap_state), "%s", overlap_state);
  reg.last_seen_epoch = (int64_t)time(NULL);
  reg.last_activity_epoch = (int64_t)time(NULL);
  reg.updated_at_epoch = (int64_t)time(NULL);
  (void)yai_owner_peer_registry_upsert(&reg, err, sizeof(err));

  mediation_json(med, med_json, sizeof(med_json));

  if (snprintf(out_json,
               out_cap,
               "{\"type\":\"yai.source.status.reply.v1\",\"workspace_id\":\"%s\",\"source_node_id\":\"%s\",\"daemon_instance_id\":\"%s\",\"source_binding_id\":\"%s\",\"workspace_peer_membership_id\":\"%s\",\"source_policy_snapshot_id\":\"%s\",\"source_capability_envelope_id\":\"%s\",\"policy_snapshot_version\":\"%s\",\"distribution_target_ref\":\"%s\",\"delegated_observation_scope\":\"%s\",\"delegated_mediation_scope\":\"%s\",\"delegated_enforcement_scope\":\"%s\",\"delegated_validity_state\":\"%s\",\"delegated_refresh_state\":\"%s\",\"delegated_revoke_state\":\"%s\",\"delegated_fallback_mode\":\"%s\",\"delegated_stale_reason\":\"%s\",\"grant_valid_from_epoch\":%lld,\"grant_refresh_after_epoch\":%lld,\"grant_expires_at_epoch\":%lld,\"policy_snapshot_issued_at_epoch\":%lld,\"policy_snapshot_refresh_after_epoch\":%lld,\"policy_snapshot_expires_at_epoch\":%lld,\"capability_issued_at_epoch\":%lld,\"capability_refresh_after_epoch\":%lld,\"capability_expires_at_epoch\":%lld,\"health\":\"%s\",\"backlog_queued\":%lld,\"backlog_retry_due\":%lld,\"backlog_failed\":%lld,\"coverage_ref\":\"%s\",\"overlap_state\":\"%s\",\"mediation\":%s}",
               workspace_id,
               source_node_id,
               daemon_instance_id,
               source_binding_id,
               workspace_peer_membership_id,
               source_policy_snapshot_id,
               source_capability_envelope_id,
               policy_snapshot_version,
               distribution_target_ref,
               delegated_observation_scope,
               delegated_mediation_scope,
               delegated_enforcement_scope,
               delegated_validity_state,
               delegated_refresh_state,
               delegated_revoke_state,
               delegated_fallback_mode,
               delegated_stale_reason,
               (long long)grant_valid_from_epoch,
               (long long)grant_refresh_after_epoch,
               (long long)grant_expires_at_epoch,
               (long long)snapshot_issued_at_epoch,
               (long long)snapshot_refresh_after_epoch,
               (long long)snapshot_expires_at_epoch,
               (long long)capability_issued_at_epoch,
               (long long)capability_refresh_after_epoch,
               (long long)capability_expires_at_epoch,
               health,
               (long long)backlog_queued,
               (long long)backlog_retry_due,
               (long long)backlog_failed,
               coverage_ref,
               overlap_state,
               med_json) <= 0)
  {
    (void)set_reason(reason, reason_cap, "source_status_response_encode_failed");
    return -4;
  }

  (void)set_reason(reason, reason_cap, "source_status_accepted");
  return 0;
}

int yai_exec_source_ingest_operation_known(const char *command_id)
{
  return command_id &&
         (strcmp(command_id, "yai.source.enroll") == 0 ||
          strcmp(command_id, "yai.source.attach") == 0 ||
          strcmp(command_id, "yai.source.emit") == 0 ||
          strcmp(command_id, "yai.source.status") == 0);
}

int yai_exec_source_ingest_handle(const char *workspace_id,
                                  const char *command_id,
                                  const char *payload_json,
                                  char *out_json,
                                  size_t out_cap,
                                  char *out_reason,
                                  size_t reason_cap)
{
  int rc = -4;
  cJSON *payload = NULL;
  yai_source_plane_mediation_state_t med;
  char med_err[128];

  if (!workspace_id || !command_id || !payload_json || !out_json || out_cap == 0)
  {
    (void)set_reason(out_reason, reason_cap, "source_ingest_bad_args");
    return -2;
  }
  if (!yai_ws_id_is_valid(workspace_id))
  {
    (void)set_reason(out_reason, reason_cap, "source_ingest_invalid_workspace");
    return -2;
  }
  if (!yai_exec_source_ingest_operation_known(command_id))
  {
    (void)set_reason(out_reason, reason_cap, "source_ingest_unknown_command");
    return -3;
  }
  if (yai_exec_source_plane_prepare(command_id, &med, med_err, sizeof(med_err)) != 0)
  {
    (void)set_reason(out_reason, reason_cap, med_err[0] ? med_err : "source_plane_mediation_failed");
    return -3;
  }

  payload = cJSON_Parse(payload_json);
  if (!payload)
  {
    (void)set_reason(out_reason, reason_cap, "source_ingest_payload_not_json");
    return -2;
  }

  if (strcmp(command_id, "yai.source.enroll") == 0)
  {
    rc = handle_enroll(workspace_id, payload, &med, out_json, out_cap, out_reason, reason_cap);
  }
  else if (strcmp(command_id, "yai.source.attach") == 0)
  {
    rc = handle_attach(workspace_id, payload, &med, out_json, out_cap, out_reason, reason_cap);
  }
  else if (strcmp(command_id, "yai.source.emit") == 0)
  {
    rc = handle_emit(workspace_id, payload, &med, out_json, out_cap, out_reason, reason_cap);
  }
  else if (strcmp(command_id, "yai.source.status") == 0)
  {
    rc = handle_status(workspace_id, payload, &med, out_json, out_cap, out_reason, reason_cap);
  }
  else
  {
    (void)set_reason(out_reason, reason_cap, "source_ingest_unmapped_command");
    rc = -3;
  }

  cJSON_Delete(payload);
  return rc;
}
