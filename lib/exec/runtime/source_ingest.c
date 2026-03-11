#define _POSIX_C_SOURCE 200809L

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#include <yai/core/workspace.h>
#include <yai/data/records.h>
#include <yai/daemon/source_ids.h>
#include <yai/daemon/source_plane_model.h>
#include <yai/exec/source_ingest.h>
#include <yai/exec/source_plane.h>
#include <yai/graph/materialization.h>
#include <yai/protocol/source_plane_contract.h>

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
  char trust_artifact_id[128];
  char trust_artifact_token[128];
  char med_json[384];
  cJSON *node = NULL;
  cJSON *inst = NULL;
  cJSON *link = NULL;
  cJSON *grant = NULL;
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
  if (!node || !inst || !link || !grant)
  {
    cJSON_Delete(node);
    cJSON_Delete(inst);
    cJSON_Delete(link);
    cJSON_Delete(grant);
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
  cJSON_AddNumberToObject(grant, "issued_at_epoch", (double)time(NULL));

  if (append_source_record(workspace_id, YAI_SOURCE_RECORD_CLASS_NODE, node, err, sizeof(err)) != 0 ||
      append_source_record(workspace_id, YAI_SOURCE_RECORD_CLASS_DAEMON_INSTANCE, inst, err, sizeof(err)) != 0 ||
      append_source_record(workspace_id, YAI_SOURCE_RECORD_CLASS_OWNER_LINK, link, err, sizeof(err)) != 0 ||
      append_source_record(workspace_id, YAI_SOURCE_RECORD_CLASS_ENROLLMENT_GRANT, grant, err, sizeof(err)) != 0)
  {
    cJSON_Delete(node);
    cJSON_Delete(inst);
    cJSON_Delete(link);
    cJSON_Delete(grant);
    (void)set_reason(reason, reason_cap, err[0] ? err : "source_enroll_persistence_failed");
    return -4;
  }

  cJSON_Delete(node);
  cJSON_Delete(inst);
  cJSON_Delete(link);
  cJSON_Delete(grant);

  mediation_json(med, med_json, sizeof(med_json));

  if (snprintf(out_json,
               out_cap,
               "{\"type\":\"yai.source.enroll.reply.v1\",\"workspace_id\":\"%s\",\"source_node_id\":\"%s\",\"daemon_instance_id\":\"%s\",\"owner_link_id\":\"%s\",\"source_enrollment_grant_id\":\"%s\",\"owner_trust_artifact_id\":\"%s\",\"owner_trust_artifact_token\":\"%s\",\"enrollment_decision\":\"%s\",\"ready_for_attach\":%s,\"registered\":%s,\"mediation\":%s}",
               workspace_id,
               node_id,
               daemon_id,
               owner_link_id,
               enrollment_grant_id,
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
  const char *owner_workspace_id = json_string(payload, "owner_workspace_id");
  char source_binding_id[YAI_SOURCE_BINDING_ID_MAX];
  char med_json[384];
  cJSON *binding = NULL;
  char err[160];
  const char *target_ws = owner_workspace_id && owner_workspace_id[0] ? owner_workspace_id : workspace_id;

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
  cJSON_AddStringToObject(binding, "attachment_status", "attached");
  cJSON_AddStringToObject(binding, "constraints_ref", constraints_ref);

  if (append_source_record(workspace_id, YAI_SOURCE_RECORD_CLASS_BINDING, binding, err, sizeof(err)) != 0)
  {
    cJSON_Delete(binding);
    (void)set_reason(reason, reason_cap, err[0] ? err : "source_attach_persistence_failed");
    return -4;
  }
  cJSON_Delete(binding);
  mediation_json(med, med_json, sizeof(med_json));

  if (snprintf(out_json,
               out_cap,
               "{\"type\":\"yai.source.attach.reply.v1\",\"workspace_id\":\"%s\",\"owner_workspace_id\":\"%s\",\"source_node_id\":\"%s\",\"source_binding_id\":\"%s\",\"attachment_status\":\"attached\",\"mediation\":%s}",
               workspace_id,
               target_ws,
               source_node_id,
               source_binding_id,
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

  if (!idempotency_key)
  {
    idempotency_key = "unset";
  }
  mediation_json(med, med_json, sizeof(med_json));

  if (snprintf(out_json,
               out_cap,
               "{\"type\":\"yai.source.emit.reply.v1\",\"workspace_id\":\"%s\",\"source_node_id\":\"%s\",\"source_binding_id\":\"%s\",\"idempotency_key\":\"%s\",\"accepted\":{\"source_asset\":%d,\"source_acquisition_event\":%d,\"source_evidence_candidate\":%d},\"mediation\":%s}",
               workspace_id,
               source_node_id,
               source_binding_id,
               idempotency_key,
               accepted_assets,
               accepted_events,
               accepted_candidates,
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
  const char *daemon_instance_id = json_string(payload, "daemon_instance_id");
  const char *health = json_string(payload, "health");
  char med_json[384];
  cJSON *inst = NULL;
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

  inst = cJSON_CreateObject();
  if (!inst)
  {
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

  if (append_source_record(workspace_id, YAI_SOURCE_RECORD_CLASS_DAEMON_INSTANCE, inst, err, sizeof(err)) != 0)
  {
    cJSON_Delete(inst);
    (void)set_reason(reason, reason_cap, err[0] ? err : "source_status_persistence_failed");
    return -4;
  }
  cJSON_Delete(inst);
  mediation_json(med, med_json, sizeof(med_json));

  if (snprintf(out_json,
               out_cap,
               "{\"type\":\"yai.source.status.reply.v1\",\"workspace_id\":\"%s\",\"source_node_id\":\"%s\",\"daemon_instance_id\":\"%s\",\"health\":\"%s\",\"mediation\":%s}",
               workspace_id,
               source_node_id,
               daemon_instance_id,
               health,
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
