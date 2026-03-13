/* SPDX-License-Identifier: Apache-2.0 */

#include <yai/sdk/client.h>
#include <yai/sdk/errors.h>
#include <yai/sdk/transport.h>

#include "client_internal.h"
#include "../protocol/reply_map.h"
#include "../platform/log_internal.h"

#include <cJSON.h>
#include <yai/protocol/control/ids.h>

#include <stdlib.h>
#include <string.h>
#include <stdio.h>

static void set_client_uds_path(yai_sdk_client_t *c, const yai_sdk_client_opts_t *opts)
{
    if (!c) {
        return;
    }
    c->uds_path[0] = '\0';
    if (opts && opts->uds_path && opts->uds_path[0]) {
        snprintf(c->uds_path, sizeof(c->uds_path), "%s", opts->uds_path);
    }
}

static int set_client_locator(yai_sdk_client_t *c, const yai_sdk_client_opts_t *opts)
{
    yai_sdk_runtime_endpoint_t endpoint;
    char resolve_err[128];
    int rc = 0;

    if (!c) {
        return YAI_SDK_BAD_ARGS;
    }

    yai_sdk_runtime_locator_state_init(&c->locator_state);
    yai_sdk_runtime_endpoint_init(&endpoint);
    c->connect_timeout_ms = 0;
    c->connect_attempts = 1;

    if (opts) {
        c->connect_timeout_ms = opts->connect_timeout_ms;
        c->connect_attempts = (opts->connect_attempts > 0) ? opts->connect_attempts : 1;
    }

    if (opts && opts->runtime_endpoint) {
        endpoint = *opts->runtime_endpoint;
    } else if (opts && opts->owner_endpoint_ref && opts->owner_endpoint_ref[0]) {
        if (yai_sdk_runtime_endpoint_owner_ref(opts->owner_endpoint_ref, &endpoint) != 0) {
            return YAI_SDK_ENDPOINT_INVALID;
        }
    } else if (opts && opts->uds_path && opts->uds_path[0]) {
        if (yai_sdk_runtime_endpoint_local_uds(opts->uds_path, &endpoint) != 0) {
            return YAI_SDK_ENDPOINT_INVALID;
        }
    } else {
        (void)yai_sdk_runtime_endpoint_local_default(&endpoint);
    }

    resolve_err[0] = '\0';
    rc = yai_sdk_runtime_endpoint_resolve(&endpoint, &c->locator_state, resolve_err, sizeof(resolve_err));
    if (rc == -2) {
        return YAI_SDK_ENDPOINT_UNRESOLVED;
    }
    if (rc == -3) {
        return YAI_SDK_TRANSPORT_UNSUPPORTED;
    }
    if (rc != 0) {
        return YAI_SDK_ENDPOINT_INVALID;
    }

    if (snprintf(c->uds_path, sizeof(c->uds_path), "%s", c->locator_state.resolved_ingress) <= 0) {
        return YAI_SDK_ENDPOINT_UNRESOLVED;
    }

    return YAI_SDK_OK;
}

static const char *summary_for_code(const char *code)
{
    if (!code || !code[0]) return "Command failed.";
    if (strcmp(code, "OK") == 0) return "Command completed.";
    if (strcmp(code, "NOT_IMPLEMENTED") == 0) return "This command is registered but not implemented yet.";
    if (strcmp(code, "BAD_ARGS") == 0) return "Invalid command arguments.";
    if (strcmp(code, "SERVER_UNAVAILABLE") == 0) return "Runtime endpoint is unreachable.";
    if (strcmp(code, "RUNTIME_NOT_READY") == 0) return "Runtime handshake is not ready.";
    if (strcmp(code, "UNAUTHORIZED") == 0 || strcmp(code, "DENIED") == 0) return "Operation denied by authority policy.";
    return "Command failed.";
}

static void reply_zero(yai_sdk_reply_t *r)
{
    if (!r) {
        return;
    }
    memset(r, 0, sizeof(*r));
    snprintf(r->status, sizeof(r->status), "error");
    snprintf(r->code, sizeof(r->code), "INTERNAL_ERROR");
    snprintf(r->reason, sizeof(r->reason), "internal_error");
    snprintf(r->summary, sizeof(r->summary), "Command failed.");
    snprintf(r->command_id, sizeof(r->command_id), "yai.unknown.unknown");
    snprintf(r->target_plane, sizeof(r->target_plane), "runtime");
}

