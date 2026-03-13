/* SPDX-License-Identifier: Apache-2.0 */

#include <yai/sdk/source.h>

#include <cJSON.h>

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define YAI_CONTROL_CALL_TYPE "yai.control.call.v1"
#define YAI_TARGET_PLANE_RUNTIME "runtime"

static void copy_str(char *out, size_t cap, const cJSON *v)
{
    if (!out || cap == 0) {
        return;
    }
    out[0] = '\0';
    if (cJSON_IsString(v) && v->valuestring) {
        (void)snprintf(out, cap, "%s", v->valuestring);
    }
}

static int bool_field(const cJSON *obj, const char *key)
{
    const cJSON *v = NULL;
    if (!obj || !key) {
        return 0;
    }
    v = cJSON_GetObjectItemCaseSensitive((cJSON *)obj, key);
    if (cJSON_IsBool(v)) {
        return cJSON_IsTrue(v) ? 1 : 0;
    }
    if (cJSON_IsNumber(v)) {
        return v->valueint != 0 ? 1 : 0;
    }
    return 0;
}

static int int_field(const cJSON *obj, const char *key)
{
    const cJSON *v = NULL;
    if (!obj || !key) {
        return 0;
    }
    v = cJSON_GetObjectItemCaseSensitive((cJSON *)obj, key);
    if (cJSON_IsNumber(v)) {
        return v->valueint;
    }
    return 0;
}

static cJSON *new_control_call(const char *command_id, const char *container_id)
{
    cJSON *root = cJSON_CreateObject();
    if (!root) {
        return NULL;
    }
    cJSON_AddStringToObject(root, "type", YAI_CONTROL_CALL_TYPE);
    cJSON_AddStringToObject(root, "target_plane", YAI_TARGET_PLANE_RUNTIME);
    cJSON_AddStringToObject(root, "command_id", command_id ? command_id : "");
    if (container_id && container_id[0]) {
        cJSON_AddStringToObject(root, "container_id", container_id);
    }
    return root;
}

static int call_json_obj(yai_sdk_client_t *client, cJSON *obj, yai_sdk_reply_t *out)
{
    char *json = NULL;
    int rc = YAI_SDK_IO;

    if (!client || !obj || !out) {
        return YAI_SDK_BAD_ARGS;
    }

    json = cJSON_PrintUnformatted(obj);
    if (!json) {
        return YAI_SDK_IO;
    }
    rc = yai_sdk_client_call_json(client, json, out);
    free(json);
    return rc;
}

static int parse_root_data(const yai_sdk_reply_t *reply, cJSON **out_root, cJSON **out_data)
{
    cJSON *root = NULL;
    cJSON *data = NULL;

    if (!reply || !out_root || !out_data || !reply->exec_reply_json) {
        return YAI_SDK_BAD_ARGS;
    }
    root = cJSON_Parse(reply->exec_reply_json);
    if (!root) {
        return YAI_SDK_PROTOCOL;
    }
    data = cJSON_GetObjectItemCaseSensitive(root, "data");
    if (!cJSON_IsObject(data)) {
        cJSON_Delete(root);
        return YAI_SDK_PROTOCOL;
    }
    *out_root = root;
    *out_data = data;
    return YAI_SDK_OK;
}

static void parse_mediation(const cJSON *data, yai_sdk_source_mediation_state_t *out)
{
    const cJSON *med = NULL;
    if (!data || !out) {
        return;
    }
    med = cJSON_GetObjectItemCaseSensitive((cJSON *)data, "mediation");
    if (!cJSON_IsObject(med)) {
        return;
    }
    copy_str(out->layer, sizeof(out->layer), cJSON_GetObjectItemCaseSensitive((cJSON *)med, "layer"));
    copy_str(out->stage, sizeof(out->stage), cJSON_GetObjectItemCaseSensitive((cJSON *)med, "stage"));
    copy_str(out->topology, sizeof(out->topology), cJSON_GetObjectItemCaseSensitive((cJSON *)med, "topology"));
    copy_str(out->route, sizeof(out->route), cJSON_GetObjectItemCaseSensitive((cJSON *)med, "route"));
    out->owner_canonical = bool_field(med, "owner_canonical");
    out->transport_ready = bool_field(med, "transport_ready");
    out->network_gate_ready = bool_field(med, "network_gate_ready");
    out->resource_gate_ready = bool_field(med, "resource_gate_ready");
    out->storage_gate_ready = bool_field(med, "storage_gate_ready");
}

