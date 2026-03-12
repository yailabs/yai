/* SPDX-License-Identifier: Apache-2.0 */
#define _POSIX_C_SOURCE 200809L

#include <yai/runtime/dispatch.h>
#include <yai/runtime/authority.h>
#include <yai/runtime/session.h>
#include <yai/runtime/workspace.h>

#include <yai/protocol/control/errors.h>
#include <yai/protocol/rpc/contract.h>
#include <yai/protocol/control/roles.h>
#include <yai/protocol/control/ids.h>

#include <errno.h>
#include <stdio.h>
#include <string.h>

static int send_response(int fd,
                         const yai_rpc_envelope_t *req,
                         uint32_t cmd,
                         const void *payload,
                         uint32_t payload_len)
{
    yai_rpc_envelope_t resp;
    memset(&resp, 0, sizeof(resp));

    resp.magic = YAI_FRAME_MAGIC;
    resp.version = YAI_PROTOCOL_IDS_VERSION;
    resp.command_id = cmd;
    resp.payload_len = payload_len;
    snprintf(resp.ws_id, sizeof(resp.ws_id), "%s", req->ws_id);
    snprintf(resp.trace_id, sizeof(resp.trace_id), "%s", req->trace_id);
    resp.role = req->role;
    resp.arming = req->arming;
    resp.checksum = 0;

    return yai_control_write_frame(fd, &resp, payload);
}

static int send_error_response(int fd,
                               const yai_rpc_envelope_t *req,
                               uint32_t code,
                               const char *reason)
{
    char payload[256];
    int n = snprintf(payload,
                     sizeof(payload),
                     "{\"status\":\"error\",\"code\":%u,\"reason\":\"%s\"}",
                     code,
                     reason ? reason : "unknown");
    if (n <= 0 || (size_t)n >= sizeof(payload))
        return -1;

    yai_rpc_envelope_t safe_req;
    memset(&safe_req, 0, sizeof(safe_req));
    if (req)
        safe_req = *req;

    return send_response(fd,
                         &safe_req,
                         safe_req.command_id ? safe_req.command_id : YAI_CMD_CONTROL,
                         payload,
                         (uint32_t)n);
}

static int is_valid_role(uint16_t role)
{
    return role == YAI_ROLE_NONE ||
           role == YAI_ROLE_USER ||
           role == YAI_ROLE_OPERATOR ||
           role == YAI_ROLE_SYSTEM;
}

static int extract_control_command_id(const char *payload, char *out, size_t out_cap)
{
    const char *needle = "\"command_id\":\"";
    const char *p = NULL;
    const char *q = NULL;
    size_t n = 0;
    if (!payload || !out || out_cap == 0)
        return -1;
    out[0] = '\0';
    p = strstr(payload, needle);
    if (!p)
        return -1;
    p += strlen(needle);
    q = strchr(p, '"');
    if (!q || q <= p)
        return -1;
    n = (size_t)(q - p);
    if (n + 1 > out_cap)
        n = out_cap - 1;
    memcpy(out, p, n);
    out[n] = '\0';
    return out[0] ? 0 : -1;
}

static int is_source_plane_command_id(const char *command_id)
{
    return command_id && strncmp(command_id, "yai.source.", 11) == 0;
}

int yai_dispatch_frame(
    int client_fd,
    const yai_rpc_envelope_t *env,
    const char *payload,
    ssize_t payload_len,
    int *handshake_done,
    yai_runtime_ingress_kind_t ingress_kind)
{
    char control_command_id[96];
    if (!env || !handshake_done)
        return -1;

    if (env->magic != YAI_FRAME_MAGIC)
        return send_error_response(client_fd, env, YAI_E_BAD_MAGIC, "bad_magic");
    if (env->version != YAI_PROTOCOL_IDS_VERSION)
        return send_error_response(client_fd, env, YAI_E_BAD_VERSION, "bad_version");
    if (env->payload_len > YAI_MAX_PAYLOAD)
        return send_error_response(client_fd, env, YAI_E_PAYLOAD_TOO_BIG, "payload_too_big");
    if (env->checksum != 0)
        return send_error_response(client_fd, env, YAI_E_BAD_CHECKSUM, "bad_checksum");
    if (env->arming > 1)
        return send_error_response(client_fd, env, YAI_E_ARMING_REQUIRED, "arming_flag_invalid");
    if (!is_valid_role(env->role))
        return send_error_response(client_fd, env, YAI_E_ROLE_REQUIRED, "role_invalid");
    if (!yai_ws_id_is_valid(env->ws_id))
        return send_error_response(client_fd, env, YAI_E_BAD_WS_ID, "bad_ws_id");

    if (env->command_id == YAI_CMD_HANDSHAKE)
    {
        if ((size_t)payload_len != sizeof(yai_handshake_req_t))
            return send_error_response(client_fd, env, YAI_E_PAYLOAD_TOO_BIG, "bad_handshake_payload_size");

        yai_handshake_ack_t ack;
        memset(&ack, 0, sizeof(ack));
        ack.server_version = YAI_PROTOCOL_IDS_VERSION;
        ack.capabilities_granted = 0;
        ack.session_id = 1;
        ack.status = (uint8_t)YAI_PROTO_STATE_READY;
        ack._pad = 0;

        if (send_response(client_fd, env, YAI_CMD_HANDSHAKE, &ack, (uint32_t)sizeof(ack)) != 0)
            return -1;
        *handshake_done = 1;
        return 0;
    }

    if (!*handshake_done)
        return send_error_response(client_fd, env, YAI_E_NEED_HANDSHAKE, "need_handshake");

    /* NP-2: peer ingress is source-plane scoped and not a generic remote runtime control plane. */
    if (ingress_kind == YAI_RUNTIME_INGRESS_PEER)
    {
        if (env->command_id != YAI_CMD_CONTROL_CALL)
            return send_error_response(client_fd, env, YAI_E_ROLE_REQUIRED, "peer_ingress_source_plane_only");
        if (extract_control_command_id(payload, control_command_id, sizeof(control_command_id)) != 0)
            return send_error_response(client_fd, env, YAI_E_BAD_WS_ID, "peer_ingress_missing_command_id");
        if (!is_source_plane_command_id(control_command_id))
            return send_error_response(client_fd, env, YAI_E_ROLE_REQUIRED, "peer_ingress_source_command_required");
    }

    {
        yai_authority_evaluation_t auth_eval;
        char auth_err[96];
        if (yai_authority_command_gate(NULL,
                                       env->command_id,
                                       env->role,
                                       env->arming,
                                       &auth_eval,
                                       auth_err,
                                       sizeof(auth_err)) != 0) {
            return send_error_response(client_fd, env, YAI_E_ROLE_REQUIRED, "authority_gate_failed");
        }
        if (auth_eval.decision == YAI_AUTHORITY_DENY) {
            if (strcmp(auth_eval.reason, "operator_arming_required") == 0) {
                if (env->role != YAI_ROLE_OPERATOR)
                    return send_error_response(client_fd, env, YAI_E_ROLE_REQUIRED, "role_required");
                return send_error_response(client_fd, env, YAI_E_ARMING_REQUIRED, "arming_required");
            }
            return send_error_response(client_fd, env, YAI_E_ROLE_REQUIRED, "authority_denied");
        }
    }

    if (payload_len < 0)
        return send_error_response(client_fd, env, YAI_E_PAYLOAD_TOO_BIG, "bad_payload");

    yai_session_dispatch(client_fd, env, payload_len > 0 ? payload : "");
    return 0;
}