static void reply_set(yai_sdk_reply_t *r,
                      const char *status,
                      const char *code,
                      const char *reason,
                      const char *summary,
                      const char *command_id,
                      const char *trace_id,
                      const char *target_plane)
{
    if (!r) {
        return;
    }
    snprintf(r->status, sizeof(r->status), "%s", (status && status[0]) ? status : "error");
    snprintf(r->code, sizeof(r->code), "%s", (code && code[0]) ? code : "INTERNAL_ERROR");
    snprintf(r->reason, sizeof(r->reason), "%s", (reason && reason[0]) ? reason : "internal_error");
    snprintf(r->summary, sizeof(r->summary), "%s",
             (summary && summary[0]) ? summary : summary_for_code(code));
    r->hint_count = 0;
    r->hints[0][0] = '\0';
    r->hints[1][0] = '\0';
    r->details[0] = '\0';
    snprintf(r->command_id, sizeof(r->command_id), "%s",
             (command_id && command_id[0]) ? command_id : "yai.unknown.unknown");
    snprintf(r->trace_id, sizeof(r->trace_id), "%s", (trace_id && trace_id[0]) ? trace_id : "");
    snprintf(r->target_plane, sizeof(r->target_plane), "%s",
             (target_plane && target_plane[0]) ? target_plane : "runtime");
}

static char *dup_bytes(const char *s, size_t n)
{
    char *out = (char *)malloc(n + 1);
    if (!out) {
        return NULL;
    }
    if (n) {
        memcpy(out, s, n);
    }
    out[n] = '\0';
    return out;
}

static void parse_command_id_from_request(const char *control_call_json, char *out, size_t out_sz)
{
    if (!out || out_sz == 0) {
        return;
    }
    snprintf(out, out_sz, "yai.unknown.unknown");
    if (!control_call_json || !control_call_json[0]) {
        return;
    }

    cJSON *req = cJSON_Parse(control_call_json);
    if (!req) {
        return;
    }
    const cJSON *command_id = cJSON_GetObjectItemCaseSensitive(req, "command_id");
    if (cJSON_IsString(command_id) && command_id->valuestring && command_id->valuestring[0]) {
        snprintf(out, out_sz, "%s", command_id->valuestring);
    }
    cJSON_Delete(req);
}

int yai_sdk_client_open(yai_sdk_client_t **out, const yai_sdk_client_opts_t *opts)
{
    if (!out) {
        return YAI_SDK_BAD_ARGS;
    }
    *out = NULL;

    yai_sdk_client_t *c = (yai_sdk_client_t *)calloc(1, sizeof(*c));
    if (!c) {
        return YAI_SDK_IO;
    }
    c->rpc.fd = -1;
    snprintf(c->scope_id, sizeof(c->scope_id), "%s",
             (opts && opts->container_id && opts->container_id[0]) ? opts->container_id : "default");
    snprintf(c->role, sizeof(c->role), "%s",
             (opts && opts->role && opts->role[0]) ? opts->role : "operator");
    snprintf(c->correlation_id, sizeof(c->correlation_id), "%s",
             (opts && opts->correlation_id && opts->correlation_id[0]) ? opts->correlation_id : "sdk");
    c->arming = (opts) ? (opts->arming ? 1 : 0) : 1;
    c->auto_handshake = (opts) ? (opts->auto_handshake ? 1 : 0) : 1;
    set_client_uds_path(c, opts);
    {
        int lrc = set_client_locator(c, opts);
        if (lrc != YAI_SDK_OK) {
            free(c);
            return lrc;
        }
    }

    if (yai_rpc_connect_at(&c->rpc, c->scope_id, c->uds_path) != 0) {
        yai_sdk_log_emit(YAI_SDK_LOG_ERROR, "client", "rpc connect failed");
        free(c);
        return YAI_SDK_SERVER_OFF;
    }
    c->is_open = 1;
    yai_rpc_set_authority(&c->rpc, c->arming, c->role);
    yai_rpc_set_correlation_id(&c->rpc, c->correlation_id);
    yai_sdk_log_emit(YAI_SDK_LOG_INFO, "client", "client opened");
    *out = c;
    return YAI_SDK_OK;
}