static void parse_distribution(const cJSON *data, yai_sdk_source_distribution_state_t *out)
{
    if (!data || !out) {
        return;
    }
    copy_str(out->source_enrollment_grant_id,
             sizeof(out->source_enrollment_grant_id),
             cJSON_GetObjectItemCaseSensitive((cJSON *)data, "source_enrollment_grant_id"));
    copy_str(out->source_policy_snapshot_id,
             sizeof(out->source_policy_snapshot_id),
             cJSON_GetObjectItemCaseSensitive((cJSON *)data, "source_policy_snapshot_id"));
    copy_str(out->source_capability_envelope_id,
             sizeof(out->source_capability_envelope_id),
             cJSON_GetObjectItemCaseSensitive((cJSON *)data, "source_capability_envelope_id"));
    copy_str(out->policy_snapshot_version,
             sizeof(out->policy_snapshot_version),
             cJSON_GetObjectItemCaseSensitive((cJSON *)data, "policy_snapshot_version"));
    copy_str(out->distribution_target_ref,
             sizeof(out->distribution_target_ref),
             cJSON_GetObjectItemCaseSensitive((cJSON *)data, "distribution_target_ref"));
    copy_str(out->delegated_observation_scope,
             sizeof(out->delegated_observation_scope),
             cJSON_GetObjectItemCaseSensitive((cJSON *)data, "delegated_observation_scope"));
    copy_str(out->delegated_mediation_scope,
             sizeof(out->delegated_mediation_scope),
             cJSON_GetObjectItemCaseSensitive((cJSON *)data, "delegated_mediation_scope"));
    copy_str(out->delegated_enforcement_scope,
             sizeof(out->delegated_enforcement_scope),
             cJSON_GetObjectItemCaseSensitive((cJSON *)data, "delegated_enforcement_scope"));
}

void yai_sdk_source_mediation_state_init(yai_sdk_source_mediation_state_t *out)
{
    if (!out) {
        return;
    }
    memset(out, 0, sizeof(*out));
}

void yai_sdk_source_distribution_state_init(yai_sdk_source_distribution_state_t *out)
{
    if (!out) {
        return;
    }
    memset(out, 0, sizeof(*out));
}

void yai_sdk_source_enroll_reply_init(yai_sdk_source_enroll_reply_t *out)
{
    if (!out) {
        return;
    }
    memset(out, 0, sizeof(*out));
    yai_sdk_source_distribution_state_init(&out->distribution);
    yai_sdk_source_mediation_state_init(&out->mediation);
}

void yai_sdk_source_attach_reply_init(yai_sdk_source_attach_reply_t *out)
{
    if (!out) {
        return;
    }
    memset(out, 0, sizeof(*out));
    yai_sdk_source_distribution_state_init(&out->distribution);
    yai_sdk_source_mediation_state_init(&out->mediation);
}

void yai_sdk_source_emit_reply_init(yai_sdk_source_emit_reply_t *out)
{
    if (!out) {
        return;
    }
    memset(out, 0, sizeof(*out));
    yai_sdk_source_mediation_state_init(&out->mediation);
}

void yai_sdk_source_status_reply_init(yai_sdk_source_status_reply_t *out)
{
    if (!out) {
        return;
    }
    memset(out, 0, sizeof(*out));
    yai_sdk_source_distribution_state_init(&out->distribution);
    yai_sdk_source_mediation_state_init(&out->mediation);
}

void yai_sdk_source_summary_init(yai_sdk_source_summary_t *out)
{
    if (!out) {
        return;
    }
    memset(out, 0, sizeof(*out));
}

