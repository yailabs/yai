/* SPDX-License-Identifier: Apache-2.0 */

#include <yai/sdk/models.h>
#include <yai/sdk/targets.h>

#include <cJSON.h>

#include <stdio.h>
#include <string.h>

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

static yai_sdk_availability_state_t avail_from_ready_degraded(int ready, int degraded)
{
    if (degraded) {
        return YAI_SDK_AVAIL_DEGRADED;
    }
    if (ready) {
        return YAI_SDK_AVAIL_AVAILABLE;
    }
    return YAI_SDK_AVAIL_UNKNOWN;
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

static int streq(const char *a, const char *b)
{
    return a && b && strcmp(a, b) == 0;
}

void yai_sdk_runtime_state_init(yai_sdk_runtime_state_t *out)
{
    if (!out) {
        return;
    }
    memset(out, 0, sizeof(*out));
    out->liveness = YAI_SDK_LIVENESS_UNKNOWN;
    out->container_binding = YAI_SDK_BINDING_UNKNOWN;
    out->recovery = YAI_SDK_RECOVERY_UNKNOWN;
    out->exec.availability = YAI_SDK_AVAIL_UNKNOWN;
    out->data.availability = YAI_SDK_AVAIL_UNKNOWN;
    out->graph.availability = YAI_SDK_AVAIL_UNKNOWN;
    out->cognition.availability = YAI_SDK_AVAIL_UNKNOWN;
}

static void parse_container_binding(yai_sdk_runtime_state_t *out, const cJSON *data)
{
    const cJSON *binding = NULL;
    const cJSON *ws_id = NULL;
    const cJSON *ws_alias = NULL;

    if (!out || !data) {
        return;
    }

    binding = cJSON_GetObjectItemCaseSensitive((cJSON *)data, "binding_status");
    ws_id = cJSON_GetObjectItemCaseSensitive((cJSON *)data, "container_id");
    if (!cJSON_IsString(ws_id)) {
        ws_id = cJSON_GetObjectItemCaseSensitive((cJSON *)data, "workspace_id");
    }
    if (!cJSON_IsString(ws_id)) {
        ws_id = cJSON_GetObjectItemCaseSensitive((cJSON *)data, "ws_id");
    }
    ws_alias = cJSON_GetObjectItemCaseSensitive((cJSON *)data, "container_alias");
    if (!cJSON_IsString(ws_alias)) {
        ws_alias = cJSON_GetObjectItemCaseSensitive((cJSON *)data, "workspace_alias");
    }

    copy_str(out->container_id, sizeof(out->container_id), ws_id);
    copy_str(out->container_alias, sizeof(out->container_alias), ws_alias);
    out->has_active_container = out->container_id[0] ? 1 : 0;

    if (cJSON_IsString(binding) && binding->valuestring) {
        if (streq(binding->valuestring, "active")) {
            out->container_binding = YAI_SDK_BINDING_ACTIVE;
        } else if (streq(binding->valuestring, "selected")) {
            out->container_binding = YAI_SDK_BINDING_SELECTED;
        } else if (streq(binding->valuestring, "degraded")) {
            out->container_binding = YAI_SDK_BINDING_DEGRADED;
        } else if (streq(binding->valuestring, "none") || streq(binding->valuestring, "detached")) {
            out->container_binding = YAI_SDK_BINDING_NONE;
        } else if (streq(binding->valuestring, "unbound")) {
            out->container_binding = YAI_SDK_BINDING_UNBOUND;
        }
    } else if (out->has_active_container) {
        out->container_binding = YAI_SDK_BINDING_SELECTED;
    }
}

static void parse_recovery(yai_sdk_runtime_state_t *out, const cJSON *data)
{
    const cJSON *recovery = NULL;
    if (!out || !data) {
        return;
    }
    recovery = cJSON_GetObjectItemCaseSensitive((cJSON *)data, "recovery_state");
    if (!cJSON_IsString(recovery) || !recovery->valuestring) {
        return;
    }
    if (streq(recovery->valuestring, "none") || streq(recovery->valuestring, "unknown")) {
        out->recovery = YAI_SDK_RECOVERY_NONE;
    } else if (streq(recovery->valuestring, "fresh")) {
        out->recovery = YAI_SDK_RECOVERY_FRESH;
    } else if (streq(recovery->valuestring, "restored")) {
        out->recovery = YAI_SDK_RECOVERY_RESTORED;
    } else if (streq(recovery->valuestring, "recovered")) {
        out->recovery = YAI_SDK_RECOVERY_RECOVERED;
    } else if (streq(recovery->valuestring, "failed")) {
        out->recovery = YAI_SDK_RECOVERY_FAILED;
    }
}

static void set_families_from_command(yai_sdk_runtime_state_t *out, const cJSON *data)
{
    const cJSON *qf = NULL;
    if (!out) {
        return;
    }

    if (strstr(out->command_id, "container.run") != NULL) {
        out->exec.ready = 1;
        out->exec.bound = out->has_active_container;
    }

    if (strstr(out->command_id, "container.graph.summary") != NULL) {
        out->graph.ready = 1;
        out->graph.bound = out->has_active_container;
        out->cognition.bound = out->has_active_container;
        if (bool_field(data, "graph_truth_authoritative")) {
            out->graph.ready = 1;
        }
        if (cJSON_IsObject(data)) {
            const cJSON *summary = cJSON_GetObjectItemCaseSensitive((cJSON *)data, "summary");
            if (cJSON_IsObject(summary)) {
                if (bool_field(summary, "graph_truth_authoritative")) {
                    out->graph.ready = 1;
                }
                if (cJSON_GetObjectItemCaseSensitive((cJSON *)summary, "last_transient_state_ref")) {
                    out->cognition.ready = 1;
                }
            }
        }
    }

    qf = cJSON_GetObjectItemCaseSensitive((cJSON *)data, "query_family");
    if (cJSON_IsString(qf) && qf->valuestring) {
        if (streq(qf->valuestring, "graph")) {
            out->graph.ready = 1;
            out->graph.bound = out->has_active_container;
        } else if (streq(qf->valuestring, "enforcement") ||
                   streq(qf->valuestring, "authority") ||
                   streq(qf->valuestring, "evidence") ||
                   streq(qf->valuestring, "artifact") ||
                   streq(qf->valuestring, "governance")) {
            out->data.ready = 1;
            out->data.bound = out->has_active_container;
        } else if (streq(qf->valuestring, "memory") || streq(qf->valuestring, "transient")) {
            out->cognition.ready = 1;
            out->cognition.bound = out->has_active_container;
        }
    }

    if (cJSON_IsObject(data)) {
        const cJSON *execution = cJSON_GetObjectItemCaseSensitive((cJSON *)data, "execution");
        if (cJSON_IsObject(execution) && bool_field(execution, "degraded")) {
            out->exec.degraded = 1;
        }
        if (bool_field(data, "execution_mode_degraded")) {
            out->exec.degraded = 1;
        }
    }

    out->exec.availability = avail_from_ready_degraded(out->exec.ready, out->exec.degraded);
    out->data.availability = avail_from_ready_degraded(out->data.ready, out->data.degraded);
    out->graph.availability = avail_from_ready_degraded(out->graph.ready, out->graph.degraded);
    out->cognition.availability = avail_from_ready_degraded(out->cognition.ready, out->cognition.degraded);
}

int yai_sdk_runtime_state_from_reply_json(const char *exec_reply_json, yai_sdk_runtime_state_t *out)
{
    cJSON *root = NULL;
    cJSON *data = NULL;

    if (!exec_reply_json || !out) {
        return -1;
    }

    yai_sdk_runtime_state_init(out);

    root = cJSON_Parse(exec_reply_json);
    if (!root) {
        return -1;
    }

    copy_str(out->status, sizeof(out->status), cJSON_GetObjectItemCaseSensitive(root, "status"));
    copy_str(out->code, sizeof(out->code), cJSON_GetObjectItemCaseSensitive(root, "code"));
    copy_str(out->reason, sizeof(out->reason), cJSON_GetObjectItemCaseSensitive(root, "reason"));
    copy_str(out->command_id, sizeof(out->command_id), cJSON_GetObjectItemCaseSensitive(root, "command_id"));
    copy_str(out->target_plane, sizeof(out->target_plane), cJSON_GetObjectItemCaseSensitive(root, "target_plane"));

    if (streq(out->status, "ok") || streq(out->status, "warn")) {
        out->liveness = YAI_SDK_LIVENESS_UP;
    } else if (streq(out->code, "SERVER_UNAVAILABLE") || streq(out->code, "RUNTIME_NOT_READY")) {
        out->liveness = YAI_SDK_LIVENESS_DOWN;
    }

    data = cJSON_GetObjectItemCaseSensitive(root, "data");
    if (cJSON_IsObject(data)) {
        parse_container_binding(out, data);
        parse_recovery(out, data);
        set_families_from_command(out, data);
    }

    if (out->container_binding == YAI_SDK_BINDING_UNKNOWN) {
        if (out->has_active_container) {
            out->container_binding = YAI_SDK_BINDING_SELECTED;
        } else {
            out->container_binding = YAI_SDK_BINDING_NONE;
        }
    }

    cJSON_Delete(root);
    return 0;
}

void yai_sdk_governance_state_init(yai_sdk_governance_state_t *out)
{
    if (!out) {
        return;
    }
    memset(out, 0, sizeof(*out));
    out->attachable = -1;
    out->blocked = -1;
}

int yai_sdk_governance_state_from_reply_json(const char *exec_reply_json, yai_sdk_governance_state_t *out)
{
    cJSON *root = NULL;
    const cJSON *data = NULL;

    if (!exec_reply_json || !out) {
        return -1;
    }

    yai_sdk_governance_state_init(out);

    root = cJSON_Parse(exec_reply_json);
    if (!root) {
        return -1;
    }

    data = cJSON_GetObjectItemCaseSensitive(root, "data");
    if (cJSON_IsObject(data)) {
        const cJSON *decision = cJSON_GetObjectItemCaseSensitive((cJSON *)data, "decision");
        const cJSON *enforcement = cJSON_GetObjectItemCaseSensitive((cJSON *)data, "enforcement");
        if (cJSON_IsObject(decision)) {
            copy_str(out->effect, sizeof(out->effect), cJSON_GetObjectItemCaseSensitive((cJSON *)decision, "effect"));
        }
        if (cJSON_IsObject(enforcement)) {
            copy_str(out->authority_decision, sizeof(out->authority_decision),
                     cJSON_GetObjectItemCaseSensitive((cJSON *)enforcement, "authority_decision"));
            copy_str(out->review_state, sizeof(out->review_state),
                     cJSON_GetObjectItemCaseSensitive((cJSON *)enforcement, "review_state"));
        }
        if (cJSON_GetObjectItemCaseSensitive((cJSON *)data, "attachability")) {
            const cJSON *a = cJSON_GetObjectItemCaseSensitive((cJSON *)data, "attachability");
            if (cJSON_IsObject(a)) {
                out->attachable = bool_field(a, "attachable");
                out->blocked = bool_field(a, "blocked");
            }
        }
    }

    if (out->blocked < 0) {
        if (streq(out->effect, "deny")) {
            out->blocked = 1;
        } else if (out->effect[0]) {
            out->blocked = 0;
        }
    }

    cJSON_Delete(root);
    return 0;
}

int yai_sdk_control_call_to_json(
    const yai_sdk_control_call_t *call,
    char *out_json,
    size_t out_cap)
{
    cJSON *root = NULL;
    cJSON *argv = NULL;
    char *tmp = NULL;
    int rc = -1;
    size_t n = 0;

    if (!call || !out_json || out_cap == 0 || !call->command_id || !call->command_id[0]) {
        return -1;
    }

    root = cJSON_CreateObject();
    if (!root) {
        return -1;
    }

    cJSON_AddStringToObject(root, "type", "yai.control.call.v1");
    cJSON_AddStringToObject(root, "target_plane",
                            (call->target_plane && call->target_plane[0]) ? call->target_plane : "runtime");
    cJSON_AddStringToObject(root, "command_id", call->command_id);

    argv = cJSON_AddArrayToObject(root, "argv");
    if (!argv) {
        cJSON_Delete(root);
        return -1;
    }

    if (call->argv && call->argv_len > 0) {
        size_t i;
        for (i = 0; i < call->argv_len; ++i) {
            const char *arg = call->argv[i] ? call->argv[i] : "";
            cJSON_AddItemToArray(argv, cJSON_CreateString(arg));
        }
    }

    tmp = cJSON_PrintUnformatted(root);
    if (!tmp) {
        cJSON_Delete(root);
        return -1;
    }

    n = strlen(tmp);
    if (n + 1 > out_cap) {
        rc = -1;
    } else {
        memcpy(out_json, tmp, n + 1);
        rc = 0;
    }

    cJSON_free(tmp);
    cJSON_Delete(root);
    return rc;
}

void yai_sdk_runtime_target_init(yai_sdk_runtime_target_t *out)
{
    if (!out) {
        return;
    }
    memset(out, 0, sizeof(*out));
    out->target_kind = YAI_SDK_RUNTIME_TARGET_UNKNOWN;
    out->mesh_plane = YAI_SDK_MESH_PLANE_UNKNOWN;
}

void yai_sdk_policy_distribution_descriptor_init(yai_sdk_policy_distribution_descriptor_t *out)
{
    if (!out) {
        return;
    }
    memset(out, 0, sizeof(*out));
}

void yai_sdk_remote_association_state_init(yai_sdk_remote_association_state_t *out)
{
    if (!out) {
        return;
    }
    memset(out, 0, sizeof(*out));
}

void yai_sdk_operational_summary_init(yai_sdk_operational_summary_t *out)
{
    if (!out) {
        return;
    }
    memset(out, 0, sizeof(*out));
}

static int read_count_value(const cJSON *obj, const char *key)
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

int yai_sdk_operational_summary_from_reply_json(const char *exec_reply_json,
                                                yai_sdk_operational_summary_t *out)
{
    cJSON *root = NULL;
    const cJSON *data = NULL;
    const cJSON *op = NULL;
    const cJSON *section = NULL;

    if (!exec_reply_json || !out) {
        return -1;
    }

    yai_sdk_operational_summary_init(out);

    root = cJSON_Parse(exec_reply_json);
    if (!root) {
        return -1;
    }

    data = cJSON_GetObjectItemCaseSensitive(root, "data");
    if (!cJSON_IsObject(data)) {
        cJSON_Delete(root);
        return -1;
    }

    op = cJSON_GetObjectItemCaseSensitive((cJSON *)data, "operational_summary");
    if (!cJSON_IsObject(op)) {
        op = data;
    }

    section = cJSON_GetObjectItemCaseSensitive((cJSON *)op, "source_edge_summary");
    if (cJSON_IsObject(section)) {
        out->source_node_count = read_count_value(section, "source_node");
        out->source_daemon_instance_count = read_count_value(section, "source_daemon_instance");
        out->source_binding_count = read_count_value(section, "source_binding");
    }

    section = cJSON_GetObjectItemCaseSensitive((cJSON *)op, "delegation_summary");
    if (cJSON_IsObject(section)) {
        out->source_policy_snapshot_count = read_count_value(section, "source_policy_snapshot");
        out->source_enrollment_grant_count = read_count_value(section, "source_enrollment_grant");
        out->source_capability_envelope_count = read_count_value(section, "source_capability_envelope");
    }

    section = cJSON_GetObjectItemCaseSensitive((cJSON *)op, "mesh_summary");
    if (cJSON_IsObject(section)) {
        out->mesh_coordination_membership_count = read_count_value(section, "mesh_coordination_membership");
        out->mesh_peer_awareness_count = read_count_value(section, "mesh_peer_awareness");
        out->mesh_peer_legitimacy_count = read_count_value(section, "mesh_peer_legitimacy");
        out->mesh_authority_scope_count = read_count_value(section, "mesh_authority_scope");
    }

    section = cJSON_GetObjectItemCaseSensitive((cJSON *)op, "transport_ingress_overlay_summary");
    if (cJSON_IsObject(section)) {
        out->mesh_transport_endpoint_count = read_count_value(section, "mesh_transport_endpoint");
        out->mesh_transport_path_state_count = read_count_value(section, "mesh_transport_path_state");
        out->mesh_owner_remote_ingress_count = read_count_value(section, "mesh_owner_remote_ingress");
        out->mesh_owner_remote_ingress_decision_count = read_count_value(section, "mesh_owner_remote_ingress_decision");
        out->mesh_overlay_presence_count = read_count_value(section, "mesh_overlay_presence");
        out->mesh_overlay_target_association_count = read_count_value(section, "mesh_overlay_target_association");
        out->mesh_overlay_path_mutation_count = read_count_value(section, "mesh_overlay_path_mutation");
    }

    out->source_action_point_count = read_count_value((cJSON *)op, "source_action_point");
    out->source_acquisition_event_count = read_count_value((cJSON *)op, "source_acquisition_event");
    out->source_ingest_outcome_count = read_count_value((cJSON *)op, "source_ingest_outcome");
    out->source_evidence_candidate_count = read_count_value((cJSON *)op, "source_evidence_candidate");

    cJSON_Delete(root);
    return 0;
}