void yai_sdk_client_close(yai_sdk_client_t *c)
{
    if (!c) {
        return;
    }
    if (c->is_open) {
        yai_rpc_close(&c->rpc);
    }
    free(c);
}

int yai_sdk_client_set_authority(yai_sdk_client_t *c, int arming, const char *role)
{
    if (!c) {
        return YAI_SDK_BAD_ARGS;
    }
    c->arming = arming ? 1 : 0;
    snprintf(c->role, sizeof(c->role), "%s", (role && role[0]) ? role : "operator");
    yai_rpc_set_authority(&c->rpc, c->arming, c->role);
    return YAI_SDK_OK;
}

int yai_sdk_client_set_container(yai_sdk_client_t *c, const char *container_id)
{
    if (!c || !container_id || !container_id[0]) {
        return YAI_SDK_BAD_ARGS;
    }
    yai_rpc_close(&c->rpc);
    c->is_open = 0;
    c->handshaken = 0;
    snprintf(c->scope_id, sizeof(c->scope_id), "%s", container_id);
    if (yai_rpc_connect_at(&c->rpc, c->scope_id, c->uds_path) != 0) {
        yai_sdk_log_emit(YAI_SDK_LOG_ERROR, "client", "rpc reconnect failed");
        return YAI_SDK_SERVER_OFF;
    }
    c->is_open = 1;
    yai_rpc_set_authority(&c->rpc, c->arming, c->role);
    yai_rpc_set_correlation_id(&c->rpc, c->correlation_id);
    return YAI_SDK_OK;
}

int yai_sdk_client_set_correlation_id(yai_sdk_client_t *c, const char *correlation_id)
{
    if (!c || !correlation_id || !correlation_id[0]) {
        return YAI_SDK_BAD_ARGS;
    }
    snprintf(c->correlation_id, sizeof(c->correlation_id), "%s", correlation_id);
    yai_rpc_set_correlation_id(&c->rpc, c->correlation_id);
    return YAI_SDK_OK;
}

int yai_sdk_client_handshake(yai_sdk_client_t *c)
{
    if (!c || !c->is_open) {
        return YAI_SDK_BAD_ARGS;
    }
    if (yai_rpc_handshake(&c->rpc) != 0) {
        c->handshaken = 0;
        return YAI_SDK_RUNTIME_NOT_READY;
    }
    c->handshaken = 1;
    return YAI_SDK_OK;
}

int yai_sdk_client_runtime_locator_state(
    const yai_sdk_client_t *c,
    yai_sdk_runtime_locator_state_t *out)
{
    if (!c || !out) {
        return YAI_SDK_BAD_ARGS;
    }
    *out = c->locator_state;
    return YAI_SDK_OK;
}