int yai_sdk_source_enroll(
    yai_sdk_client_t *client,
    const yai_sdk_source_enroll_request_t *req,
    yai_sdk_reply_t *out)
{
    cJSON *call = NULL;
    int rc;

    if (!client || !req || !out || !req->container_id || !req->container_id[0]) {
        return YAI_SDK_BAD_ARGS;
    }
    call = new_control_call(YAI_SDK_CMD_SOURCE_ENROLL, req->container_id);
    if (!call) {
        return YAI_SDK_IO;
    }
    if (req->source_label && req->source_label[0]) {
        cJSON_AddStringToObject(call, "source_label", req->source_label);
    }
    if (req->owner_ref && req->owner_ref[0]) {
        cJSON_AddStringToObject(call, "owner_ref", req->owner_ref);
    }
    if (req->source_node_id && req->source_node_id[0]) {
        cJSON_AddStringToObject(call, "source_node_id", req->source_node_id);
    }
    if (req->daemon_instance_id && req->daemon_instance_id[0]) {
        cJSON_AddStringToObject(call, "daemon_instance_id", req->daemon_instance_id);
    }
    rc = call_json_obj(client, call, out);
    cJSON_Delete(call);
    return rc;
}

int yai_sdk_source_attach(
    yai_sdk_client_t *client,
    const yai_sdk_source_attach_request_t *req,
    yai_sdk_reply_t *out)
{
    cJSON *call = NULL;
    int rc;

    if (!client || !req || !out || !req->container_id || !req->container_id[0] ||
        !req->source_node_id || !req->source_node_id[0]) {
        return YAI_SDK_BAD_ARGS;
    }
    call = new_control_call(YAI_SDK_CMD_SOURCE_ATTACH, req->container_id);
    if (!call) {
        return YAI_SDK_IO;
    }
    cJSON_AddStringToObject(call, "source_node_id", req->source_node_id);
    if (req->owner_container_id && req->owner_container_id[0]) {
        cJSON_AddStringToObject(call, "owner_container_id", req->owner_container_id);
    }
    if (req->binding_scope && req->binding_scope[0]) {
        cJSON_AddStringToObject(call, "binding_scope", req->binding_scope);
    }
    if (req->constraints_ref && req->constraints_ref[0]) {
        cJSON_AddStringToObject(call, "constraints_ref", req->constraints_ref);
    }
    rc = call_json_obj(client, call, out);
    cJSON_Delete(call);
    return rc;
}