int yai_sdk_client_call_json(yai_sdk_client_t *c, const char *control_call_json, yai_sdk_reply_t *out)
{
    char fallback_command_id[128];
    if (!c || !control_call_json || !control_call_json[0] || !out) {
        return YAI_SDK_BAD_ARGS;
    }

    reply_zero(out);
    parse_command_id_from_request(control_call_json, fallback_command_id, sizeof(fallback_command_id));

    if (c->auto_handshake && !c->handshaken) {
        int hrc = yai_sdk_client_handshake(c);
        if (hrc != 0) {
            yai_sdk_log_emit(YAI_SDK_LOG_WARN, "client", "handshake not ready");
            reply_set(out, "error", "RUNTIME_NOT_READY", "runtime_not_ready",
                      "Runtime handshake is not ready.",
                      fallback_command_id, "", "runtime");
            return hrc;
        }
    }

    /* Workspace inspect/policy/debug replies can be significantly larger than 4KB. */
    char raw[262144];
    uint32_t out_len = 0;
    int rc = yai_rpc_call_raw(
        &c->rpc,
        YAI_CMD_CONTROL_CALL,
        control_call_json,
        (uint32_t)strlen(control_call_json),
        raw,
        sizeof(raw) - 1,
        &out_len);

    if (rc != 0) {
        if (rc == -5) {
            yai_sdk_log_emit(YAI_SDK_LOG_ERROR, "client", "server unavailable");
            reply_set(out, "error", "SERVER_UNAVAILABLE", "server_unavailable",
                      "Runtime endpoint is unreachable.",
                      fallback_command_id, "", "runtime");
            return YAI_SDK_SERVER_OFF;
        }
        yai_sdk_log_emit(YAI_SDK_LOG_ERROR, "client", "rpc call failed");
        reply_set(out, "error", "PROTOCOL_ERROR", "rpc_call_failed",
                  "RPC request failed.",
                  fallback_command_id, "", "runtime");
        return YAI_SDK_RPC;
    }

    if (out_len >= sizeof(raw)) {
        out_len = (uint32_t)(sizeof(raw) - 1);
    }
    raw[out_len] = '\0';

    out->exec_reply_json = dup_bytes(raw, out_len);
    if (!out->exec_reply_json) {
        reply_set(out, "error", "INTERNAL_ERROR", "alloc_failed",
                  "Reply allocation failed.",
                  fallback_command_id, "", "runtime");
        return YAI_SDK_IO;
    }

    cJSON *resp = cJSON_Parse(out->exec_reply_json);
    if (!resp) {
        yai_sdk_log_emit(YAI_SDK_LOG_ERROR, "client", "response parse failed");
        reply_set(out, "error", "PROTOCOL_ERROR", "response_parse_failed",
                  "Reply parsing failed.",
                  fallback_command_id, "", "runtime");
        return YAI_SDK_PROTOCOL;
    }

    const cJSON *type = cJSON_GetObjectItemCaseSensitive(resp, "type");
    const cJSON *status = cJSON_GetObjectItemCaseSensitive(resp, "status");
    const cJSON *code = cJSON_GetObjectItemCaseSensitive(resp, "code");
    const cJSON *reason = cJSON_GetObjectItemCaseSensitive(resp, "reason");
    const cJSON *command_id = cJSON_GetObjectItemCaseSensitive(resp, "command_id");
    const cJSON *trace_id = cJSON_GetObjectItemCaseSensitive(resp, "trace_id");
    const cJSON *target_plane = cJSON_GetObjectItemCaseSensitive(resp, "target_plane");

    if (!cJSON_IsString(type) || !type->valuestring ||
        strcmp(type->valuestring, "yai.exec.reply.v1") != 0) {
        cJSON_Delete(resp);
        reply_set(out, "error", "PROTOCOL_ERROR", "bad_response_type",
                  "Reply envelope type is invalid.",
                  fallback_command_id, "", "runtime");
        return YAI_SDK_PROTOCOL;
    }

    const cJSON *summary = cJSON_GetObjectItemCaseSensitive(resp, "summary");
    const cJSON *hint = cJSON_GetObjectItemCaseSensitive(resp, "hint");
    const cJSON *hints = cJSON_GetObjectItemCaseSensitive(resp, "hints");
    const cJSON *details = cJSON_GetObjectItemCaseSensitive(resp, "details");

    reply_set(out,
              cJSON_IsString(status) ? status->valuestring : "error",
              cJSON_IsString(code) ? code->valuestring : "PROTOCOL_ERROR",
              cJSON_IsString(reason) ? reason->valuestring : "missing_reason",
              cJSON_IsString(summary) ? summary->valuestring : NULL,
              cJSON_IsString(command_id) ? command_id->valuestring : fallback_command_id,
              cJSON_IsString(trace_id) ? trace_id->valuestring : "",
              cJSON_IsString(target_plane) ? target_plane->valuestring : "runtime");

    const cJSON *harr = cJSON_IsArray(hints) ? hints : (cJSON_IsArray(hint) ? hint : NULL);
    if (harr)
    {
        const cJSON *h0 = cJSON_GetArrayItem(harr, 0);
        const cJSON *h1 = cJSON_GetArrayItem(harr, 1);
        if (cJSON_IsString(h0) && h0->valuestring && h0->valuestring[0])
        {
            snprintf(out->hints[0], sizeof(out->hints[0]), "%s", h0->valuestring);
            out->hint_count = 1;
        }
        if (cJSON_IsString(h1) && h1->valuestring && h1->valuestring[0])
        {
            snprintf(out->hints[1], sizeof(out->hints[1]), "%s", h1->valuestring);
            out->hint_count = 2;
        }
    }
    if (cJSON_IsObject(details))
    {
        char *tmp = cJSON_PrintUnformatted((cJSON *)details);
        if (tmp)
        {
            snprintf(out->details, sizeof(out->details), "%s", tmp);
            free(tmp);
        }
    }

    cJSON_Delete(resp);
    return yai_reply_map_rc(out->status, out->code);
}