int yai_sdk_source_emit(
    yai_sdk_client_t *client,
    const yai_sdk_source_emit_request_t *req,
    yai_sdk_reply_t *out)
{
    cJSON *call = NULL;
    size_t i;
    int rc;

    if (!client || !req || !out || !req->container_id || !req->container_id[0] ||
        !req->source_node_id || !req->source_node_id[0] ||
        !req->source_binding_id || !req->source_binding_id[0]) {
        return YAI_SDK_BAD_ARGS;
    }
    call = new_control_call(YAI_SDK_CMD_SOURCE_EMIT, req->container_id);
    if (!call) {
        return YAI_SDK_IO;
    }

    cJSON_AddStringToObject(call, "source_node_id", req->source_node_id);
    cJSON_AddStringToObject(call, "source_binding_id", req->source_binding_id);
    if (req->idempotency_key && req->idempotency_key[0]) {
        cJSON_AddStringToObject(call, "idempotency_key", req->idempotency_key);
    }

    if (req->assets && req->assets_len > 0) {
        cJSON *assets = cJSON_CreateArray();
        if (!assets) {
            cJSON_Delete(call);
            return YAI_SDK_IO;
        }
        for (i = 0; i < req->assets_len; ++i) {
            cJSON *it = cJSON_CreateObject();
            if (!it) {
                cJSON_Delete(call);
                return YAI_SDK_IO;
            }
            cJSON_AddStringToObject(it, "type", "yai.source_asset.v1");
            cJSON_AddStringToObject(it, "source_asset_id", req->assets[i].source_asset_id ? req->assets[i].source_asset_id : "");
            cJSON_AddStringToObject(it, "source_binding_id", req->assets[i].source_binding_id ? req->assets[i].source_binding_id : req->source_binding_id);
            cJSON_AddStringToObject(it, "locator", req->assets[i].locator ? req->assets[i].locator : "");
            cJSON_AddStringToObject(it, "asset_type", req->assets[i].asset_type ? req->assets[i].asset_type : "unknown");
            cJSON_AddStringToObject(it, "provenance_fingerprint", req->assets[i].provenance_fingerprint ? req->assets[i].provenance_fingerprint : "unset");
            cJSON_AddStringToObject(it, "observation_state", req->assets[i].observation_state ? req->assets[i].observation_state : "observed");
            cJSON_AddItemToArray(assets, it);
        }
        cJSON_AddItemToObject(call, "source_assets", assets);
    }

    if (req->events && req->events_len > 0) {
        cJSON *events = cJSON_CreateArray();
        if (!events) {
            cJSON_Delete(call);
            return YAI_SDK_IO;
        }
        for (i = 0; i < req->events_len; ++i) {
            cJSON *it = cJSON_CreateObject();
            if (!it) {
                cJSON_Delete(call);
                return YAI_SDK_IO;
            }
            cJSON_AddStringToObject(it, "type", "yai.source_acquisition_event.v1");
            cJSON_AddStringToObject(it, "source_acquisition_event_id", req->events[i].source_acquisition_event_id ? req->events[i].source_acquisition_event_id : "");
            cJSON_AddStringToObject(it, "source_node_id", req->events[i].source_node_id ? req->events[i].source_node_id : req->source_node_id);
            cJSON_AddStringToObject(it, "source_binding_id", req->events[i].source_binding_id ? req->events[i].source_binding_id : req->source_binding_id);
            cJSON_AddStringToObject(it, "source_asset_id", req->events[i].source_asset_id ? req->events[i].source_asset_id : "");
            cJSON_AddStringToObject(it, "event_type", req->events[i].event_type ? req->events[i].event_type : "discovered");
            cJSON_AddNumberToObject(it, "observed_at_epoch", (double)req->events[i].observed_at_epoch);
            cJSON_AddStringToObject(it, "idempotency_key", req->events[i].idempotency_key ? req->events[i].idempotency_key : (req->idempotency_key ? req->idempotency_key : "unset"));
            cJSON_AddStringToObject(it, "delivery_status", req->events[i].delivery_status ? req->events[i].delivery_status : "received");
            cJSON_AddItemToArray(events, it);
        }
        cJSON_AddItemToObject(call, "source_acquisition_events", events);
    }

    if (req->candidates && req->candidates_len > 0) {
        cJSON *candidates = cJSON_CreateArray();
        if (!candidates) {
            cJSON_Delete(call);
            return YAI_SDK_IO;
        }
        for (i = 0; i < req->candidates_len; ++i) {
            cJSON *it = cJSON_CreateObject();
            if (!it) {
                cJSON_Delete(call);
                return YAI_SDK_IO;
            }
            cJSON_AddStringToObject(it, "type", "yai.source_evidence_candidate.v1");
            cJSON_AddStringToObject(it, "source_evidence_candidate_id", req->candidates[i].source_evidence_candidate_id ? req->candidates[i].source_evidence_candidate_id : "");
            cJSON_AddStringToObject(it, "source_acquisition_event_id", req->candidates[i].source_acquisition_event_id ? req->candidates[i].source_acquisition_event_id : "");
            cJSON_AddStringToObject(it, "candidate_type", req->candidates[i].candidate_type ? req->candidates[i].candidate_type : "observation");
            cJSON_AddStringToObject(it, "derived_metadata_ref", req->candidates[i].derived_metadata_ref ? req->candidates[i].derived_metadata_ref : "meta://unset");
            cJSON_AddStringToObject(it, "owner_resolution_status", req->candidates[i].owner_resolution_status ? req->candidates[i].owner_resolution_status : "pending");
            cJSON_AddItemToArray(candidates, it);
        }
        cJSON_AddItemToObject(call, "source_evidence_candidates", candidates);
    }

    rc = call_json_obj(client, call, out);
    cJSON_Delete(call);
    return rc;
}

int yai_sdk_source_status(
    yai_sdk_client_t *client,
    const yai_sdk_source_status_request_t *req,
    yai_sdk_reply_t *out)
{
    cJSON *call = NULL;
    int rc;

    if (!client || !req || !out || !req->container_id || !req->container_id[0] ||
        !req->source_node_id || !req->source_node_id[0]) {
        return YAI_SDK_BAD_ARGS;
    }
    call = new_control_call(YAI_SDK_CMD_SOURCE_STATUS, req->container_id);
    if (!call) {
        return YAI_SDK_IO;
    }
    cJSON_AddStringToObject(call, "source_node_id", req->source_node_id);
    if (req->daemon_instance_id && req->daemon_instance_id[0]) {
        cJSON_AddStringToObject(call, "daemon_instance_id", req->daemon_instance_id);
    }
    if (req->health && req->health[0]) {
        cJSON_AddStringToObject(call, "health", req->health);
    }
    rc = call_json_obj(client, call, out);
    cJSON_Delete(call);
    return rc;
}

int yai_sdk_source_summary(yai_sdk_client_t *client, yai_sdk_reply_t *out)
{
    return yai_sdk_container_query_family(client, YAI_SDK_SOURCE_QUERY_FAMILY, out);
}

int yai_sdk_source_enroll_reply_from_sdk(
    const yai_sdk_reply_t *reply,
    yai_sdk_source_enroll_reply_t *out)
{
    cJSON *root = NULL;
    cJSON *data = NULL;
    int rc;

    if (!out) {
        return YAI_SDK_BAD_ARGS;
    }
    rc = parse_root_data(reply, &root, &data);
    if (rc != YAI_SDK_OK) {
        return rc;
    }
    yai_sdk_source_enroll_reply_init(out);
    copy_str(out->container_id, sizeof(out->container_id), cJSON_GetObjectItemCaseSensitive(data, "container_id"));
    copy_str(out->source_node_id, sizeof(out->source_node_id), cJSON_GetObjectItemCaseSensitive(data, "source_node_id"));
    copy_str(out->daemon_instance_id, sizeof(out->daemon_instance_id), cJSON_GetObjectItemCaseSensitive(data, "daemon_instance_id"));
    copy_str(out->owner_link_id, sizeof(out->owner_link_id), cJSON_GetObjectItemCaseSensitive(data, "owner_link_id"));
    out->registered = bool_field(data, "registered");
    parse_distribution(data, &out->distribution);
    parse_mediation(data, &out->mediation);
    cJSON_Delete(root);
    return YAI_SDK_OK;
}

int yai_sdk_source_attach_reply_from_sdk(
    const yai_sdk_reply_t *reply,
    yai_sdk_source_attach_reply_t *out)
{
    cJSON *root = NULL;
    cJSON *data = NULL;
    int rc;

    if (!out) {
        return YAI_SDK_BAD_ARGS;
    }
    rc = parse_root_data(reply, &root, &data);
    if (rc != YAI_SDK_OK) {
        return rc;
    }
    yai_sdk_source_attach_reply_init(out);
    copy_str(out->container_id, sizeof(out->container_id), cJSON_GetObjectItemCaseSensitive(data, "container_id"));
    copy_str(out->owner_container_id, sizeof(out->owner_container_id), cJSON_GetObjectItemCaseSensitive(data, "owner_container_id"));
    copy_str(out->source_node_id, sizeof(out->source_node_id), cJSON_GetObjectItemCaseSensitive(data, "source_node_id"));
    copy_str(out->source_binding_id, sizeof(out->source_binding_id), cJSON_GetObjectItemCaseSensitive(data, "source_binding_id"));
    copy_str(out->attachment_status, sizeof(out->attachment_status), cJSON_GetObjectItemCaseSensitive(data, "attachment_status"));
    parse_distribution(data, &out->distribution);
    parse_mediation(data, &out->mediation);
    cJSON_Delete(root);
    return YAI_SDK_OK;
}