int yai_sdk_client_ping(yai_sdk_client_t *c, const char *command_id, yai_sdk_reply_t *out)
{
    char buf[256];
    uint32_t out_len = 0;
    const char *cid = (command_id && command_id[0]) ? command_id : "yai.runtime.ping";
    const char *reason = "runtime_ping_ok";
    const char *plane = "runtime";
    if (!c || !out) {
        return YAI_SDK_BAD_ARGS;
    }
    reply_zero(out);

    if (c->auto_handshake && !c->handshaken) {
        int hrc = yai_sdk_client_handshake(c);
        if (hrc != 0) {
            reply_set(out, "error", "RUNTIME_NOT_READY", "runtime_not_ready",
                      "Runtime handshake is not ready.", cid, "", plane);
            snprintf(out->summary, sizeof(out->summary), "Runtime handshake is not ready.");
            return hrc;
        }
    }

    int rc = yai_rpc_call_raw(&c->rpc, YAI_CMD_PING, NULL, 0, buf, sizeof(buf) - 1, &out_len);
    if (rc != 0) {
        if (rc == -5) {
            reply_set(out, "error", "SERVER_UNAVAILABLE", "server_unavailable",
                      "Runtime endpoint is unreachable.", cid, "", plane);
            snprintf(out->summary, sizeof(out->summary), "Runtime endpoint is unreachable.");
            return YAI_SDK_SERVER_OFF;
        }
        reply_set(out, "error", "PROTOCOL_ERROR", "rpc_call_failed",
                  "RPC request failed.", cid, "", plane);
        snprintf(out->summary, sizeof(out->summary), "RPC request failed.");
        return YAI_SDK_RPC;
    }
    if (out_len >= sizeof(buf)) {
        out_len = (uint32_t)(sizeof(buf) - 1);
    }
    buf[out_len] = '\0';
    reply_set(out, "ok", "OK", reason, "Command completed.", cid, "", plane);
    {
        char json[768];
        int n = snprintf(json, sizeof(json),
                         "{\"type\":\"yai.exec.reply.v1\",\"status\":\"%s\",\"code\":\"%s\",\"reason\":\"%s\",\"summary\":\"%s\",\"command_id\":\"%s\",\"target_plane\":\"%s\",\"trace_id\":\"\"}",
                         out->status, out->code, out->reason, out->summary, out->command_id, out->target_plane);
        if (n > 0 && (size_t)n < sizeof(json)) {
            out->exec_reply_json = dup_bytes(json, (size_t)n);
        } else {
            out->exec_reply_json = dup_bytes(buf, out_len);
        }
    }
    return YAI_SDK_OK;
}

void yai_sdk_reply_free(yai_sdk_reply_t *r)
{
    if (!r) {
        return;
    }
    free(r->exec_reply_json);
    r->exec_reply_json = NULL;
}

int yai_sdk_client_call(yai_sdk_client_t *c, const yai_sdk_control_call_t *call, yai_sdk_reply_t *out)
{
    char payload[4096];
    if (!c || !call || !out) {
        return YAI_SDK_BAD_ARGS;
    }
    if (yai_sdk_control_call_to_json(call, payload, sizeof(payload)) != 0) {
        return YAI_SDK_BAD_ARGS;
    }
    return yai_sdk_client_call_json(c, payload, out);
}

int yai_sdk_reply_runtime_state(const yai_sdk_reply_t *reply, yai_sdk_runtime_state_t *out)
{
    if (!reply || !reply->exec_reply_json || !out) {
        return YAI_SDK_BAD_ARGS;
    }
    if (yai_sdk_runtime_state_from_reply_json(reply->exec_reply_json, out) != 0) {
        return YAI_SDK_PROTOCOL;
    }
    return YAI_SDK_OK;
}

int yai_sdk_reply_governance_state(const yai_sdk_reply_t *reply, yai_sdk_governance_state_t *out)
{
    if (!reply || !reply->exec_reply_json || !out) {
        return YAI_SDK_BAD_ARGS;
    }
    if (yai_sdk_governance_state_from_reply_json(reply->exec_reply_json, out) != 0) {
        return YAI_SDK_PROTOCOL;
    }
    return YAI_SDK_OK;
}