int yai_sdk_source_emit_reply_from_sdk(
    const yai_sdk_reply_t *reply,
    yai_sdk_source_emit_reply_t *out)
{
    cJSON *root = NULL;
    cJSON *data = NULL;
    const cJSON *accepted = NULL;
    int rc;

    if (!out) {
        return YAI_SDK_BAD_ARGS;
    }
    rc = parse_root_data(reply, &root, &data);
    if (rc != YAI_SDK_OK) {
        return rc;
    }
    yai_sdk_source_emit_reply_init(out);
    copy_str(out->container_id, sizeof(out->container_id), cJSON_GetObjectItemCaseSensitive(data, "container_id"));
    copy_str(out->source_node_id, sizeof(out->source_node_id), cJSON_GetObjectItemCaseSensitive(data, "source_node_id"));
    copy_str(out->source_binding_id, sizeof(out->source_binding_id), cJSON_GetObjectItemCaseSensitive(data, "source_binding_id"));
    copy_str(out->idempotency_key, sizeof(out->idempotency_key), cJSON_GetObjectItemCaseSensitive(data, "idempotency_key"));
    accepted = cJSON_GetObjectItemCaseSensitive(data, "accepted");
    if (cJSON_IsObject(accepted)) {
        out->accepted_assets = int_field(accepted, "source_asset");
        out->accepted_events = int_field(accepted, "source_acquisition_event");
        out->accepted_candidates = int_field(accepted, "source_evidence_candidate");
    }
    parse_mediation(data, &out->mediation);
    cJSON_Delete(root);
    return YAI_SDK_OK;
}

int yai_sdk_source_status_reply_from_sdk(
    const yai_sdk_reply_t *reply,
    yai_sdk_source_status_reply_t *out)
{
    cJSON *root = NULL;
    cJSON *data = NULL;
    int rc;

    if (!out) {
        return YAI_SDK_BAD_ARGS;
    }
    rc = parse_root_data(reply, &root, &data);
    if (rc != YAI_SDK_OK) {
        return rc;
    }
    yai_sdk_source_status_reply_init(out);
    copy_str(out->container_id, sizeof(out->container_id), cJSON_GetObjectItemCaseSensitive(data, "container_id"));
    copy_str(out->source_node_id, sizeof(out->source_node_id), cJSON_GetObjectItemCaseSensitive(data, "source_node_id"));
    copy_str(out->daemon_instance_id, sizeof(out->daemon_instance_id), cJSON_GetObjectItemCaseSensitive(data, "daemon_instance_id"));
    copy_str(out->health, sizeof(out->health), cJSON_GetObjectItemCaseSensitive(data, "health"));
    parse_distribution(data, &out->distribution);
    parse_mediation(data, &out->mediation);
    cJSON_Delete(root);
    return YAI_SDK_OK;
}

int yai_sdk_source_summary_from_sdk(
    const yai_sdk_reply_t *reply,
    yai_sdk_source_summary_t *out)
{
    cJSON *root = NULL;
    cJSON *data = NULL;
    const cJSON *summary = NULL;
    int rc;

    if (!out) {
        return YAI_SDK_BAD_ARGS;
    }
    rc = parse_root_data(reply, &root, &data);
    if (rc != YAI_SDK_OK) {
        return rc;
    }
    yai_sdk_source_summary_init(out);
    summary = cJSON_GetObjectItemCaseSensitive(data, "summary");
    if (!cJSON_IsObject(summary)) {
        cJSON_Delete(root);
        return YAI_SDK_PROTOCOL;
    }
    out->source_node_count = int_field(summary, "source_node_count");
    out->source_daemon_instance_count = int_field(summary, "source_daemon_instance_count");
    out->source_binding_count = int_field(summary, "source_binding_count");
    out->source_asset_count = int_field(summary, "source_asset_count");
    out->source_acquisition_event_count = int_field(summary, "source_acquisition_event_count");
    out->source_evidence_candidate_count = int_field(summary, "source_evidence_candidate_count");
    out->source_owner_link_count = int_field(summary, "source_owner_link_count");
    out->source_graph_node_count = int_field(summary, "source_graph_node_count");
    out->source_graph_edge_count = int_field(summary, "source_graph_edge_count");
    cJSON_Delete(root);
    return YAI_SDK_OK;
}
